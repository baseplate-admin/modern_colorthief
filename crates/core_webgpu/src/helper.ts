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

    // Derived dimensions for the compute shaders
    const totalPixels: number = imageWidth * imageHeight;
    const numberOfChunks: number = Math.ceil(Math.sqrt(totalPixels)) * 4;
    const uniformBufferSize: number = 24; // 6 u32 values x 4 bytes

    // Validate dimensions
    if (imageWidth <= 0 || imageHeight <= 0) {
        throw new Error("Image dimensions must be positive");
    }

   // Convert RGBA u8 (0-255) to vec4<f32> (0.0-1.0) for the shader
    const pixelData: ArrayBuffer = new ArrayBuffer(totalPixels * 16);
    const pixelView: DataView = new DataView(pixelData);
    for (let i = 0; i < totalPixels; i++) {
        pixelView.setFloat32(i * 16,     rawPixels[i * 4]     / 255.0, true);
        pixelView.setFloat32(i * 16 + 4, rawPixels[i * 4 + 1] / 255.0, true);
        pixelView.setFloat32(i * 16 + 8, rawPixels[i * 4 + 2] / 255.0, true);
        pixelView.setFloat32(i * 16 + 12, rawPixels[i * 4 + 3] / 255.0, true);
    }

    // Allocate the uniform buffer for shader parameters
    const uniformBuffer: GPUBuffer = device.createBuffer({
        size: uniformBufferSize,
        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });

    // Allocate the input pixel buffer
    const pixelBuffer: GPUBuffer = device.createBuffer({
        size: totalPixels * 16, // vec4<f32> per pixel = 16 bytes
        usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
    });

    // Intermediate buffer for per-chunk average colors
    const chunkColorBuffer: GPUBuffer = device.createBuffer({
        size: numberOfChunks * 12, // vec3<f32> per chunk = 12 bytes
        usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    });

    // Staging buffer to read chunk colors back from GPU to CPU
    const stagingChunkBuffer: GPUBuffer = device.createBuffer({
        size: numberOfChunks * 12,
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
        numberOfChunks * 12,
    );

    // Submit sampling + copy and wait for GPU completion
    device.queue.submit([samplingEncoder.finish(), copyEncoder.finish()]);

    // Map staging buffer and read chunk colors back to CPU
    await stagingChunkBuffer.mapAsync(GPUMapMode.READ);
    const chunkArrayBuffer: ArrayBuffer = stagingChunkBuffer.getMappedRange();
    const chunkView: DataView = new DataView(chunkArrayBuffer);

    // Convert float32 color data (0.0-1.0) to uint8 (0-255) and deduplicate
    const seen = new Set<string>();
    const unique: number[] = [];

    for (let i = 0; i < numberOfChunks && unique.length < maxColors; i++) {
        const r = Math.round(chunkView.getFloat32(i * 12, true) * 255);
        const g = Math.round(chunkView.getFloat32(i * 12 + 4, true) * 255);
        const b = Math.round(chunkView.getFloat32(i * 12 + 8, true) * 255);
        const key = `${r},${g},${b}`;
        if (!seen.has(key)) {
            seen.add(key);
            unique.push(r, g, b);
        }
    }

    // Clean up: unmap and release staging buffer
    stagingChunkBuffer.unmap();

    return new Uint8Array(unique);
}
