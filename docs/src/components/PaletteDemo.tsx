import { useCallback, useEffect, useRef, useState } from 'react';
import styles from './PaletteDemo.module.css';

type Rgb = [number, number, number];

type WasmGlue = {
  default?: () => Promise<unknown>;
  getPaletteFromPixels: (
    pixels: Uint8Array,
    width: number,
    height: number,
    colorCount?: number,
    quality?: number,
  ) => Promise<Rgb[]>;
};

type WasmGpuGlue = {
  default?: () => Promise<unknown>;
  getPaletteGpu?: (
    pixels: Uint8Array,
    width: number,
    height: number,
    colorCount?: number,
    quality?: number,
  ) => Promise<Rgb[]>;
  getPaletteFromPixels?: (
    pixels: Uint8Array,
    width: number,
    height: number,
    colorCount?: number,
    quality?: number,
  ) => Promise<Rgb[]>;
};

type Engine = 'wasm' | 'webgpu' | 'auto';

const BASE = import.meta.env.BASE_URL;
const WASM_MODULE = BASE + 'wasm/modern_colorthief_wasm.js';
const WASM_GPU_MODULE = BASE + 'wasm-webgpu/modern_colorthief_wasm_webgpu.js';

let cpuGlue: Promise<WasmGlue> | undefined;
let gpuGlue: Promise<WasmGpuGlue> | undefined;
let webgpuAvailable: boolean | undefined;

function loadCpu(): Promise<WasmGlue> {
  cpuGlue ??= import(/* @vite-ignore */ WASM_MODULE).then(async (mod: WasmGlue) => {
    await mod.default?.();
    return mod;
  });
  return cpuGlue;
}

function loadGpu(): Promise<WasmGpuGlue> {
  gpuGlue ??= import(/* @vite-ignore */ WASM_GPU_MODULE).then(async (mod: WasmGpuGlue) => {
    await mod.default?.();
    return mod;
  });
  return gpuGlue;
}

async function detectWebGpu(): Promise<boolean> {
  if (webgpuAvailable !== undefined) return webgpuAvailable;
  if (typeof navigator === 'undefined' || !navigator.gpu) {
    webgpuAvailable = false;
    return false;
  }
  try {
    const adapter = await navigator.gpu.requestAdapter();
    webgpuAvailable = adapter !== null;
  } catch {
    webgpuAvailable = false;
  }
  return webgpuAvailable;
}

// Decoding happens here rather than in `getPalette()`: the WASM image decoder
// polls `img.complete` on the main thread, which deadlocks because the image's
// load event can never run while the poll loop holds the thread.
async function decodeImage(
  source: string | File,
): Promise<{ pixels: Uint8Array; width: number; height: number }> {
  let blob: Blob;
  if (typeof source === 'string') {
    const response = await fetch(source);
    if (!response.ok) {
      throw new Error(`${response.status} ${response.statusText}`);
    }
    blob = await response.blob();
  } else {
    blob = source;
  }

  const bitmap = await createImageBitmap(blob);
  try {
    const canvas = document.createElement('canvas');
    canvas.width = bitmap.width;
    canvas.height = bitmap.height;

    const context = canvas.getContext('2d', { willReadFrequently: true });
    if (!context) {
      throw new Error('2D canvas context unavailable');
    }
    context.drawImage(bitmap, 0, 0);

    const { data, width, height } = context.getImageData(0, 0, bitmap.width, bitmap.height);
    return { pixels: new Uint8Array(data.buffer, data.byteOffset, data.byteLength), width, height };
  } finally {
    bitmap.close();
  }
}

type Status = { kind: 'idle' | 'busy' | 'error'; message: string };

function toHex([r, g, b]: Rgb): string {
  return '#' + ((1 << 24) | (r << 16) | (g << 8) | b).toString(16).slice(1);
}

