/**
 * Input parameters for the WebGPU palette extraction.
 */
interface ExtractPaletteInput {
    /** Raw pixel data in RGBA format (4 bytes per pixel). */
    pixels: Uint8Array;
    /** Image width in pixels. */
    w: number;
    /** Image height in pixels. */
    h: number;
    /** Maximum number of colors to extract. */
    maxC: number;
    /** Sampling quality (1 = every pixel, higher = more sparse). */
    quality: number;
}

/**
 * WebGPU-based color palette extraction.
 *
 * Uses WebGPU compute shaders to efficiently extract dominant colors from raw
 * pixel data. Runs entirely on the GPU for sampling, with CPU-side deduplication
 * to guarantee correctness (avoids race conditions in concurrent GPU dedup).
 *
 * The GPU device is cached on globalThis so it survives across js_eval() calls.
 * Dawn (Node.js WebGPU) expects devices to be long-lived and reused.
 *
 * @param gpu - GPU instance from the global object (passed from Rust).
 * @param input - The input parameters for palette extraction.
 * @returns A flat Uint8Array of RGB values (3 bytes per color).
 */
async function extractPaletteOnGpu(gpu: GPU, input: ExtractPaletteInput): Promise<Uint8Array> {
    // Get or create a cached GPU device
    // globalThis persists across js_eval() calls in the same runtime
    let device = (globalThis as any)["__wt_gpu_device"] as GPUDevice | null;
    if (!device) {
        if (!gpu) {
            throw new Error("WebGPU is not supported in this environment");
        }
        const adapter: GPUAdapter | null = await gpu.requestAdapter();
        if (!adapter) {
            throw new Error("No GPU adapter is available on this device");
        }
        device = await adapter.requestDevice();
    }
    // Set up device.lost handler and cache the device
    // (separate from device assignment to avoid minifier chaining issues)
    if (device) {
        device.lost.then(() => {
            (globalThis as any)["__wt_gpu_device"] = null;
        });
        (globalThis as any)["__wt_gpu_device"] = device;
    }

    const rawPixels: Uint8Array = input.pixels;
    const imageWidth: number = input.w;
    const imageHeight: number = input.h;
    const maxColors: number = input.maxC;
    const samplingQuality: number = input.quality;

    if (imageWidth <= 0 || imageHeight <= 0) {
        throw new Error("Image dimensions must be positive");
    }

    const totalPixels = imageWidth * imageHeight;
    const inputBytes = totalPixels * 4;
    const storageLimit = Math.min(device.limits.maxBufferSize, device.limits.maxStorageBufferBindingSize);
    if (inputBytes > storageLimit) {
        throw new Error(`Image needs ${inputBytes} bytes of WebGPU storage; this device supports ${storageLimit}. Use the CPU engine or choose a smaller image.`);
    }

    const numberOfChunks = Math.ceil(Math.sqrt(totalPixels)) * 4;
    const chunkStride = 16; // RGB plus a flag marking chunks that contain pixels.
    const chunkBytes = numberOfChunks * chunkStride;
    const uniformBufferSize = 24; // 6 u32 values.
    const pixelData = rawPixels.subarray(0, inputBytes);

    // Allocate the uniform buffer for shader parameters
    const uniformBuffer: GPUBuffer = device.createBuffer({
        size: uniformBufferSize,
        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });

    // Allocate the input pixel buffer
    const pixelBuffer: GPUBuffer = device.createBuffer({
        size: inputBytes, // One packed RGBA u32 per pixel
        usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
    });

    // Intermediate buffer for per-chunk average colors
    const chunkColorBuffer: GPUBuffer = device.createBuffer({
        size: chunkBytes,
        usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    });

    // Staging buffer to read chunk colors back from GPU to CPU
    const stagingChunkBuffer: GPUBuffer = device.createBuffer({
        size: chunkBytes,
        usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
    });

    // Create the shader module from the embedded WGSL code
    const wgslCode: string = `${__WGSL_SHADER_CODE__}`;
    const shaderModule: GPUShaderModule = device.createShaderModule({
        code: wgslCode,
    });

    // Sampling pipeline: sample pixels and compute per-chunk average colors
    const samplingPipeline: GPUComputePipeline = device.createComputePipeline({
        layout: "auto",
        compute: { module: shaderModule, entryPoint: "main" },
    });

    // Bind group for the sampling pipeline (uses bindings 0, 1, 2)
    const samplingBindGroup: GPUBindGroup = device.createBindGroup({
        layout: samplingPipeline.getBindGroupLayout(0),
        entries: [
            { binding: 0, resource: { buffer: pixelBuffer } },
            { binding: 1, resource: { buffer: uniformBuffer } },
            { binding: 2, resource: { buffer: chunkColorBuffer } },
        ],
    });

    // Upload uniform data to the GPU
    device.queue.writeBuffer(
        uniformBuffer,
        0,
        new Uint32Array([
            imageWidth,
            imageHeight,
            maxColors,
            samplingQuality,
            totalPixels,
            numberOfChunks,
        ]),
    );

    // Upload raw pixel data to the GPU
    device.queue.writeBuffer(pixelBuffer, 0, pixelData as unknown as GPUAllowSharedBufferSource);

    // Encode and dispatch the sampling compute pass
    const samplingEncoder: GPUCommandEncoder = device.createCommandEncoder();
    const samplingPass: GPUComputePassEncoder = samplingEncoder.beginComputePass();
    samplingPass.setPipeline(samplingPipeline);
    samplingPass.setBindGroup(0, samplingBindGroup);
    samplingPass.dispatchWorkgroups(Math.ceil(numberOfChunks / 64));
    samplingPass.end();

    // Copy chunk colors to staging buffer for CPU readback
    const copyEncoder: GPUCommandEncoder = device.createCommandEncoder();
    copyEncoder.copyBufferToBuffer(
        chunkColorBuffer,
        0,
        stagingChunkBuffer,
        0,
        chunkBytes,
    );

    // Submit sampling + copy and wait for GPU completion
    device.queue.submit([samplingEncoder.finish(), copyEncoder.finish()]);

    try {
        await stagingChunkBuffer.mapAsync(GPUMapMode.READ);
        const chunkView = new DataView(stagingChunkBuffer.getMappedRange());
        const seen = new Set<string>();
        const unique: number[] = [];

        for (let i = 0; i < numberOfChunks && unique.length / 3 < maxColors; i++) {
            const offset = i * chunkStride;
            if (chunkView.getFloat32(offset + 12, true) === 0) continue;

            const r = Math.round(chunkView.getFloat32(offset, true) * 255);
            const g = Math.round(chunkView.getFloat32(offset + 4, true) * 255);
            const b = Math.round(chunkView.getFloat32(offset + 8, true) * 255);
            const key = `${r},${g},${b}`;
            if (!seen.has(key)) {
                seen.add(key);
                unique.push(r, g, b);
            }
        }

        return new Uint8Array(unique);
    } finally {
        stagingChunkBuffer.destroy();
        chunkColorBuffer.destroy();
        pixelBuffer.destroy();
        uniformBuffer.destroy();
    }
}
