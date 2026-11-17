# doctor.ps1 — assert every tool/version the repo needs.
# Usage: powershell -ExecutionPolicy Bypass -File scripts/doctor.ps1
# Exits non-zero on the first failure so CI and contributors fail fast.

$ErrorActionPreference = "Stop"
$failures = @()

function Test-Command {
    param([string]$Name, [string]$VersionCmd)
    $cmd = Get-Command $Name -ErrorAction SilentlyContinue
    if (-not $cmd) {
        $script:failures += "$Name not found on PATH"
        Write-Host "MISSING  $Name" -ForegroundColor Red
        return $false
    }
    try {
        $ver = & $Name $VersionCmd 2>&1 | Select-Object -First 1
        Write-Host "OK       $Name -> $ver" -ForegroundColor Green
        return $true
    } catch {
        Write-Host "OK       $Name (installed, version probe failed: $_)" -ForegroundColor Yellow
        return $true
    }
}

function Test-PythonModule {
    param([string]$Module)
    $out = python -c "import importlib.util; exit(0 if importlib.util.find_spec('$Module') else 1)" 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "OK       python module $Module" -ForegroundColor Green
        return $true
    } else {
        $script:failures += "python module $Module not installed"
        Write-Host "MISSING  python module $Module" -ForegroundColor Red
        return $false
    }
}

Write-Host "=== modern_colorthief doctor ===" -ForegroundColor Cyan

# Rust toolchain
Test-Command cargo "-V" | Out-Null
Test-Command rustc "-V" | Out-Null
Test-Command rustup "-V" | Out-Null

# wasm target + components (checked via rustup, no network needed)
$wasmTarget = rustup target list --installed 2>&1 | Select-String "wasm32-unknown-unknown"
if ($wasmTarget) {
    Write-Host "OK       rustup target wasm32-unknown-unknown" -ForegroundColor Green
} else {
    $script:failures += "rustup target wasm32-unknown-unknown missing"
    Write-Host "MISSING  rustup target wasm32-unknown-unknown (rustup target add wasm32-unknown-unknown)" -ForegroundColor Red
}
$llvm = rustup component list 2>&1 | Select-String "llvm-tools.*\(installed\)"
if ($llvm) {
    Write-Host "OK       rustup component llvm-tools-preview" -ForegroundColor Green
} else {
    $script:failures += "rustup component llvm-tools-preview missing"
    Write-Host "MISSING  rustup component llvm-tools-preview (rustup component add llvm-tools-preview)" -ForegroundColor Red
}

# Rust utilities
Test-Command wasm-pack "-V" | Out-Null
Test-Command cargo-llvm-cov "-V" | Out-Null
Test-Command cargo-nextest "-V" | Out-Null

# Node.js (>= 22 required; repo targets Node 26)
$node = Test-Command node "-v"
if ($node) {
    $nodeVer = & node -v 2>&1 | Select-Object -First 1
    $major = [int](($nodeVer -replace '^v','' -split '\.')[0])
    if ($major -ge 22) {
        Write-Host "OK       node major $major >= 22" -ForegroundColor Green
    } else {
        $script:failures += "node $nodeVer < 22"
        Write-Host "BAD      node $nodeVer < 22 required" -ForegroundColor Red
    }
}

# Python (>= 3.10 for abi3-py39 wheels and pytest stack)
$py = Test-Command python "-V"
if ($py) {
    $pyVer = & python -c "import sys; print(sys.version_info[0], sys.version_info[1])"
    $pymajor = [int]($pyVer -split ' ')[0]
    $pyminor = [int]($pyVer -split ' ')[1]
    if (($pymajor -gt 3) -or (($pymajor -eq 3) -and ($pyminor -ge 10))) {
        Write-Host "OK       python $pymajor.$pyminor >= 3.10" -ForegroundColor Green
    } else {
        $script:failures += "python $pymajor.$pyminor < 3.10"
        Write-Host "BAD      python $pymajor.$pyminor < 3.10 required" -ForegroundColor Red
    }
    Test-PythonModule pytest | Out-Null
    Test-PythonModule pytest_asyncio | Out-Null
    Test-PythonModule pytest_cov | Out-Null
}

# JVM (optional — only needed for the JVM bindings workstream)
if (Get-Command java -ErrorAction SilentlyContinue) {
    $ErrorActionPreference = "Continue"
    $javaVer = & java -version 2>&1 | Select-Object -First 1
    $ErrorActionPreference = "Stop"
    Write-Host "       $javaVer" -ForegroundColor DarkGray
} else {
    Write-Host "OPTIONAL java not found (needed only for JVM bindings)" -ForegroundColor Yellow
}

Write-Host ""
if ($failures.Count -gt 0) {
    Write-Host "doctor: $($failures.Count) problem(s):" -ForegroundColor Red
    $failures | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
    exit 1
}
Write-Host "doctor: all checks passed" -ForegroundColor Green
exit 0
