<#
.SYNOPSIS
    Install / uninstall argmem.

.DESCRIPTION
    1) Gets argmem.exe: uses the one next to this script if present, otherwise
       downloads it from GitHub Releases and verifies its SHA256.
    2) Copies it to %LOCALAPPDATA%\Programs\argmem and adds that folder to the user PATH.
    3) Runs `argmem install` to hook PowerShell profiles and cmd AutoRun.
    (This file is ASCII-only on purpose so that `irm | iex` and PowerShell 5.1 read it correctly.)

.EXAMPLE
    irm https://github.com/schiva/argMem/releases/latest/download/install.ps1 | iex

.EXAMPLE
    & ([scriptblock]::Create((irm https://github.com/schiva/argMem/releases/latest/download/install.ps1))) -Version v0.1.0
    & ([scriptblock]::Create((irm https://github.com/schiva/argMem/releases/latest/download/install.ps1))) -Uninstall
#>
param(
    [string]$Version = 'latest',
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\argmem'),
    [switch]$Uninstall,
    [switch]$NoShellSetup
)

$ErrorActionPreference = 'Stop'
$Repo = 'schiva/argMem'
$Exe = Join-Path $InstallDir 'argmem.exe'

function Write-Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

function Get-UserPath {
    $p = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ($null -eq $p) { return @() }
    return @($p -split ';' | Where-Object { $_ -ne '' })
}

function Set-UserPath([string[]]$parts) {
    # SetEnvironmentVariable broadcasts WM_SETTINGCHANGE, so newly opened windows see the change.
    [Environment]::SetEnvironmentVariable('Path', ($parts -join ';'), 'User')
}

function Test-SamePath($a, $b) {
    return $a.TrimEnd('\') -ieq $b.TrimEnd('\')
}

# ------------------------------------------------------------------ uninstall
if ($Uninstall) {
    if (Test-Path $Exe) {
        Write-Step 'Unhooking PowerShell profile / cmd AutoRun'
        & $Exe uninstall
    }
    $parts = Get-UserPath
    $kept = @($parts | Where-Object { -not (Test-SamePath $_ $InstallDir) })
    if ($kept.Count -ne $parts.Count) {
        Write-Step "Removing from user PATH: $InstallDir"
        Set-UserPath $kept
    }
    if (Test-Path $InstallDir) {
        Write-Step "Deleting $InstallDir"
        Remove-Item -Recurse -Force $InstallDir
    }
    Write-Host ''
    Write-Host "argmem uninstalled. Your rules are kept in $env:USERPROFILE\.argmem" -ForegroundColor Green
    Write-Host "To delete them too: Remove-Item -Recurse `"$env:USERPROFILE\.argmem`""
    return
}

# ------------------------------------------------------------------ install
if (-not [Environment]::Is64BitOperatingSystem) {
    throw 'argmem supports 64-bit Windows only.'
}

$tmp = Join-Path ([IO.Path]::GetTempPath()) ('argmem-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
    $local = if ($PSScriptRoot) { Join-Path $PSScriptRoot 'argmem.exe' } else { $null }
    if ($local -and (Test-Path $local)) {
        Write-Step "Using local argmem.exe: $local"
        $src = $local
    } else {
        [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
        $base = if ($Version -eq 'latest') {
            "https://github.com/$Repo/releases/latest/download"
        } else {
            "https://github.com/$Repo/releases/download/$Version"
        }
        $src = Join-Path $tmp 'argmem.exe'
        $sums = Join-Path $tmp 'SHA256SUMS.txt'
        $old = $ProgressPreference; $ProgressPreference = 'SilentlyContinue'   # progress bar makes 5.1 downloads very slow
        try {
            Write-Step "Downloading $base/argmem.exe"
            Invoke-WebRequest -UseBasicParsing -Uri "$base/argmem.exe" -OutFile $src
            Invoke-WebRequest -UseBasicParsing -Uri "$base/SHA256SUMS.txt" -OutFile $sums
        } finally { $ProgressPreference = $old }

        $line = Get-Content $sums | Where-Object { $_ -match '\s\*?argmem\.exe$' } | Select-Object -First 1
        if (-not $line) { throw 'SHA256SUMS.txt has no entry for argmem.exe' }
        $expected = ($line -split '\s+')[0].ToLower()
        $actual = (Get-FileHash -Algorithm SHA256 $src).Hash.ToLower()
        if ($expected -ne $actual) { throw "SHA256 mismatch: expected $expected, got $actual" }
        Write-Step 'SHA256 verified'
    }

    Write-Step "Copying to $Exe"
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Copy-Item -Force $src $Exe
    Unblock-File -Path $Exe -ErrorAction SilentlyContinue
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}

$parts = Get-UserPath
if (-not ($parts | Where-Object { Test-SamePath $_ $InstallDir })) {
    Write-Step "Adding to user PATH: $InstallDir"
    Set-UserPath (@($parts) + $InstallDir)
}
if (-not (($env:Path -split ';') | Where-Object { $_ -and (Test-SamePath $_ $InstallDir) })) {
    $env:Path = "$env:Path;$InstallDir"
}

& $Exe --version

if (-not $NoShellSetup) {
    Write-Step 'Hooking PowerShell profile / cmd AutoRun'
    & $Exe install
}

Write-Host ''
Write-Host 'Done. Open a new PowerShell / cmd window, then for example:' -ForegroundColor Green
Write-Host '  argmem add claude --dangerously-skip-permissions'
Write-Host '  argmem add --as gl git log --oneline --graph -20'
Write-Host '  argmem list'
