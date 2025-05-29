# Nitroterm Windows Installer
# PowerShell script to install Nitroterm on Windows

param(
    [string]$InstallPath = "$env:LOCALAPPDATA\Nitroterm",
    [switch]$AddToPath = $true,
    [switch]$CreateDesktopShortcut = $false,
    [switch]$Force = $false
)

$ErrorActionPreference = "Stop"

Write-Host @"
    ███╗   ██╗██╗████████╗██████╗  ██████╗ ██╗  ██╗██╗████████╗
    ████╗  ██║██║╚══██╔══╝██╔══██╗██╔═══██╗██║ ██╔╝██║╚══██╔══╝
    ██╔██╗ ██║██║   ██║   ██████╔╝██║   ██║█████╔╝ ██║   ██║
    ██║╚██╗██║██║   ██║   ██╔══██╗██║   ██║██╔═██╗ ██║   ██║
    ██║ ╚████║██║   ██║   ██║  ██║╚██████╔╝██║  ██╗██║   ██║
    ╚═╝  ╚═══╝╚═╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝   ╚═╝

    🚀 Nitroterm Windows Installer
    A terminal tool for project management and automation

"@ -ForegroundColor Cyan

Write-Host "Starting Nitroterm installation..." -ForegroundColor Green
Write-Host "Installation directory: $InstallPath" -ForegroundColor Yellow

# Check if running as administrator for system-wide installation
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")

if (-not $isAdmin -and $InstallPath.StartsWith($env:ProgramFiles)) {
    Write-Host "⚠️  Administrator privileges required for system-wide installation." -ForegroundColor Yellow
    Write-Host "Installing to user directory instead: $env:LOCALAPPDATA\Nitroterm" -ForegroundColor Yellow
    $InstallPath = "$env:LOCALAPPDATA\Nitroterm"
}

# Create installation directory
if (Test-Path $InstallPath) {
    if (-not $Force) {
        $response = Read-Host "Installation directory already exists. Overwrite? (y/N)"
        if ($response -ne 'y' -and $response -ne 'Y') {
            Write-Host "Installation cancelled." -ForegroundColor Red
            exit 1
        }
    }
    Remove-Item -Path $InstallPath -Recurse -Force
}

Write-Host "📁 Creating installation directory..." -ForegroundColor Blue
New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null

# Check if Rust is installed
Write-Host "🔍 Checking for Rust installation..." -ForegroundColor Blue
try {
    $rustVersion = cargo --version 2>$null
    Write-Host "✅ Rust found: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Rust not found. Installing Rust..." -ForegroundColor Red

    # Download and install Rust
    Write-Host "📥 Downloading Rust installer..." -ForegroundColor Blue
    $rustupUrl = "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
    $rustupPath = "$env:TEMP\rustup-init.exe"

    Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath

    Write-Host "🔧 Installing Rust (this may take a few minutes)..." -ForegroundColor Blue
    Start-Process -FilePath $rustupPath -ArgumentList "-y" -Wait

    # Refresh environment
    $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")

    Write-Host "✅ Rust installation completed!" -ForegroundColor Green
}

# Check for Git
Write-Host "🔍 Checking for Git installation..." -ForegroundColor Blue
try {
    $gitVersion = git --version 2>$null
    Write-Host "✅ Git found: $gitVersion" -ForegroundColor Green
} catch {
    Write-Host "⚠️  Git not found. Please install Git from https://git-scm.com/download/win" -ForegroundColor Yellow
    Write-Host "Continuing installation without Git (some features may not work)..." -ForegroundColor Yellow
}

# Download or build Nitroterm
$buildFromSource = $true
$nitrotermExe = "$InstallPath\nitroterm.exe"

