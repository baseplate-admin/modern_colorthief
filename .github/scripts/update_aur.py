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
    if cpython_wheels:
        # Sort CPython wheels (reverse=True picks highest version, e.g. cp311 > cp310)
        cpython_wheels.sort(reverse=True)
        best_wheel_path = cpython_wheels[0]
    elif pypy_wheels:
        pypy_wheels.sort(reverse=True)
        best_wheel_path = pypy_wheels[0]
    else:
        # Fallback if naming convention doesn't match expected cp/pp
        manylinux_wheels.sort(reverse=True)
        best_wheel_path = manylinux_wheels[0]

    best_wheel_name = os.path.basename(best_wheel_path)
    print(f"Selected wheel: {best_wheel_name}")

    # Calculate SHA256 of local file
    sha256_hash = hashlib.sha256()
    with open(best_wheel_path, "rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)
    sha256 = sha256_hash.hexdigest()
    print(f"SHA256: {sha256}")

    # Construct download URL for PKGBUILD
    # https://github.com/<owner>/<repo>/releases/download/<tag>/<filename>
    download_url = (
        f"https://github.com/{repo}/releases/download/{version}/{best_wheel_name}"
    )
    print(f"Download URL: {download_url}")

    # Update PKGBUILD
    pkgbuild_path = "aur/modern-colorthief/PKGBUILD"
    with open(pkgbuild_path, "r", encoding="utf-8") as f:
        content = f.read()

    # Update version, source, checksum
    # content = re.sub(r"^pkgver=.*", f"pkgver={version}", content, flags=re.MULTILINE)
    content = re.sub(
        r"^source=\(.*?\)", f'source=("{download_url}")', content, flags=re.MULTILINE
    )
    content = re.sub(
        r"^sha256sums=\(.*?\)", f"sha256sums=('{sha256}')", content, flags=re.MULTILINE
    )

    with open(pkgbuild_path, "w", encoding="utf-8") as f:
        f.write(content)

# --- HANDLE GIT PACKAGE ---
print("Processing modern-colorthief-git...")
# No explicit changes needed for git package PKGBUILD
