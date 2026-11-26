param([switch]$Offline)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Push-Location $projectRoot
try {
    if (-not (Get-Command wasm-bindgen -ErrorAction SilentlyContinue)) { throw 'Install wasm-bindgen-cli 0.2.108 and the wasm32-unknown-unknown target. See docs/BUILD.md.' }
    $version = & wasm-bindgen --version
    if ($version -notmatch '0\.2\.108$') { throw 'The Rust UI needs wasm-bindgen-cli 0.2.108 to match Cargo.lock.' }
    $cargoArgs = @('build', '--manifest-path', 'ui/Cargo.toml', '--release', '--target', 'wasm32-unknown-unknown', '--locked')
    if ($Offline) { $cargoArgs += '--offline' }
    & cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) { throw 'Rust UI compilation failed.' }
    $destination = Join-Path $projectRoot 'android/app/src/main/assets/ui'
    New-Item -ItemType Directory -Path $destination -Force | Out-Null
    & wasm-bindgen 'ui/target/wasm32-unknown-unknown/release/auralis_ui.wasm' --target web --out-dir (Join-Path $destination 'pkg')
    if ($LASTEXITCODE -ne 0) { throw 'Rust UI bindings failed.' }
    Copy-Item -Path 'ui/host/*' -Destination $destination -Force
    Write-Host "Rust UI assets: $destination"
} finally { Pop-Location }
