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

# Filter for manylinux, x86_64, CPython (no PyPy, no free-threaded)
wheels_to_consider = [
    w
    for w in wheels
    if "manylinux" in w
    and "x86_64" in w
    and "-cp" in os.path.basename(w)
    and "-pp" not in os.path.basename(w)
    and not re.search(r"cp\d+t", os.path.basename(w))
]

if not wheels_to_consider:
    print("No suitable manylinux x86_64 CPython wheels found!")
    # Fallback or exit? If strict, we might want to just proceed with empty list or error.
    # But usually update scripts should fail if they can't find what they need.
    pass
else:
    # Sort to find suitable wheels
    wheels_to_consider.sort(reverse=True)

    # Allow picking specific python version via env var
    target_py = os.environ.get("TARGET_PYTHON_VERSION")
    if target_py:
        print(f"Filtering wheels for {target_py}...")
        cpython_wheels_filtered = [
            w for w in wheels_to_consider if target_py in os.path.basename(w)
        ]
        if cpython_wheels_filtered:
            wheels_to_consider = cpython_wheels_filtered
        else:
            print(f"Warning: No wheels found for {target_py}")

    # Select only the single best match (Top 1)
    selected_wheels = wheels_to_consider[:1]
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
        new_cmd = """_pyver="cp$(${PYTHON:-python} -c 'import sys; print(f"{sys.version_info.major}{sys.version_info.minor}")')"
    ${PYTHON:-python} -m installer --destdir="$pkgdir" *"${_pyver}"*.whl"""
        content = content.replace(
            'python -m installer --destdir="$pkgdir" *.whl', new_cmd
        )

    with open(pkgbuild_path, "w", encoding="utf-8") as f:
        f.write(content)

# --- HANDLE GIT PACKAGE ---
print("Processing modern-colorthief-git...")
# No explicit changes needed for git package PKGBUILD
