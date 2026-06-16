#!/usr/bin/env bash
# setup_vulkan_macos.sh
# Configure MoltenVK / SwiftShader / Lavapipe Vulkan environment for macOS CI.
#
# The jakoch/install-vulkan-sdk-action sets VULKAN_SDK and DYLD_LIBRARY_PATH,
# but our Rust code (vulkan.rs::find_vulkan_loader) searches for
# libMoltenVK.dylib in hardcoded paths + $MOLTENVK_ROOT/libMoltenVK.dylib.
# This script bridges the gap by locating the dylib and exporting MOLTENVK_ROOT.
#
# Usage: bash .github/scripts/setup_vulkan_macos.sh

set -euo pipefail

echo "::group::Setup Vulkan (macOS)"

# --- Locate libMoltenVK.dylib ---
# Search order:
#  1. $VULKAN_SDK (set by jakoch/install-vulkan-sdk-action)
#  2. Homebrew prefixes (/usr/local/lib, /opt/homebrew/lib)
#  3. Common SDK subdirectories
#  4. find in $HOME/vulkan-sdk as fallback

MOLTENVK=""

find_in_sdk() {
  local sdk="${VULKAN_SDK:-}"
  [ -z "$sdk" ] && return 0

  # Direct paths inside the SDK
  for candidate in \
    "$sdk/lib/libMoltenVK.dylib" \
    "$sdk/libMoltenVK.dylib" \
    "$sdk/macOS/lib/libMoltenVK.dylib" \
    "$sdk/MoltenVK/framework/MoltenVK.framework/MoltenVK" \
    "$sdk/MoltenVK/lib/libMoltenVK.dylib"; do
    if [ -f "$candidate" ]; then
      MOLTENVK="$(dirname "$candidate")"
      return 0
    fi
  done

  # Recursive search (limited depth to avoid scanning everything)
  local found
  found="$(find "$sdk" -maxdepth 4 -name 'libMoltenVK.dylib' -print -quit 2>/dev/null || true)"
  if [ -n "$found" ]; then
    MOLTENVK="$(dirname "$found")"
    return 0
  fi
}

find_in_homebrew() {
  for prefix in /opt/homebrew /usr/local; do
    candidate="$prefix/lib/libMoltenVK.dylib"
    if [ -f "$candidate" ]; then
      MOLTENVK="$prefix/lib"
      return 0
    fi
  done
}

find_in_home() {
  local found
  found="$(find "$HOME" -maxdepth 5 -name 'libMoltenVK.dylib' -print -quit 2>/dev/null || true)"
  if [ -n "$found" ]; then
    MOLTENVK="$(dirname "$found")"
    return 0
  fi
}

# Try each source in priority order
find_in_sdk || find_in_homebrew || find_in_home || true

if [ -z "$MOLTENVK" ]; then
  echo "::error::libMoltenVK.dylib not found. Vulkan GPU tests will fail."
  echo "VULKAN_SDK=${VULKAN_SDK:-<not set>}"
  echo "HOME=$HOME"
  if [ -n "${VULKAN_SDK:-}" ]; then
    echo "SDK contents:"
    ls -la "$VULKAN_SDK" 2>/dev/null || true
    echo "SDK lib contents:"
    ls -la "$VULKAN_SDK/lib" 2>/dev/null || true
  fi
  exit 1
fi

echo "Found MoltenVK at: $MOLTENVK"

# --- Export environment variables for subsequent CI steps ---
# GITHUB_ENV is persisted across steps in a GitHub Actions workflow.
# We also export for the current shell in case this script is sourced.

export MOLTENVK_ROOT="$MOLTENVK"

if [ -n "${GITHUB_ENV:-}" ]; then
  echo "MOLTENVK_ROOT=$MOLTENVK" >> "$GITHUB_ENV"
  echo "Exported MOLTENVK_ROOT to GITHUB_ENV"
fi

# --- Verify the loader is loadable ---
if ! dyld "$MOLTENVK/libMoltenVK.dylib" >/dev/null 2>&1; then
  # Try otool to at least verify it's a valid Mach-O
  if otool -hv "$MOLTENVK/libMoltenVK.dylib" >/dev/null 2>&1; then
    echo "libMoltenVK.dylib is a valid Mach-O binary"
  else
    echo "::warning::Cannot verify libMoltenVK.dylib loadability"
  fi
fi

# --- Verify Vulkan ICDs are discoverable ---
# The Vulkan loader discovers ICDs from VK_ICD_FILENAMES or standard locations.
# Show what ICDs are available for debugging.
echo "Vulkan ICD search paths:"
for icd_dir in \
  "${VULKAN_SDK:-}/etc/vulkan/icd.d" \
  "${VULKAN_SDK:-}/share/vulkan/icd.d" \
  "/etc/vulkan/icd.d" \
  "/usr/share/vulkan/icd.d" \
  "$HOME/lavapipe/share/vulkan/icd.d" \
  "$HOME/swiftshader"; do
  if [ -d "$icd_dir" ]; then
    echo "  $icd_dir:"
    ls "$icd_dir"/*.json 2>/dev/null | while read -r f; do
      echo "    - $f"
    done
  fi
done

# --- Optional: run vulkaninfo if available ---
vulkaninfo_cmd=""
for candidate in \
  "${VULKAN_SDK:-}/bin/vulkaninfo" \
  "vulkaninfo"; do
  if command -v "$candidate" >/dev/null 2>&1 || [ -x "$candidate" ]; then
    vulkaninfo_cmd="$candidate"
    break
  fi
done

if [ -n "$vulkaninfo_cmd" ]; then
  echo "Running vulkaninfo --summary..."
  VK_ICD_FILENAMES="" $vulkaninfo_cmd --summary 2>&1 || true
else
  echo "vulkaninfo not found (optional verification skipped)"
fi

echo "::endgroup::"
echo "Vulkan setup complete. MOLTENVK_ROOT=$MOLTENVK"
