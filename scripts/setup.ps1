# gh_cherry Windows setup script
# - Installs gh_cherry.exe to a per-user directory
# - Adds that directory to the user's PATH for global execution
#
# Usage (from the extracted release folder):
#   powershell -ExecutionPolicy Bypass -File .\scripts\setup.ps1

param(
  [ValidateSet('User','Machine')]
  [string] $Scope = 'User',
  [string] $InstallDir,
  [switch] $Force,
  [switch] $NoPathUpdate
)

$ErrorActionPreference = 'Stop'

function Write-Info($msg) { Write-Host "[INFO] $msg" -ForegroundColor Cyan }
function Write-Warn($msg) { Write-Host "[WARN] $msg" -ForegroundColor Yellow }
function Write-Err($msg)  { Write-Host "[ERROR] $msg" -ForegroundColor Red }

try {
  $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
  $sourceExe = Join-Path $scriptDir '..\gh_cherry.exe' | Resolve-Path -ErrorAction Stop
} catch {
  Write-Err "Could not locate gh_cherry.exe next to the setup script. Ensure the executable and scripts folder are together."
  exit 1
}

# Derive default InstallDir based on scope if not provided
if (-not $PSBoundParameters.ContainsKey('InstallDir')) {
  if ($Scope -eq 'Machine') {
    $InstallDir = Join-Path $env:ProgramFiles 'gh_cherry'
  } else {
    $InstallDir = Join-Path $env:USERPROFILE 'gh_cherry'
  }
}

# If machine scope, ensure we're running as Administrator
if ($Scope -eq 'Machine') {
  $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltinRole]::Administrator)
  if (-not $isAdmin) {
    Write-Err "Machine scope requires an elevated PowerShell. Right-click PowerShell and 'Run as Administrator', then re-run with -Scope Machine."
    exit 1
  }
}

Write-Info "Installing gh_cherry from: $($sourceExe.Path)"
Write-Info "Target install directory: $InstallDir"

# Ensure install directory exists
if (-not (Test-Path -LiteralPath $InstallDir)) {
  New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$destExe = Join-Path $InstallDir 'gh_cherry.exe'

# Copy (overwrite if -Force or existing)
if ((Test-Path -LiteralPath $destExe) -and -not $Force) {
  Write-Warn "gh_cherry.exe already exists at destination. Use -Force to overwrite. Skipping copy."
} else {
  Copy-Item -LiteralPath $sourceExe -Destination $destExe -Force
  Write-Info "Copied to: $destExe"
}

if (-not $NoPathUpdate) {
  if ($Scope -eq 'Machine') {
    $currentPath = [Environment]::GetEnvironmentVariable('Path', 'Machine')
    $parts = @()
    if ($null -ne $currentPath -and $currentPath.Trim() -ne '') { $parts = $currentPath -split ';' }
    $already = $parts | Where-Object { $_.Trim().ToLower() -eq $InstallDir.Trim().ToLower() }
    if (-not $already) {
      $newPath = if ($null -ne $currentPath -and $currentPath.Trim() -ne '') { "$currentPath;$InstallDir" } else { $InstallDir }
      $envKey = 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Environment'
      Set-ItemProperty -Path $envKey -Name Path -Value $newPath
      # Update current session PATH for immediate availability (best-effort)
      if (-not ($env:Path.ToLower().Split(';') -contains $InstallDir.ToLower())) { $env:Path = "$env:Path;$InstallDir" }
    } else {
      Write-Info "Install directory already present in Machine PATH."
    }
  } else {
    $currentUserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $pathParts = @()
    if ($null -ne $currentUserPath -and $currentUserPath.Trim() -ne '') { $pathParts = $currentUserPath -split ';' }
    $alreadyPresent = $pathParts | Where-Object { $_.Trim().ToLower() -eq $InstallDir.Trim().ToLower() }

    if (-not $alreadyPresent) {
      $newPath = if ($null -ne $currentUserPath -and $currentUserPath.Trim() -ne '') { "$currentUserPath;$InstallDir" } else { $InstallDir }
      $envKey = 'HKCU:\Environment'
      New-Item -Path $envKey -Force | Out-Null
      Set-ItemProperty -Path $envKey -Name Path -Value $newPath
      # Update current session PATH
      $env:Path = $newPath
    } else {
      Write-Info "Install directory already present in User PATH."
    }
  }

  # Broadcast environment change so other processes may pick it up
  $sig = @"
using System;
using System.Runtime.InteropServices;
public static class NativeMethods {
  [DllImport("user32.dll", SetLastError=true, CharSet=CharSet.Auto)]
  public static extern IntPtr SendMessageTimeout(IntPtr hWnd, int Msg, IntPtr wParam, string lParam, int fuFlags, int uTimeout, out IntPtr lpdwResult);
}
"@
  try {
    Add-Type -TypeDefinition $sig -ErrorAction SilentlyContinue | Out-Null
    [void][NativeMethods]::SendMessageTimeout([IntPtr]0xffff, 0x1A, [IntPtr]0, 'Environment', 2, 5000, [ref]([IntPtr]::Zero))
    if ($Scope -eq 'Machine') { Write-Info "Updated Machine PATH and notified the system." } else { Write-Info "Updated User PATH and notified the system." }
  } catch {
    if ($Scope -eq 'Machine') { Write-Warn "Updated Machine PATH. You may need to restart terminals to pick up changes." } else { Write-Warn "Updated User PATH. You may need to restart terminals to pick up changes." }
  }
} else {
  Write-Warn "Skipping PATH update due to -NoPathUpdate."
}

Write-Host "`nInstallation complete." -ForegroundColor Green
Write-Host "Run: gh_cherry" -ForegroundColor Green
Write-Host "If 'gh_cherry' is not recognized in existing terminals, restart them." -ForegroundColor Yellow
