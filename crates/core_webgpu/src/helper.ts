declare const __WGSL_SHADER_CODE__: string;
declare const GPUBufferUsage: Record<"UNIFORM" | "STORAGE" | "COPY_SRC" | "COPY_DST" | "MAP_READ", GPUBufferUsageFlags>;
declare const GPUMapMode: { readonly READ: GPUMapModeFlags };

interface ExtractPaletteInput {
    pixels: Uint8Array<ArrayBuffer>;
    w: number;
    h: number;
    maxC: number;
    quality: number;
}

interface Bucket {
    start: number;
    length: number;
    axis: number;
    volume: number;
    uniform: boolean;
    color: number[];
}

/** Stable median cut with packed pixels resident on the GPU. */
async function extractPaletteOnGpu(gpu: GPU, input: ExtractPaletteInput): Promise<Uint8Array> {
    const cache = globalThis as typeof globalThis & { __wt_gpu_device?: Promise<GPUDevice> };
    if (!cache.__wt_gpu_device) {
        const pending = (async () => {
            const adapter = await gpu.requestAdapter();
            if (!adapter) throw new Error("No GPU adapter is available on this device");
            return adapter.requestDevice();
        })();
        cache.__wt_gpu_device = pending;
        const clear = () => {
            if (cache.__wt_gpu_device === pending) delete cache.__wt_gpu_device;
        };
        pending.then((device) => { void device.lost.then(clear); }, clear);
    }
    const device = await cache.__wt_gpu_device;

    if (input.w <= 0 || input.h <= 0) throw new Error("Image dimensions must be positive");
    const pixelCount = Math.floor(input.pixels.length / 4);
    const quality = Math.max(input.quality, 1);
    const sampleCount = Math.ceil(pixelCount / quality);
    const storageLimit = Math.min(device.limits.maxBufferSize, device.limits.maxStorageBufferBindingSize);
    const dispatchLimit = device.limits.maxComputeWorkgroupsPerDimension;
    const inputBytes = pixelCount * 4;
    const tileSize = 1024;
    const tileCount = Math.ceil(sampleCount / tileSize);
    const tileBytes = tileCount * 256 * 4;
    const sampleWorkgroups = Math.min(Math.ceil(sampleCount / 64), dispatchLimit);
    if (inputBytes > storageLimit || tileBytes > storageLimit ||
        tileCount > dispatchLimit) {
        throw new Error("Image exceeds this device's WebGPU limits. Use the CPU engine or choose a smaller image.");
    }

    const buffers: GPUBuffer[] = [];
    function allocate(size: number, usage: GPUBufferUsageFlags) {
        const buffer = device.createBuffer({ size, usage });
        buffers.push(buffer);
        return buffer;
    }

    try {
        const params = allocate(16, GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST);
        const samples = allocate(sampleCount * 4, GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST);
        // After sampling, the upload buffer doubles as counting-sort output.
        const scratch = allocate(inputBytes, GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST | GPUBufferUsage.COPY_SRC);
        const tiles = allocate(tileBytes, GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST);
        const staging = allocate(tileBytes, GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ);
        const shader = device.createShaderModule({ code: `${__WGSL_SHADER_CODE__}` });

        function prepare(entryPoint: string, bindings: [number, GPUBuffer][]) {
            const pipeline = device.createComputePipeline({ layout: "auto", compute: { module: shader, entryPoint } });
            const group = device.createBindGroup({
                layout: pipeline.getBindGroupLayout(0),
                entries: bindings.map(([binding, buffer]) => ({ binding, resource: { buffer } })),
            });
            return { pipeline, group };
        }

        const sample = prepare("sample_pixels", [[0, params], [1, samples], [2, scratch]]);
        const summary = prepare("summarize", [[0, params], [1, samples], [3, tiles]]);
        const histogram = prepare("histogram", [[0, params], [1, samples], [3, tiles]]);
        const scatter = prepare("scatter", [[0, params], [1, samples], [2, scratch], [3, tiles]]);

        function encode(
            encoder: GPUCommandEncoder,
            stage: { pipeline: GPUComputePipeline; group: GPUBindGroup },
            workgroups: number,
        ) {
            const pass = encoder.beginComputePass();
            pass.setPipeline(stage.pipeline);
            pass.setBindGroup(0, stage.group);
            pass.dispatchWorkgroups(workgroups);
            pass.end();
        }

        async function readTiles(
            stage: { pipeline: GPUComputePipeline; group: GPUBindGroup },
            workgroups: number,
            words: number,
        ) {
            const encoder = device.createCommandEncoder();
            encode(encoder, stage, workgroups);
            encoder.copyBufferToBuffer(tiles, 0, staging, 0, words * 4);
            device.queue.submit([encoder.finish()]);
            await staging.mapAsync(GPUMapMode.READ, 0, words * 4);
            try {
                return new Uint32Array(staging.getMappedRange(0, words * 4)).slice();
            } finally {
                staging.unmap();
            }
        }

        async function summarize(start: number, length: number): Promise<Bucket> {
            const count = Math.ceil(length / tileSize);
            device.queue.writeBuffer(params, 0, new Uint32Array([start, length, 0, 0]));
            const values = await readTiles(summary, Math.ceil(count / 64), count * 5);
            const low = [255, 255, 255];
            const high = [0, 0, 0];
            const sums = [0, 0, 0];
            for (let offset = 0; offset < values.length; offset += 5) {
                for (let channel = 0; channel < 3; channel++) {
                    low[channel] = Math.min(low[channel], (values[offset] >>> (channel * 8)) & 255);
                    high[channel] = Math.max(high[channel], (values[offset + 1] >>> (channel * 8)) & 255);
                    sums[channel] += values[offset + 2 + channel];
                }
            }
            const [r, g, b] = high.map((max, channel) => max - low[channel]);
            return {
                start,
                length,
                axis: r >= g && r >= b ? 0 : g >= b ? 1 : 2,
                volume: Math.max(r * g * b, 1) * length,
                uniform: r === 0 && g === 0 && b === 0,
                color: sums.map((sum) => Math.floor(sum / length)),
            };
        }

        async function sortBucket(bucket: Bucket) {
            const count = Math.ceil(bucket.length / tileSize);
            device.queue.writeBuffer(params, 0, new Uint32Array([bucket.start, bucket.length, bucket.axis, 0]));
            const offsets = await readTiles(histogram, count, count * 256);

            // Prefix by channel value and then by original tile. The GPU walks
            // each tile in order, matching the CPU's stable channel sort.
            let next = 0;
            for (let value = 0; value < 256; value++) {
                for (let tile = 0; tile < count; tile++) {
                    const index = tile * 256 + value;
                    const length = offsets[index];
                    offsets[index] = next;
                    next += length;
                }
            }
            device.queue.writeBuffer(tiles, 0, offsets);
            const encoder = device.createCommandEncoder();
            encode(encoder, scatter, count);
            const offset = bucket.start * 4;
            encoder.copyBufferToBuffer(scratch, offset, samples, offset, bucket.length * 4);
            device.queue.submit([encoder.finish()]);
        }

        device.queue.writeBuffer(scratch, 0, input.pixels.subarray(0, inputBytes));
        device.queue.writeBuffer(params, 0, new Uint32Array([0, sampleCount, 0, quality]));
        const encoder = device.createCommandEncoder();
        encode(encoder, sample, sampleWorkgroups);
        device.queue.submit([encoder.finish()]);

        const first = await summarize(0, sampleCount);
        if (first.uniform) return new Uint8Array(first.color);
        const buckets = [first];

        while (buckets.length < input.maxC) {
            let selected = -1;
            let volume = 0;
            for (let index = 0; index < buckets.length; index++) {
                const bucket = buckets[index];
                if (bucket.length > 1 && bucket.volume > volume) {
                    selected = index;
                    volume = bucket.volume;
                }
            }
            if (selected < 0) break;

            const [bucket] = buckets.splice(selected, 1);
            if (!bucket.uniform) await sortBucket(bucket);
            const middle = Math.floor(bucket.length / 2);
            buckets.push(await summarize(bucket.start, middle));
            buckets.push(await summarize(bucket.start + middle, bucket.length - middle));
        }

        return new Uint8Array(buckets.flatMap((bucket) => bucket.color));
    } finally {
        for (const buffer of buffers) buffer.destroy();
    }
}
