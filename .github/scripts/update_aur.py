import os
import sys
import glob
import hashlib
import re

# Get release info
ref = os.environ.get("GITHUB_REF", "")
version = ref.replace("refs/tags/", "")
print(f"Detected version: {version}")

repo = os.environ.get("GITHUB_REPOSITORY")

# --- HANDLE BIN PACKAGE (WHEEL) ---
print("Processing modern-colorthief...")

# Find wheel in dist/
# We downloaded artifacts to 'dist' dir in previous step
dist_dir = "dist"
if not os.path.exists(dist_dir):
    print(f"Distribution directory {dist_dir} not found!")
    sys.exit(1)

wheels = glob.glob(os.path.join(dist_dir, "*.whl"))
print(f"Found wheels: {wheels}")

# Filter for manylinux
# We prioritize x86_64 as it's the standard Arch architecture.
manylinux_wheels = [w for w in wheels if "manylinux" in w and "x86_64" in w]
if not manylinux_wheels:
    print("No x86_64 manylinux wheels found! Falling back to all manylinux wheels.")
    manylinux_wheels = [w for w in wheels if "manylinux" in w]

if not manylinux_wheels:
    print("No manylinux wheels found! Cannot update bin package.")
    sys.exit(1)
else:
    # Separate CPython and PyPy wheels
    cpython_wheels = []
    pypy_wheels = []

    for w in manylinux_wheels:
        base = os.path.basename(w)
        if "-cp" in base:
            # Check for free-threaded ABI (e.g. cp313t) which are not compatible with standard python
            # We want to exclude these unless no other option?
            # Heuristic: part starts with 'cp', ends with 't', and middle is digits.
            parts = base.split("-")
            is_free_threaded = False
            for part in parts:
                if (
                    part.startswith("cp")
                    and part.endswith("t")
                    and len(part) > 3
                    and part[2:-1].isdigit()
                ):
                    is_free_threaded = True
                    break

            if not is_free_threaded:
                cpython_wheels.append(w)
        elif "-pp" in base:
            pypy_wheels.append(w)

    # Sort to find a suitable wheel
    selected_wheels = []
    if cpython_wheels:
        # Sort CPython wheels (reverse=True picks highest version, e.g. cp311 > cp310)
        cpython_wheels.sort(reverse=True)
        selected_wheels = cpython_wheels[:3]
    elif pypy_wheels:
        pypy_wheels.sort(reverse=True)
        selected_wheels = pypy_wheels[:3]
    else:
        # Fallback if naming convention doesn't match expected cp/pp
        manylinux_wheels.sort(reverse=True)
        selected_wheels = manylinux_wheels[:3]

    print(f"Selected wheels: {[os.path.basename(w) for w in selected_wheels]}")

    wheel_data = []

    for wheel_path in selected_wheels:
        best_wheel_name = os.path.basename(wheel_path)

        # Calculate SHA256 of local file
        sha256_hash = hashlib.sha256()
        with open(wheel_path, "rb") as f:
            for byte_block in iter(lambda: f.read(4096), b""):
                sha256_hash.update(byte_block)
        sha256 = sha256_hash.hexdigest()
        print(f"SHA256 ({best_wheel_name}): {sha256}")

        # Construct download URL for PKGBUILD
        # https://github.com/<owner>/<repo>/releases/download/<tag>/<filename>
        download_url = (
            f"https://github.com/{repo}/releases/download/{version}/{best_wheel_name}"
        )
        wheel_data.append((download_url, sha256))

    # Update PKGBUILD
    pkgbuild_path = "aur/modern-colorthief/PKGBUILD"
    with open(pkgbuild_path, "r", encoding="utf-8") as f:
        content = f.read()

    sources_str = "source=(" + " ".join([f'"{url}"' for url, _ in wheel_data]) + ")"
    shas_str = "sha256sums=(" + " ".join([f"'{sha}'" for _, sha in wheel_data]) + ")"

    # Update version, source, checksum
    content = re.sub(r"^pkgver=.*", f"pkgver={version}", content, flags=re.MULTILINE)
    content = re.sub(r"^source=\(.*?\)", sources_str, content, flags=re.MULTILINE)
    content = re.sub(r"^sha256sums=\(.*?\)", shas_str, content, flags=re.MULTILINE)

    # Update package step to select correct wheel based on python version
    if 'python -m installer --destdir="$pkgdir" *.whl' in content:
        new_cmd = """_pyver="cp$(python -c 'import sys; print(f"{sys.version_info.major}{sys.version_info.minor}")')"
    python -m installer --destdir="$pkgdir" *"${_pyver}"*.whl"""
        content = content.replace(
            'python -m installer --destdir="$pkgdir" *.whl', new_cmd
        )

    with open(pkgbuild_path, "w", encoding="utf-8") as f:
        f.write(content)

# --- HANDLE GIT PACKAGE ---
print("Processing modern-colorthief-git...")
# No explicit changes needed for git package PKGBUILD
