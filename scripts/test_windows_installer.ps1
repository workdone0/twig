# Offline contracts, run under both Windows PowerShell 5.1 and PowerShell 7.
param([Parameter(Mandatory=$true)][string]$Binary)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\install.ps1"
$root = Join-Path ([IO.Path]::GetTempPath()) ('twig-installer-test-' + [Guid]::NewGuid().ToString('N'))
$originalPath = $env:Path
$originalTemp = $env:TEMP
$originalTmp = $env:TMP
function Assert($Condition, [string]$Message) { if (-not $Condition) { throw $Message } }
function Expect-Failure([scriptblock]$Action, [string]$Text) {
    $caught = $false
    try { & $Action } catch { $caught = $true; Assert ($_.Exception.Message -like "*$Text*") "Unexpected error: $_" }
    Assert $caught "Expected failure: $Text"
}
try {
    New-Item -ItemType Directory -Path $root | Out-Null
    $env:TEMP = $root; $env:TMP = $root
    $fixture = Join-Path $root 'fixture'; New-Item -ItemType Directory -Path $fixture | Out-Null
    Copy-Item -LiteralPath $Binary -Destination (Join-Path $fixture 'twig.exe')
    Copy-Item -LiteralPath "$PSScriptRoot\..\LICENSE" -Destination $fixture
    $script:releaseVersion = 'v' + ((& $Binary --version) -replace '^twig ', '')
    $script:fixtureArchive = Join-Path $root 'archive.tar.gz'
    & tar.exe -czf $script:fixtureArchive -C $fixture twig.exe LICENSE
    Assert ($LASTEXITCODE -eq 0) 'Fixture archive failed'
    $script:hash = (Get-FileHash -LiteralPath $script:fixtureArchive -Algorithm SHA256).Hash
    $script:badHash = $false; $script:missingHash = $false; $script:arch = 'AMD64'; $script:userPath = 'C:\existing'
    function Get-TwigRelease { $script:releaseVersion }
    function Get-TwigArchitecture { $script:arch }
    function Get-TwigUserPath { $script:userPath }
    function Set-TwigUserPath([string]$Value) { $script:userPath = $Value }
    function Receive-TwigFile([string]$Uri, [string]$Destination) {
        if ($Uri.EndsWith('.sha256')) {
            if ($script:missingHash) { throw 'Missing checksum' }
            $checksum = if ($script:badHash) { '0' * 64 } else { $script:hash }
            Set-Content -LiteralPath $Destination -Value "$checksum  archive.tar.gz" -Encoding ASCII
        } else { Copy-Item -LiteralPath $script:fixtureArchive -Destination $Destination }
    }
    Assert ((Install-Twig -Help | Out-String) -match 'PowerShell 5.1') 'Help unavailable'
    $destination = Join-Path $root 'directory with spaces'
    Install-Twig -InstallDir $destination
    Assert (Test-Path (Join-Path $destination 'twig.exe')) 'Executable missing'
    Assert ($script:userPath -eq "C:\existing;$destination") 'User PATH was not preserved'
    Install-Twig -InstallDir $destination
    Assert ($script:userPath -eq "C:\existing;$destination") 'PATH duplicated during upgrade'
    Assert ((& (Join-Path $destination 'twig.exe') --version) -eq "twig $($script:releaseVersion.Substring(1))") 'Wrong installed binary'
    $before = (Get-FileHash (Join-Path $destination 'twig.exe')).Hash
    $locked = [IO.File]::Open((Join-Path $destination 'twig.exe'), [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::None)
    try { Expect-Failure { Install-Twig -InstallDir $destination } 'Could not replace' }
    finally { $locked.Dispose() }
    Assert ((Get-FileHash (Join-Path $destination 'twig.exe')).Hash -eq $before) 'Locked install modified existing executable'
    $script:badHash = $true
    Expect-Failure { Install-Twig -InstallDir $destination } 'Checksum mismatch'
    $script:badHash = $false; $script:missingHash = $true
    Expect-Failure { Install-Twig -InstallDir $destination } 'Missing checksum'
    $script:missingHash = $false
    Expect-Failure { Install-Twig -Version 'v9.9.9' -InstallDir $destination } 'requested version'
    Assert ((Get-FileHash (Join-Path $destination 'twig.exe')).Hash -eq $before) 'Failed install modified existing executable'
    Expect-Failure { Install-Twig -Version '../bad' -InstallDir $destination } 'Invalid release tag'
    $script:arch = 'ARM64'
    Expect-Failure { Install-Twig -InstallDir $destination } 'Windows x64'
    $script:arch = 'AMD64'
    $other = Join-Path $root 'without path'
    Install-Twig -InstallDir $other -NoPath
    Assert (-not $script:userPath.Contains($other)) '-NoPath modified user PATH'
    Assert (@(Get-ChildItem -LiteralPath $root -Filter 'twig-install-*').Count -eq 0) 'Temporary downloads leaked'
    Assert (@(Get-ChildItem -LiteralPath $destination -Filter '.twig-*').Count -eq 0) 'Staged executable leaked'
    Write-Host 'Windows installer contracts passed: help, install, upgrade, PATH, checksum failures, wrong version, invalid tag, architecture, cleanup.'
} finally {
    $env:Path = $originalPath; $env:TEMP = $originalTemp; $env:TMP = $originalTmp
    if (Test-Path -LiteralPath $root) { Remove-Item -LiteralPath $root -Recurse -Force }
}