if ($buildFromSource) {
    Write-Host "🏗️  Building Nitroterm from source..." -ForegroundColor Blue

    # Clone repository
    $tempDir = "$env:TEMP\nitroterm-build"
    if (Test-Path $tempDir) {
        Remove-Item -Path $tempDir -Recurse -Force
    }

    Write-Host "📥 Cloning repository..." -ForegroundColor Blue
    git clone https://github.com/mustafagenc/nitroterm.git $tempDir

    # Build project
    Write-Host "🔨 Compiling Nitroterm..." -ForegroundColor Blue
    Set-Location "$tempDir\nitroterm"
    cargo build --release

    # Copy executable
    Copy-Item "target\release\nitroterm.exe" $nitrotermExe

    # Cleanup
    Set-Location $PSScriptRoot
    Remove-Item -Path $tempDir -Recurse -Force
} else {
    # In future, download pre-built binary
    Write-Host "📥 Downloading Nitroterm binary..." -ForegroundColor Blue
    # Invoke-WebRequest -Uri "https://github.com/mustafagenc/nitroterm/releases/latest/download/nitroterm-windows.exe" -OutFile $nitrotermExe
}

# Verify installation
if (Test-Path $nitrotermExe) {
    Write-Host "✅ Nitroterm binary installed successfully!" -ForegroundColor Green
} else {
    Write-Host "❌ Failed to install Nitroterm binary!" -ForegroundColor Red
    exit 1
}

# Add to PATH
if ($AddToPath) {
    Write-Host "🔧 Adding Nitroterm to PATH..." -ForegroundColor Blue

    $userPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
    if ($userPath -notlike "*$InstallPath*") {
        $newPath = "$userPath;$InstallPath"
        [System.Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
        Write-Host "✅ Added to PATH! (restart your terminal to use 'nitroterm' command)" -ForegroundColor Green
    } else {
        Write-Host "✅ Already in PATH!" -ForegroundColor Green
    }
}

# Create desktop shortcut
if ($CreateDesktopShortcut) {
    Write-Host "🖥️  Creating desktop shortcut..." -ForegroundColor Blue

    $WshShell = New-Object -comObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut("$env:USERPROFILE\Desktop\Nitroterm.lnk")
    $Shortcut.TargetPath = "powershell.exe"
    $Shortcut.Arguments = "-Command `"& '$nitrotermExe' -i`""
    $Shortcut.WorkingDirectory = $env:USERPROFILE
    $Shortcut.IconLocation = "$nitrotermExe"
    $Shortcut.Description = "Nitroterm - Project Management Tool"
    $Shortcut.Save()

    Write-Host "✅ Desktop shortcut created!" -ForegroundColor Green
}

# Create Start Menu shortcut
Write-Host "📂 Creating Start Menu shortcut..." -ForegroundColor Blue
$startMenuPath = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
$WshShell = New-Object -comObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut("$startMenuPath\Nitroterm.lnk")
$Shortcut.TargetPath = "powershell.exe"
$Shortcut.Arguments = "-Command `"& '$nitrotermExe' -i`""
$Shortcut.WorkingDirectory = $env:USERPROFILE
$Shortcut.IconLocation = "$nitrotermExe"
$Shortcut.Description = "Nitroterm - Project Management Tool"
$Shortcut.Save()

Write-Host "✅ Start Menu shortcut created!" -ForegroundColor Green

# Installation complete
Write-Host @"

🎉 Installation completed successfully!

📍 Installation location: $InstallPath
🚀 Usage:
   • Command line: nitroterm
   • Interactive mode: nitroterm -i
   • Generate release notes: nitroterm release-notes
   • Update dependencies: nitroterm update-dependencies

📚 Documentation: https://github.com/mustafagenc/nitroterm
🐛 Issues: https://github.com/mustafagenc/nitroterm/issues

"@ -ForegroundColor Green

if ($AddToPath) {
    Write-Host "💡 Don't forget to restart your terminal to use the 'nitroterm' command!" -ForegroundColor Yellow
}

Write-Host "Press any key to exit..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
