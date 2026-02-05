# Phybkc Release Bundling Script

Write-Host "Building Phybkc in release mode..."
cargo build --release

$DistDir = "dist"
if (Test-Path $DistDir) {
    Remove-Item -Recurse -Force $DistDir
}

New-Item -ItemType Directory -Path $DistDir
New-Item -ItemType Directory -Path "$DistDir\profiles"
New-Item -ItemType Directory -Path "$DistDir\scripts"

Write-Host "Copying binaries..."
Copy-Item "target\release\daemon.exe" "$DistDir\"
Copy-Item "target\release\gui.exe" "$DistDir\"
Copy-Item "target\release\cli.exe" "$DistDir\"

Write-Host "Copying and reorganizing files..."
# Place config in root
Copy-Item "config.toml" "$DistDir\"
# Place JSONs in profiles
Copy-Item "profiles\*.json" "$DistDir\profiles\"
# Place phybkc in scripts
Copy-Item "scripts\*.phybkc" "$DistDir\scripts\"

Write-Host "Done! Release package is ready in the '$DistDir' folder."
