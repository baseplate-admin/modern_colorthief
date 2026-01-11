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
manylinux_wheels = [w for w in wheels if "manylinux" in w]

if not manylinux_wheels:
    print("No manylinux wheels found! Cannot update bin package.")
else:
    # Prefer CPython wheels
    cpython_wheels = [
        w
        for w in manylinux_wheels
        if "-cp" in os.path.basename(w) and "-pp" not in os.path.basename(w)
    ]
    if cpython_wheels:
        # Filter out free-threaded (t) builds
        regular_cpython = [
            w for w in cpython_wheels if not re.search(r"cp\d+t", os.path.basename(w))
        ]
        if regular_cpython:
            cpython_wheels = regular_cpython
        wheels_to_consider = cpython_wheels
    else:
        wheels_to_consider = manylinux_wheels

    # Sort to find suitable wheels (top 3)
    wheels_to_consider.sort(reverse=True)
    selected_wheels = wheels_to_consider[:3]
    print(f"Selected wheels: {[os.path.basename(w) for w in selected_wheels]}")

    wheel_data = []

    for wheel_path in selected_wheels:
        wheel_name = os.path.basename(wheel_path)

        # Calculate SHA256 of local file
        sha256_hash = hashlib.sha256()
        with open(wheel_path, "rb") as f:
            for byte_block in iter(lambda: f.read(4096), b""):
                sha256_hash.update(byte_block)
        sha256 = sha256_hash.hexdigest()
        print(f"SHA256 ({wheel_name}): {sha256}")

        # Construct download URL
        download_url = (
            f"https://github.com/{repo}/releases/download/{version}/{wheel_name}"
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
