# recallweave demo (PowerShell) — mirrors examples/demo.sh.
# Builds a small store with the real binary, drives the full lifecycle, and
# exports a memory-pack. Deterministic: every command pins --now.
#
# Run from the project root:  pwsh examples/demo.ps1

$ErrorActionPreference = "Stop"

$root  = Split-Path -Parent $PSScriptRoot
$bin   = Join-Path $root "target\release\recallweave.exe"
$store = Join-Path $root "examples\demo-store"
$pack  = Join-Path $root "examples\memory-pack.json"

if (-not (Test-Path $bin)) {
    Write-Host "building release binary..."
    Push-Location $root; cargo build --release; Pop-Location
}
