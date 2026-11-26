param([switch]$Offline, [string]$Aapt2Path = '')
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
& (Join-Path $projectRoot 'build-ui.ps1') -Offline:$Offline
if ($LASTEXITCODE -ne 0) { throw 'Rust UI build failed.' }
$sdk = if ($env:ANDROID_HOME) { $env:ANDROID_HOME } elseif ($env:ANDROID_SDK_ROOT) { $env:ANDROID_SDK_ROOT } else { throw 'Set ANDROID_HOME or ANDROID_SDK_ROOT to the Android SDK.' }
$ndkRoot = if ($env:ANDROID_NDK_HOME) { $env:ANDROID_NDK_HOME } else {
    $candidate = Get-ChildItem (Join-Path $sdk 'ndk') -Directory | Sort-Object Name -Descending | Select-Object -First 1
    if (-not $candidate) { throw 'Android NDK is missing.' }
    $candidate.FullName
}
$bin = Join-Path $ndkRoot 'toolchains\llvm\prebuilt\windows-x86_64\bin'
$clang = Join-Path $bin 'aarch64-linux-android26-clang.cmd'
if (-not (Test-Path $clang)) { throw "Android arm64 compiler is missing: $clang" }
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = $clang
$env:CC_aarch64_linux_android = $clang
$env:AR_aarch64_linux_android = Join-Path $bin 'llvm-ar.exe'
$tempDir = Join-Path $projectRoot 'core\.tmp'
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
$env:TEMP = $tempDir
$env:TMP = $tempDir
$cargoArgs = @('build', '--release', '--target', 'aarch64-linux-android', '--locked')
if ($Offline) { $cargoArgs += '--offline' }
Push-Location (Join-Path $projectRoot 'core')
try { & cargo @cargoArgs; if ($LASTEXITCODE -ne 0) { throw 'Rust build failed.' } } finally { Pop-Location }
$native = Join-Path $projectRoot 'core\target\aarch64-linux-android\release\libqqlyric_core.so'
$destination = Join-Path $projectRoot 'android\app\src\main\jniLibs\arm64-v8a\libqqlyric_core.so'
New-Item -ItemType Directory -Path (Split-Path $destination) -Force | Out-Null
Copy-Item $native $destination -Force
Push-Location (Join-Path $projectRoot 'android')
try {
    $gradleArgs = @(':app:assembleDebug')
    if ($Offline) { $gradleArgs += '--offline' }
    if ($Aapt2Path) { $gradleArgs += "-Pandroid.aapt2FromMavenOverride=$Aapt2Path" }
    & .\gradlew.bat @gradleArgs
    if ($LASTEXITCODE -ne 0) { throw 'Android build failed.' }
} finally { Pop-Location }
Write-Host (Join-Path $projectRoot 'android\app\build\outputs\apk\debug\app-debug.apk')
