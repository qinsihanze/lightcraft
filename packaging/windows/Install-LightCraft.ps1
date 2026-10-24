# Helper script to create Desktop and Start Menu shortcuts for LightCraft
$target = Join-Path $PSScriptRoot "lightcraft.exe"
if (-not (Test-Path $target)) {
    Write-Error "lightcraft.exe not found in $PSScriptRoot"
    exit 1
}

$wsh = New-Object -ComObject WScript.Shell

# Desktop shortcut
$desktop = [Environment]::GetFolderPath("Desktop")
$desktopShortcut = $wsh.CreateShortcut((Join-Path $desktop "LightCraft.lnk"))
$desktopShortcut.TargetPath = $target
$desktopShortcut.WorkingDirectory = $PSScriptRoot
$desktopShortcut.Description = "LightCraft Photo Library and Raw Developer"
$desktopShortcut.IconLocation = "$target,0"
$desktopShortcut.Save()

# Start menu shortcut
$programs = [Environment]::GetFolderPath("Programs")
$startShortcut = $wsh.CreateShortcut((Join-Path $programs "LightCraft.lnk"))
$startShortcut.TargetPath = $target
$startShortcut.WorkingDirectory = $PSScriptRoot
$startShortcut.Description = "LightCraft Photo Library and Raw Developer"
$startShortcut.IconLocation = "$target,0"
$startShortcut.Save()

Write-Host "LightCraft shortcuts successfully created on Desktop and Start Menu!" -ForegroundColor Green