export default function PaletteDemo() {
  const [status, setStatus] = useState<Status>({
    kind: 'idle',
    message: 'Loading WebAssembly…',
  });
  const [engine, setEngine] = useState<Engine>('auto');
  const [gpuSupported, setGpuSupported] = useState<boolean | undefined>(undefined);
  const [colors, setColors] = useState<Rgb[]>([]);
  const [copied, setCopied] = useState<number | null>(null);
  const [dragging, setDragging] = useState(false);
  const urlInput = useRef<HTMLInputElement>(null);
  const fileInput = useRef<HTMLInputElement>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      const [cpu, gpu] = await Promise.allSettled([loadCpu(), detectWebGpu()]);
      if (cancelled) return;
      if (cpu.status === 'fulfilled') {
        setStatus({ kind: 'idle', message: 'Ready - drop an image or paste a URL.' });
      } else {
        setStatus({ kind: 'error', message: `Failed to load WASM: ${cpu.reason}` });
      }
      setGpuSupported(gpu.status === 'fulfilled' ? gpu.value : false);
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const effectiveEngine: Engine =
    engine === 'webgpu' && gpuSupported === false ? 'wasm' : engine;

  const extract = useCallback(
    async (image: string | File) => {
      setStatus({ kind: 'busy', message: 'Extracting palette…' });
      setColors([]);
      try {
        const { pixels, width, height } = await decodeImage(image);
        let palette: Rgb[];
        let usedEngine: 'wasm' | 'webgpu';

        if (effectiveEngine === 'webgpu') {
          const glue = await loadGpu();
          const fn = glue.getPaletteGpu ?? glue.getPaletteFromPixels;
          if (!fn) throw new Error('WebGPU bundle does not expose getPaletteGpu');
          palette = await fn.call(glue, pixels, width, height, 10, 10);
          usedEngine = 'webgpu';
        } else if (effectiveEngine === 'wasm') {
          const glue = await loadCpu();
          palette = await glue.getPaletteFromPixels(pixels, width, height, 10, 10);
          usedEngine = 'wasm';
        } else {
          // auto: prefer WebGPU when available, fall back to WASM
          if (gpuSupported) {
            try {
              const glue = await loadGpu();
              const fn = glue.getPaletteGpu ?? glue.getPaletteFromPixels;
              if (fn) {
                palette = await fn.call(glue, pixels, width, height, 10, 10);
                usedEngine = 'webgpu';
              }
            } catch {
              // fall through to WASM
            }
          }
          if (!palette) {
            const glue = await loadCpu();
            palette = await glue.getPaletteFromPixels(pixels, width, height, 10, 10);
            usedEngine = 'wasm';
          }
        }

        setColors(palette);
        setStatus({
          kind: 'idle',
          message: `${palette.length} colors (${usedEngine}) - click a swatch to copy its hex.`,
        });
      } catch (error) {
        setStatus({ kind: 'error', message: `Failed: ${error}` });
      }
    },
    [effectiveEngine, gpuSupported],
  );

  const copy = useCallback(async (index: number, hex: string) => {
    try {
      await navigator.clipboard.writeText(hex);
      setCopied(index);
      window.setTimeout(() => setCopied((current) => (current === index ? null : current)), 800);
    } catch {
      setStatus({ kind: 'error', message: `Could not copy ${hex}` });
    }
  }, []);

  const engineLabel =
    effectiveEngine === 'webgpu' ? 'WebGPU (GPU)' : effectiveEngine === 'wasm' ? 'WASM (CPU)' : 'Auto';

  return (
    <div className={styles.root}>
      <div
        className={styles.drop}
        data-over={dragging}
        onDragEnter={(event) => {
          event.preventDefault();
          setDragging(true);
        }}
        onDragOver={(event) => {
          event.preventDefault();
          setDragging(true);
        }}
        onDragLeave={() => setDragging(false)}
        onDrop={(event) => {
          event.preventDefault();
          setDragging(false);
          const file = event.dataTransfer.files[0];
          if (file?.type.startsWith('image/')) {
            void extract(file);
          } else if (file) {
            setStatus({ kind: 'error', message: 'That file is not an image.' });
          }
        }}
      >
        <input
          ref={fileInput}
          type="file"
          accept="image/*"
          onChange={(event) => {
            const file = event.target.files?.[0];
            if (file) void extract(file);
            event.target.value = '';
          }}
        />
        <span>
          Drop an image here, or <strong>browse</strong>
        </span>
      </div>

      <form
        className={styles.url}
        onSubmit={(event) => {
          event.preventDefault();
          const url = urlInput.current?.value.trim();
          if (url) void extract(url);
        }}
      >
        <input
          ref={urlInput}
          type="text"
          placeholder="…or paste an image URL"
          aria-label="Image URL"
        />
        <button type="submit">Extract</button>
      </form>

      <div className={styles.engine}>
        <label>
          Engine:{' '}
          <select
            value={engine}
            onChange={(event) => setEngine(event.target.value as Engine)}
            disabled={engine === 'webgpu' && gpuSupported === false}
          >
            <option value="auto">Auto (WebGPU first)</option>
            <option value="webgpu" disabled={gpuSupported === false}>
              WebGPU {gpuSupported === false ? '(unavailable)' : ''}
            </option>
            <option value="wasm">WASM (CPU)</option>
          </select>
        </label>
        <span className={styles.engineNote} title={engineLabel}>
          {gpuSupported === undefined
            ? 'Checking WebGPU…'
            : gpuSupported
              ? 'WebGPU available'
              : 'WebGPU unavailable - using WASM'}
        </span>
      </div>

      <p
        className={styles.status}
        data-kind={status.kind}
        role={status.kind === 'error' ? 'alert' : 'status'}
      >
        {status.message}
      </p>

      <div className={styles.swatches}>
        {colors.map((color, index) => {
          const hex = toHex(color);
          return (
            <button
              key={`${hex}-${index}`}
              type="button"
              className={styles.swatch}
              style={{ background: `rgb(${color[0]}, ${color[1]}, ${color[2]})` }}
              title={`rgb(${color[0]}, ${color[1]}, ${color[2]})`}
              onClick={() => void copy(index, hex)}
            >
              <span>{copied === index ? 'copied!' : hex}</span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
