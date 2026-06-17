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
 * pixel data. Runs entirely on the GPU, suitable for processing large images
 * in the browser without blocking the main thread.
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
    const pixelData: Float32Array = new Float32Array(totalPixels * 4);
    for (let i = 0; i < totalPixels; i++) {
        pixelData[i * 4]     = rawPixels[i * 4]     / 255.0;
        pixelData[i * 4 + 1] = rawPixels[i * 4 + 1] / 255.0;
        pixelData[i * 4 + 2] = rawPixels[i * 4 + 2] / 255.0;
        pixelData[i * 4 + 3] = rawPixels[i * 4 + 3] / 255.0;
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
        usage: GPUBufferUsage.STORAGE,
    });

    // Buffer to hold the final deduplicated color palette
    const uniqueColorBuffer: GPUBuffer = device.createBuffer({
        size: maxColors * 12,
        usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    });

    // Buffer to hold the count of unique colors found
    const colorCountBuffer: GPUBuffer = device.createBuffer({
        size: 4, // single u32
        usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    });

    // Staging buffers to read results back from GPU to CPU
    const stagingColorBuffer: GPUBuffer = device.createBuffer({
        size: maxColors * 12,
        usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
    });

    const stagingCountBuffer: GPUBuffer = device.createBuffer({
        size: 4,
        usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
    });

    // Create the shader module from the embedded WGSL code
    const wgslCode: string = `${__WGSL_SHADER_CODE__}`;
    const shaderModule: GPUShaderModule = device.createShaderModule({
        code: wgslCode,
    });

    // First pass: sample pixels and compute per-chunk average colors
    const samplingPipeline: GPUComputePipeline = device.createComputePipeline({
        layout: "auto",
        compute: { module: shaderModule, entryPoint: "main" },
    });

    // Second pass: deduplicate similar colors to produce the final palette
    const deduplicationPipeline: GPUComputePipeline = device.createComputePipeline({
        layout: "auto",
        compute: { module: shaderModule, entryPoint: "dedup" },
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

    // Bind group for the deduplication pipeline (uses bindings 1, 2, 3, 4)
    const deduplicationBindGroup: GPUBindGroup = device.createBindGroup({
        layout: deduplicationPipeline.getBindGroupLayout(0),
        entries: [
            { binding: 1, resource: { buffer: uniformBuffer } },
            { binding: 2, resource: { buffer: chunkColorBuffer } },
            { binding: 3, resource: { buffer: uniqueColorBuffer } },
            { binding: 4, resource: { buffer: colorCountBuffer } },
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

    // Encode and dispatch the deduplication compute pass
    const deduplicationEncoder: GPUCommandEncoder = device.createCommandEncoder();
    const deduplicationPass: GPUComputePassEncoder = deduplicationEncoder.beginComputePass();
    deduplicationPass.setPipeline(deduplicationPipeline);
    deduplicationPass.setBindGroup(0, deduplicationBindGroup);
    deduplicationPass.dispatchWorkgroups(Math.ceil(numberOfChunks / 64));
    deduplicationPass.end();

    // Copy results from GPU buffers to staging buffers for reading back
    const copyEncoder: GPUCommandEncoder = device.createCommandEncoder();
    copyEncoder.copyBufferToBuffer(
        uniqueColorBuffer,
        0,
        stagingColorBuffer,
        0,
        maxColors * 12,
    );
    copyEncoder.copyBufferToBuffer(
        colorCountBuffer,
        0,
        stagingCountBuffer,
        0,
        4,
    );

    // Submit all command buffers and wait for GPU completion
    device.queue.submit([
        samplingEncoder.finish(),
        deduplicationEncoder.finish(),
        copyEncoder.finish(),
    ]);

    // Map staging buffers and read results back to CPU
    await stagingColorBuffer.mapAsync(GPUMapMode.READ);
    await stagingCountBuffer.mapAsync(GPUMapMode.READ);

    const countView: DataView = new DataView(stagingCountBuffer.getMappedRange());
    const actualColorCount: number = countView.getUint32(0, true);
    const colorView: DataView = new DataView(stagingColorBuffer.getMappedRange());

    // Convert float32 color data to uint8 output array
    const result: Uint8Array = new Uint8Array(actualColorCount * 3);
    for (let i: number = 0; i < actualColorCount; i++) {
        result[i * 3] = Math.round(colorView.getFloat32(i * 12, true));
        result[i * 3 + 1] = Math.round(colorView.getFloat32(i * 12 + 4, true));
        result[i * 3 + 2] = Math.round(colorView.getFloat32(i * 12 + 8, true));
    }

    // Clean up: unmap staging buffers
    stagingColorBuffer.unmap();
    stagingCountBuffer.unmap();

    return result;
}
