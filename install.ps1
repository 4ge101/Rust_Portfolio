# Downloads the prebuilt portfolio.exe from GitHub Releases and puts it on
# PATH. No git clone, no Rust toolchain required.
#
# Usage (PowerShell):
#   irm https://raw.githubusercontent.com/4ge101/Rust_Portfolio/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "4ge101/Rust_Portfolio"
$InstallDir = "$env:LOCALAPPDATA\portfolio\bin"
$Target = "x86_64-pc-windows-msvc"
$Url = "https://github.com/$Repo/releases/latest/download/portfolio-$Target.zip"

Write-Host "Downloading portfolio for $Target ..."

$tmp = New-Item -ItemType Directory -Path ([System.IO.Path]::GetTempPath()) -Name ([System.Guid]::NewGuid())
$zipPath = Join-Path $tmp "portfolio.zip"

Invoke-WebRequest -Uri $Url -OutFile $zipPath
Expand-Archive -Path $zipPath -DestinationPath $tmp -Force

New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
Move-Item -Path (Join-Path $tmp "portfolio.exe") -Destination (Join-Path $InstallDir "portfolio.exe") -Force
Remove-Item -Recurse -Force $tmp

Write-Host "Installed to $InstallDir\portfolio.exe"

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
    Write-Host ""
    Write-Host "Added $InstallDir to your user PATH."
    Write-Host "Open a new terminal window for this to take effect."
}

Write-Host ""
Write-Host "Run it with: portfolio"