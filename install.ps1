#requires -Version 5.1
<#
.SYNOPSIS
Install a verified Twig release for the current Windows user. No admin required.
.EXAMPLE
.\install.ps1 -Version v3.1.0 -InstallDir "$env:LOCALAPPDATA\Programs\Twig\bin"
.EXAMPLE
.\install.ps1 -NoPath
#>
[CmdletBinding()]
param(
    [string]$Version = 'latest',
    [string]$InstallDir = '',
    [switch]$NoPath,
    [switch]$Help
)

function Receive-TwigFile([string]$Uri, [string]$Destination) {
    Invoke-WebRequest -UseBasicParsing -Uri $Uri -OutFile $Destination -TimeoutSec 120 -ErrorAction Stop
}
function Get-TwigRelease {
    (Invoke-RestMethod -Uri 'https://api.github.com/repos/workdone0/twig/releases/latest' -TimeoutSec 30 -ErrorAction Stop).tag_name
}
function Get-TwigArchitecture {
    if ($env:PROCESSOR_ARCHITEW6432) { return $env:PROCESSOR_ARCHITEW6432 }
    return $env:PROCESSOR_ARCHITECTURE
}
function Get-TwigUserPath { [Environment]::GetEnvironmentVariable('Path', 'User') }
function Set-TwigUserPath([string]$Value) { [Environment]::SetEnvironmentVariable('Path', $Value, 'User') }
function Add-TwigPath([string]$Directory) {
    $userPath = Get-TwigUserPath
    $entries = @($userPath -split ';' | Where-Object { $_ })
    $exists = @($entries | Where-Object { [Environment]::ExpandEnvironmentVariables($_).TrimEnd('\') -ieq $Directory.TrimEnd('\') }).Count -gt 0
    if (-not $exists) { Set-TwigUserPath (($entries + $Directory) -join ';') }
    if (-not (@($env:Path -split ';') -icontains $Directory)) { $env:Path = "$Directory;$env:Path" }
}
function Install-Twig {
    [CmdletBinding()]
    param([string]$Version = 'latest', [string]$InstallDir = '', [switch]$NoPath, [switch]$Help)
    $ErrorActionPreference = 'Stop'
    if ($Help) {
        Write-Output @'
Twig Windows installer (PowerShell 5.1+)
  -Version TAG       Release tag; default: latest published release
  -InstallDir DIR    Default: %LOCALAPPDATA%\Programs\Twig\bin
  -NoPath            Do not update the current user's PATH
  -Help              Show this help
Supports Windows x64; requires tar.exe and HTTPS access to GitHub.
Verifies SHA-256 before extraction. Close Twig before upgrading.
'@
        return
    }
    if ($env:OS -ne 'Windows_NT') { throw 'Use install.sh on Linux or macOS.' }
    if ((Get-TwigArchitecture) -ne 'AMD64') { throw 'This release provides a Windows x64 binary only. Windows ARM64 and 32-bit installs are not supported by this installer.' }
    if (-not (Get-Command tar.exe -ErrorAction SilentlyContinue)) { throw 'tar.exe is required. Use Windows 10 version 1803 or newer, or Windows 11, with the built-in tar tool available on PATH.' }
    if (-not $InstallDir) { $InstallDir = Join-Path $env:LOCALAPPDATA 'Programs\Twig\bin' }
    $InstallDir = [IO.Path]::GetFullPath($InstallDir)
    if ($InstallDir.Contains(';')) { throw 'InstallDir must not contain a semicolon (a Windows PATH separator).' }
    $oldTls = [Net.ServicePointManager]::SecurityProtocol
    $temp = $null
    $staged = $null
    try {
        [Net.ServicePointManager]::SecurityProtocol = $oldTls -bor [Net.SecurityProtocolType]::Tls12
        if ($Version -eq 'latest') { $Version = Get-TwigRelease }
        if ($Version -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+([-+][A-Za-z0-9.-]+)?$') { throw "Invalid release tag: $Version" }
        $temp = Join-Path ([IO.Path]::GetTempPath()) ('twig-install-' + [Guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory -Path $temp | Out-Null
        $asset = 'twig-x86_64-pc-windows-msvc.tar.gz'
        $archive = Join-Path $temp $asset
        $checksum = Join-Path $temp 'checksum'
        $url = "https://github.com/workdone0/twig/releases/download/$Version/$asset"
        Write-Host "Downloading Twig $Version (Windows x64)..."
        Receive-TwigFile $url $archive
        Receive-TwigFile "$url.sha256" $checksum
        $expected = ((Get-Content -LiteralPath $checksum -Raw).Trim() -split '\s+')[0]
        if ($expected -notmatch '^[a-fA-F0-9]{64}$') { throw 'Invalid checksum file; refusing to install.' }
        if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash -ine $expected) { throw 'Checksum mismatch; refusing to install.' }
        & tar.exe -xzf $archive -C $temp twig.exe LICENSE
        if ($LASTEXITCODE -ne 0) { throw 'Could not extract the verified archive.' }
        $binary = Join-Path $temp 'twig.exe'
        if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) { throw 'Archive did not contain twig.exe.' }
        $reported = & $binary --version
        if ($LASTEXITCODE -ne 0 -or "$reported".Trim() -ne "twig $($Version.Substring(1))") { throw 'The downloaded executable did not report the requested version.' }
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        $target = Join-Path $InstallDir 'twig.exe'
        $staged = Join-Path $InstallDir ('.twig-' + [Guid]::NewGuid().ToString('N') + '.exe')
        Copy-Item -LiteralPath $binary -Destination $staged
        # Keep the existing installation intact if download/verification or replacement fails.
        try {
            if ([IO.File]::Exists($target)) { [IO.File]::Replace($staged, $target, [NullString]::Value) }
            else { [IO.File]::Move($staged, $target) }
        } catch { throw "Could not replace '$target'. Close any running Twig process and check directory permissions. $($_.Exception.Message)" }
        $staged = $null
        Copy-Item -LiteralPath (Join-Path $temp 'LICENSE') -Destination (Join-Path $InstallDir 'LICENSE') -Force
        if (-not $NoPath) {
            try { Add-TwigPath $InstallDir }
            catch { Write-Warning "Twig is installed, but PATH could not be updated. Add '$InstallDir' to your user PATH manually." }
        }
        Write-Host "Installed Twig $Version at $target"
        if ($NoPath) { Write-Host "Run with: & `"$target`" --version" }
        else { Write-Host 'Close and reopen your terminal, then run: twig --version' }
    } finally {
        [Net.ServicePointManager]::SecurityProtocol = $oldTls
        if ($staged -and (Test-Path -LiteralPath $staged)) { Remove-Item -LiteralPath $staged -Force -ErrorAction SilentlyContinue }
        if ($temp -and (Test-Path -LiteralPath $temp)) { Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue }
    }
}

# Dot-sourcing exposes the functions for isolated offline tests, without installing.
if ($MyInvocation.InvocationName -ne '.') {
    try { Install-Twig @PSBoundParameters }
    catch { Write-Error "Twig installation failed: $($_.Exception.Message)"; exit 1 }
}
