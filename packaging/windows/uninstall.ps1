#Requires -Version 5.1
<#
.SYNOPSIS
    Uninstalls design:os studio for the current Windows user.
#>
param(
    [string]$InstallRoot = "$env:LOCALAPPDATA\Programs\designos"
)

$ErrorActionPreference = "SilentlyContinue"
$BinDir = Join-Path $InstallRoot "bin"

# Remove from User PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath) {
    $Filtered = ($UserPath -split ";") | Where-Object { $_ -ne "" -and $_ -ne $BinDir -and $_ -ne $InstallRoot }
    [Environment]::SetEnvironmentVariable("Path", ($Filtered -join ";"), "User")
}

# Remove shortcuts
$StartMenuDir = Join-Path ([Environment]::GetFolderPath("Programs")) "designos"
Remove-Item -Recurse -Force $StartMenuDir
$DesktopShortcut = Join-Path ([Environment]::GetFolderPath("Desktop")) "designos studio.lnk"
Remove-Item -Force $DesktopShortcut

# Remove registry keys
Remove-Item -Recurse -Force "HKCU:\Software\Classes\.designos"
Remove-Item -Recurse -Force "HKCU:\Software\Classes\designos.Document"
Remove-Item -Recurse -Force "HKCU:\Software\Classes\.fig"
Remove-Item -Recurse -Force "HKCU:\Software\Classes\designos.FigmaDocument"
Remove-Item -Recurse -Force "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\designos"
Remove-Item -Recurse -Force "HKCU:\Software\designos"

# Remove installed files
if (Test-Path $InstallRoot) {
    Remove-Item -Recurse -Force $InstallRoot
}

Write-Host "design:os studio has been uninstalled."
