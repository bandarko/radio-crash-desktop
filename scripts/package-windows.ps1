$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
cargo build --release --locked --target x86_64-pc-windows-msvc
if ($LASTEXITCODE -ne 0) { throw 'Windows build failed' }
$BuildDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { 'target' }
$Destination = 'dist/windows-x64'
New-Item -ItemType Directory -Force $Destination | Out-Null
Copy-Item (Join-Path $BuildDir 'x86_64-pc-windows-msvc/release/radio-crash-desktop.exe') "$Destination/RadioCrashDesktop.exe" -Force
Copy-Item 'RELEASE-NOTES.md', 'INSTALL-Windows.md', 'LICENSE', 'THIRD_PARTY_NOTICES.md', 'DEPENDENCIES.md', 'VERIFICATION.md', 'README.md', 'UI-ASSETS.md', 'assets/*OFL.txt', 'assets/*.prompt.txt' $Destination -Force
Copy-Item 'licenses' $Destination -Recurse -Force
Copy-Item 'RELEASE-NOTES.md' "$Destination/RELEASE-NOTES.txt" -Force
Copy-Item 'INSTALL-Windows.md' "$Destination/INSTALL.txt" -Force
$Version = ((Select-String -Path 'Cargo.toml' -Pattern '^version = "([^"]+)"').Matches[0].Groups[1].Value)
Compress-Archive -Path $Destination -DestinationPath "dist/Radio-Crash-Desktop-$Version-Windows-x64.zip" -Force
