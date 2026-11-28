# Open Images corpus test

Runs palette extraction over real photographs from
[Google Open Images](https://storage.googleapis.com/openimages/web/index.html)
to catch crashes, invariant violations, and regressions that synthetic tests
miss.

The dataset is never committed or downloaded as a whole. `fetch_open_images.py`
reads the public image-list CSV, picks a deterministic, evenly spaced sample,
and streams only those JPEGs from the unsigned `open-images-dataset` S3 bucket.
CI does this on the runner (`.github/workflows/corpus.yml`).

Images are processed in parallel with rayon, so a run uses every available core.
Set `RAYON_NUM_THREADS` to cap the worker count.

## Run it locally

```bash
python3 corpus/fetch_open_images.py --count 1000 --out corpus/images
CORPUS_DIR=corpus/images cargo run --release --manifest-path corpus/Cargo.toml
```

Environment variables:

| Variable | Meaning |
| --- | --- |
| `CORPUS_DIR` | Directory containing `.jpg`, `.jpeg`, or `.png` images (required) |
| `CORPUS_LIMIT` | Process at most this many images |
| `CORPUS_GPU` | Set to `1` to also exercise the Vulkan backend |

## What is checked

For every decoded image:

- CPU palette extraction for color counts `1, 5, 10, 16` at sampling quality 10
  returns a non-empty palette no larger than `color_count`.
- One full-resolution (quality 1) pass covers the slowest, most allocation-heavy
  path on every image.
- Two identical calls return identical palettes.
- `extract_color` matches the head of the 5-color palette.
- With `CORPUS_GPU=1`, one representative GPU extraction preserves the same
  invariants. Machines without a Vulkan device skip the GPU checks and report it.

CI downloads 1000 images on pull requests and 5000 on the weekly schedule or a
manual run. Pass a different `image_count` when dispatching the workflow.
