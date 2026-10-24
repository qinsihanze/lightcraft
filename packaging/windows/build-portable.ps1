# Build and package LightCraft portable release
param(
  [string]$Arch = 'x64'
)

$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$Target = 'x86_64-pc-windows-msvc'
$Bin = Join-Path $Root "target\$Target\release"
$Dist = Join-Path $Root "dist\release"
$Version = "0.10.0"

New-Item -ItemType Directory -Force -Path $Dist | Out-Null

$PackageDir = Join-Path $Dist "LightCraft-v$Version-windows-$Arch"
Remove-Item -Recurse -Force $PackageDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $PackageDir | Out-Null

Write-Host "==> Packaging LightCraft v$Version ($Arch)..." -ForegroundColor Cyan

# Copy binaries
Copy-Item (Join-Path $Bin 'lightcraft.exe') $PackageDir
Copy-Item (Join-Path $Bin 'lightcraft-cli.exe') $PackageDir

# Copy shortcut installer
Copy-Item (Join-Path $PSScriptRoot 'Install-LightCraft.ps1') $PackageDir

# Generate font licenses
. (Join-Path $PSScriptRoot 'font-licences.ps1')
$CraftFontsDir = (Resolve-Path (Join-Path $Root '..\craft-fonts')).Path
Write-FontLicences (Join-Path $PackageDir 'FONT-LICENCES.txt') (Join-Path $Root 'assets\fonts\OFL-Inter.txt') $CraftFontsDir

# Copy docs and license files
foreach ($f in 'README.md', 'LICENSE', 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE') {
  $p = Join-Path $Root $f
  if (Test-Path $p) { Copy-Item $p $PackageDir }
}

# Create zip archive
$ZipPath = Join-Path $Dist "LightCraft-v$Version-windows-$Arch-portable.zip"
Remove-Item -Force $ZipPath -ErrorAction SilentlyContinue
Write-Host "==> Compressing archive to $ZipPath..." -ForegroundColor Cyan
Compress-Archive -Path "$PackageDir\*" -DestinationPath $ZipPath

Write-Host "==> Package ready!" -ForegroundColor Green
Get-Item $ZipPath | Format-Table Name, Length, LastWriteTime
Get-Item "$PackageDir\lightcraft.exe" | Format-Table Name, Length, LastWriteTime
