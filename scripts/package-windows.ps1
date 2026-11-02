$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
cargo build --release --locked --target x86_64-pc-windows-msvc
if ($LASTEXITCODE -ne 0) { throw 'Windows build failed' }
$BuildDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { 'target' }
$Destination = 'dist/windows-x64'
python scripts/download_payload.py windows (Join-Path $BuildDir 'x86_64-pc-windows-msvc/release/radio-crash-desktop.exe')
if ($LASTEXITCODE -ne 0) { throw 'Windows payload preparation failed' }
$Version = ((Select-String -Path 'Cargo.toml' -Pattern '^version = "([^"]+)"').Matches[0].Groups[1].Value)
Compress-Archive -Path $Destination -DestinationPath "dist/Radio-Crash-Desktop-$Version-Windows-x64.zip" -Force
Copy-Item "dist/Radio-Crash-Desktop-$Version-Windows-x64.zip" 'dist/Radio-Crash-Desktop-Windows-x64.zip' -Force
