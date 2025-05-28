# Nitroterm Windows Uninstaller

param(
    [string]$InstallPath = "$env:LOCALAPPDATA\Nitroterm",
    [switch]$RemoveFromPath = $true,
    [switch]$Force = $false
)

Write-Host "🗑️  Nitroterm Uninstaller" -ForegroundColor Red
Write-Host "Removing Nitroterm from: $InstallPath" -ForegroundColor Yellow

if (-not $Force) {
    $response = Read-Host "Are you sure you want to uninstall Nitroterm? (y/N)"
    if ($response -ne 'y' -and $response -ne 'Y') {
        Write-Host "Uninstallation cancelled." -ForegroundColor Green
        exit 0
    }
}

# Remove installation directory
if (Test-Path $InstallPath) {
    Write-Host "📁 Removing installation directory..." -ForegroundColor Blue
    Remove-Item -Path $InstallPath -Recurse -Force
    Write-Host "✅ Installation directory removed!" -ForegroundColor Green
}

# Remove from PATH
if ($RemoveFromPath) {
    Write-Host "🔧 Removing from PATH..." -ForegroundColor Blue
    $userPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
    $newPath = $userPath -replace [regex]::Escape(";$InstallPath"), ""
    $newPath = $newPath -replace [regex]::Escape("$InstallPath;"), ""
    $newPath = $newPath -replace [regex]::Escape("$InstallPath"), ""

    [System.Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "✅ Removed from PATH!" -ForegroundColor Green
}

# Remove shortcuts
$shortcuts = @("$env:USERPROFILE\Desktop\Nitroterm.lnk", "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Nitroterm.lnk")

foreach ($shortcut in $shortcuts) {
    if (Test-Path $shortcut) {
        Remove-Item $shortcut -Force
        Write-Host "✅ Removed shortcut: $(Split-Path $shortcut -Leaf)" -ForegroundColor Green
    }
}

Write-Host "🎉 Nitroterm has been successfully uninstalled!" -ForegroundColor Green
