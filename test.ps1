param([switch]$Offline)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$cargoArgs = @('test', '--manifest-path', (Join-Path $projectRoot 'core\Cargo.toml'), '--locked')
if ($Offline) { $cargoArgs += '--offline' }
Push-Location $projectRoot
try {
    & cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) { throw 'Rust tests failed.' }
    $output = Join-Path $projectRoot 'tools\.test-classes'
    New-Item -ItemType Directory -Path $output -Force | Out-Null
    $tests = @('PlaybackOrder', 'ArtworkDiskStore', 'ArtworkSeed')
    $sources = foreach ($name in $tests) {
        Join-Path $projectRoot "android\app\src\main\java\dev\qqlyric\app\$name.java"
        Join-Path $projectRoot "tools\${name}Test.java"
    }
    $javac = if ($env:JAVA_HOME) { Join-Path $env:JAVA_HOME 'bin\javac.exe' } else { 'javac' }
    $java = if ($env:JAVA_HOME) { Join-Path $env:JAVA_HOME 'bin\java.exe' } else { 'java' }
    & $javac -encoding UTF-8 -d $output @sources
    if ($LASTEXITCODE -ne 0) { throw 'Java test compilation failed.' }
    foreach ($name in $tests) {
        & $java -cp $output "${name}Test"
        if ($LASTEXITCODE -ne 0) { throw "$name tests failed." }
    }
} finally { Pop-Location }
