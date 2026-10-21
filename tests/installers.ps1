# Exercise the Windows installer against the freshly packaged native binaries.
[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$Archive)

$ErrorActionPreference = 'Stop'
$env:DOWNTUBE_TEST_ARCHIVE = (Resolve-Path -LiteralPath $Archive).Path
$metadata = cargo metadata --locked --no-deps --format-version 1 | ConvertFrom-Json
$env:DOWNTUBE_TEST_TAG = 'v' + ($metadata.packages | Where-Object { $_.name -eq 'downtube' }).version
$env:DOWNTUBE_TEST_BAD_HASH = '0'
$scratch = Join-Path ([IO.Path]::GetTempPath()) ('downtube-installer-test-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $scratch | Out-Null
$installer = Join-Path $PSScriptRoot '..\install.ps1'

# Functions in this script scope shadow network requests made by the installer.
function Invoke-RestMethod {
    param([string]$Uri, [hashtable]$Headers)
    if ($Uri -ne 'https://api.github.com/repos/Nightmare99/downtube/releases/latest') { throw "Unexpected metadata URL: $Uri" }
    return @{ tag_name = $env:DOWNTUBE_TEST_TAG }
}

function Invoke-WebRequest {
    param([string]$Uri, [hashtable]$Headers, [string]$OutFile, [switch]$UseBasicParsing)
    $name = [IO.Path]::GetFileName($env:DOWNTUBE_TEST_ARCHIVE)
    $base = "https://github.com/Nightmare99/downtube/releases/download/$env:DOWNTUBE_TEST_TAG"
    if ($Uri -eq "$base/$name") {
        Copy-Item -LiteralPath $env:DOWNTUBE_TEST_ARCHIVE -Destination $OutFile
    } elseif ($Uri -eq "$base/SHA256SUMS") {
        $hash = (Get-FileHash -LiteralPath $env:DOWNTUBE_TEST_ARCHIVE -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($env:DOWNTUBE_TEST_BAD_HASH -eq '1') { $hash = '0' * 64 }
        [IO.File]::WriteAllText($OutFile, "$hash  $name`n")
    } else {
        throw "Unexpected asset URL: $Uri"
    }
}

try {
    $destination = Join-Path $scratch 'install with spaces'
    foreach ($version in @('latest', $env:DOWNTUBE_TEST_TAG.Substring(1))) {
        & $installer -Version $version -InstallDir $destination -NoModifyPath
        foreach ($binary in @('downtube.exe', 'dtube.exe')) {
            $actual = & (Join-Path $destination $binary) --version
            if ($LASTEXITCODE -ne 0 -or $actual -ne "downtube $($env:DOWNTUBE_TEST_TAG.Substring(1))") { throw 'Installed binary has the wrong version.' }
        }
    }
    $existing = Join-Path $destination 'downtube.exe'
    $before = (Get-FileHash -LiteralPath $existing -Algorithm SHA256).Hash
    $env:DOWNTUBE_TEST_BAD_HASH = '1'
    try {
        & $installer -Version $env:DOWNTUBE_TEST_TAG -InstallDir $destination -NoModifyPath
        throw 'Expected checksum verification to reject the download.'
    } catch {
        if ($_.Exception.Message -notlike '*Checksum mismatch*') { throw }
    }
    if ((Get-FileHash -LiteralPath $existing -Algorithm SHA256).Hash -ne $before) { throw 'Checksum failure changed an existing installation.' }
    Write-Host 'PASS: Windows latest/pinned installs, upgrade, paths with spaces, both commands, and checksum rejection.'
} finally {
    Remove-Item -LiteralPath $scratch -Recurse -Force -ErrorAction SilentlyContinue
}
