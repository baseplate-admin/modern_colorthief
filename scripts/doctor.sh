#!/usr/bin/env bash
# doctor.sh — assert every tool/version the repo needs (Linux/macOS + CI).
# Usage: bash scripts/doctor.sh
# Exits non-zero on the first failure so CI and contributors fail fast.

set -u
FAILURES=0

ok() { printf 'OK       %s\n' "$1"; }
miss() { printf 'MISSING  %s\n' "$1"; FAILURES=$((FAILURES + 1)); }

have() { command -v "$1" >/dev/null 2>&1; }

check_cmd() { # name [version-args...]
  local name=$1; shift
  if have "$name"; then
    ok "$name -> $("$name" "$@" 2>/dev/null | head -n1)"
  else
    miss "$name"
  fi
}

check_py_module() {
  if python3 -c "import importlib.util, sys; sys.exit(0 if importlib.util.find_spec('$1') else 1)" 2>/dev/null; then
    ok "python module $1"
  else
    miss "python module $1"
  fi
}

echo "=== modern_colorthief doctor ==="

# Rust toolchain
check_cmd cargo -V
check_cmd rustc -V
check_cmd rustup -V

# wasm target + llvm-tools component
if rustup target list --installed 2>/dev/null | grep -q '^wasm32-unknown-unknown$'; then
  ok "rustup target wasm32-unknown-unknown"
else
  miss "rustup target wasm32-unknown-unknown (rustup target add wasm32-unknown-unknown)"
fi
if rustup component list 2>/dev/null | grep -q 'llvm-tools.* (installed)'; then
  ok "rustup component llvm-tools-preview"
else
  miss "rustup component llvm-tools-preview (rustup component add llvm-tools-preview)"
fi

# Rust utilities
check_cmd wasm-pack -V
check_cmd cargo-llvm-cov -V
check_cmd cargo-nextest -V

# Node.js >= 22 (repo targets Node 26)
if have node; then
  nodever=$(node -v 2>/dev/null)
  major=${nodever#v}
  if [ "${major:-0}" -ge 22 ]; then
    ok "node $nodever (major >= 22)"
  else
    miss "node $nodever < 22"
  fi
else
  miss "node"
fi

# Python >= 3.10 + pytest stack
if have python3; then
  pymajor=$(python3 -c 'import sys; print(sys.version_info[0])' 2>/dev/null)
  pyminor=$(python3 -c 'import sys; print(sys.version_info[1])' 2>/dev/null)
  if [ "${pymajor:-0}" -gt 3 ] || { [ "${pymajor:-0}" -eq 3 ] && [ "${pyminor:-0}" -ge 10 ]; }; then
    ok "python $pymajor.$pyminor (>= 3.10)"
  else
    miss "python $pymajor.$pyminor < 3.10"
  fi
  check_py_module pytest
  check_py_module pytest_asyncio
  check_py_module pytest_cov
else
  miss "python3"
fi

# JVM (optional — needed only for the JVM bindings workstream)
if have java; then
  java -version 2>&1 | head -n1 | sed 's/^/       /'
else
  echo "OPTIONAL java not found (needed only for JVM bindings)"
fi

echo
if [ "$FAILURES" -gt 0 ]; then
  echo "doctor: $FAILURES problem(s) — see MISSING lines above"
  exit 1
fi
echo "doctor: all checks passed"
exit 0
