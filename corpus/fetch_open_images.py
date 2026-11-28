#!/usr/bin/env python3
"""Download a deterministic sample of Google Open Images for corpus testing.

The full dataset is ~500 GB, so this fetches only the image IDs selected from
the public image-list CSV and streams them from the unsigned Open Images S3
bucket. Nothing is committed to the repository; CI runs this on the runner.

Usage:
    python3 corpus/fetch_open_images.py --count 1000 --out corpus/images
    python3 corpus/fetch_open_images.py --split test --count 200
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
import sys
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

IMAGE_LISTS = {
    "train": "https://storage.googleapis.com/openimages/2018_04/train/train-images-boxable-with-rotation.csv",
    "validation": "https://storage.googleapis.com/openimages/2018_04/validation/validation-images-with-rotation.csv",
    "test": "https://storage.googleapis.com/openimages/2018_04/test/test-images-with-rotation.csv",
}
S3_BASE = "https://open-images-dataset.s3.amazonaws.com"
USER_AGENT = "modern-colorthief-corpus/1.0"
ATTEMPTS_PER_IMAGE = 3
TIMEOUT_SECONDS = 60


def read_image_ids(split: str) -> list[str]:
    url = IMAGE_LISTS[split]
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=TIMEOUT_SECONDS) as response:
        text = io.TextIOWrapper(response, encoding="utf-8-sig", newline="")
        reader = csv.DictReader(text)
        if not reader.fieldnames or "ImageID" not in reader.fieldnames:
            raise SystemExit(f"{url} did not contain an ImageID column")
        return [row["ImageID"] for row in reader if row.get("ImageID")]


def sample_evenly(ids: list[str], count: int) -> list[str]:
    """Pick `count` IDs spread evenly across the sorted, deduplicated list."""
    unique = sorted(set(ids))
    if count >= len(unique):
        return unique
    stride = len(unique) / count
    return [unique[int(index * stride)] for index in range(count)]


def fetch_one(split: str, image_id: str, out_dir: Path) -> tuple[str, bytes]:
    dest = out_dir / f"{image_id}.jpg"
    if dest.is_file() and dest.stat().st_size > 2:
        return image_id, dest.read_bytes()

    url = f"{S3_BASE}/{split}/{image_id}.jpg"
    last_error: Exception | None = None
    for _ in range(ATTEMPTS_PER_IMAGE):
        try:
            request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
            with urllib.request.urlopen(request, timeout=TIMEOUT_SECONDS) as response:
                data = response.read()
            if not data.startswith(b"\xff\xd8\xff"):
                raise ValueError("response is not a JPEG")
            dest.write_bytes(data)
            return image_id, data
        except (urllib.error.URLError, TimeoutError, ValueError) as error:
            last_error = error
    raise RuntimeError(f"{url}: {last_error}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--count", type=int, default=1000, help="images to download (default: 1000)")
    parser.add_argument("--split", choices=sorted(IMAGE_LISTS), default="validation")
    parser.add_argument("--out", type=Path, default=Path("corpus/images"))
    parser.add_argument("--workers", type=int, default=16)
    args = parser.parse_args()

    if args.count <= 0:
        raise SystemExit("--count must be positive")

    args.out.mkdir(parents=True, exist_ok=True)
    print(f"Reading {args.split} image list...")
    candidates = sample_evenly(read_image_ids(args.split), args.count * 3)
    print(f"Selected {len(candidates)} candidate IDs for {args.count} images")

    downloaded: list[dict] = []
    failed: list[str] = []
    with ThreadPoolExecutor(max_workers=args.workers) as executor:
        futures = {
            executor.submit(fetch_one, args.split, image_id, args.out): image_id
            for image_id in candidates
        }
        for future in as_completed(futures):
            image_id = futures[future]
            try:
                image_id, data = future.result()
            except Exception as error:  # noqa: BLE001 - report and continue
                failed.append(f"{image_id}: {error}")
                continue
            downloaded.append(
                {
                    "id": image_id,
                    "bytes": len(data),
                    "sha256": hashlib.sha256(data).hexdigest(),
                }
            )
            if len(downloaded) >= args.count:
                for pending in futures:
                    pending.cancel()
                break

    manifest = {
        "source": IMAGE_LISTS[args.split],
        "split": args.split,
        "requested": args.count,
        "downloaded": sorted(downloaded, key=lambda item: item["id"]),
        "failed": failed,
    }
    (args.out / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")

    print(
        f"Downloaded {len(downloaded)}/{args.count} images "
        f"({sum(item['bytes'] for item in downloaded) / (1024 * 1024):.1f} MiB) to {args.out}"
    )
    if len(downloaded) < args.count:
        print(f"Only {len(downloaded)} images available; {len(failed)} attempts failed", file=sys.stderr)
        raise SystemExit(1)


if __name__ == "__main__":
    main()
