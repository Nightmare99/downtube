# Install published Windows binaries. Compatible with Windows PowerShell 5.1+.
[CmdletBinding()]
param(
    [string]$Version = $(if ($env:DOWNTUBE_VERSION) { $env:DOWNTUBE_VERSION } else { 'latest' }),
    [string]$InstallDir = $(if ($env:DOWNTUBE_INSTALL_DIR) { $env:DOWNTUBE_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'Programs\downtube\bin' }),
    [switch]$NoModifyPath
)

$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') { throw 'Use install.sh on Linux and macOS.' }
if ([string]::IsNullOrWhiteSpace($InstallDir)) { throw 'Installation directory cannot be empty.' }
$InstallDir = [IO.Path]::GetFullPath($InstallDir)
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
$repo = 'https://github.com/Nightmare99/downtube'
$headers = @{ 'User-Agent' = 'downtube-installer' }
$architecture = if ($env:PROCESSOR_ARCHITEW6432) { $env:PROCESSOR_ARCHITEW6432 } else { $env:PROCESSOR_ARCHITECTURE }
$target = switch ($architecture) {
    'AMD64' { 'x86_64-pc-windows-msvc' }
    'ARM64' { 'aarch64-pc-windows-msvc' }
    default { throw "Unsupported CPU architecture: $architecture (need x64 or ARM64)." }
}
if ($Version -eq 'latest') {
    $release = Invoke-RestMethod -Uri 'https://api.github.com/repos/Nightmare99/downtube/releases/latest' -Headers $headers
    $Version = $release.tag_name
}
if (-not $Version.StartsWith('v')) { $Version = "v$Version" }
if ($Version -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$') { throw 'Invalid version; expected vX.Y.Z or vX.Y.Z-prerelease.' }
$asset = "downtube-$Version-$target.zip"
$scratch = Join-Path ([IO.Path]::GetTempPath()) ("downtube-install-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $scratch | Out-Null
try {
    Write-Host "Downloading downtube $Version ($target)..."
    $archive = Join-Path $scratch $asset
    $manifest = Join-Path $scratch 'SHA256SUMS'
    Invoke-WebRequest -UseBasicParsing -Uri "$repo/releases/download/$Version/$asset" -Headers $headers -OutFile $archive
    Invoke-WebRequest -UseBasicParsing -Uri "$repo/releases/download/$Version/SHA256SUMS" -Headers $headers -OutFile $manifest
    $matchesForAsset = @(Get-Content -LiteralPath $manifest | Where-Object { $_ -match ('^[0-9a-f]{64}  ' + [regex]::Escape($asset) + '$') })
    if ($matchesForAsset.Count -ne 1) { throw 'Release checksum missing or malformed.' }
    $expected = $matchesForAsset[0].Substring(0, 64)
    $actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) { throw 'Checksum mismatch; nothing was installed.' }
    $unpacked = Join-Path $scratch 'unpacked'
    Expand-Archive -LiteralPath $archive -DestinationPath $unpacked
    foreach ($binary in @('downtube.exe', 'dtube.exe')) {
        if (-not (Test-Path -LiteralPath (Join-Path $unpacked $binary) -PathType Leaf)) { throw "Release is missing $binary." }
    }
    $reported = & (Join-Path $unpacked 'downtube.exe') --version
    if ($LASTEXITCODE -ne 0 -or $reported -ne "downtube $($Version.Substring(1))") { throw 'Downloaded binary could not run or has an unexpected version.' }
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    foreach ($binary in @('downtube.exe', 'dtube.exe')) {
        Copy-Item -LiteralPath (Join-Path $unpacked $binary) -Destination (Join-Path $InstallDir $binary) -Force
    }
    if (-not $NoModifyPath) {
        $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
        $entries = @($userPath -split ';' | Where-Object { $_ })
        if ($entries -notcontains $InstallDir) {
            [Environment]::SetEnvironmentVariable('Path', (($entries + $InstallDir) -join ';'), 'User')
        }
        if (($env:Path -split ';') -notcontains $InstallDir) { $env:Path = "$InstallDir;$env:Path" }
    }
    Write-Host "Installed $reported and dtube to $InstallDir"
    if ($NoModifyPath) { Write-Host 'PATH was not changed.' }
    if (-not (Get-Command yt-dlp -ErrorAction SilentlyContinue) -or -not (Get-Command ffmpeg -ErrorAction SilentlyContinue) -or -not (Get-Command ffprobe -ErrorAction SilentlyContinue)) {
        Write-Host 'Downloads also need yt-dlp, FFmpeg (including ffprobe), and Deno or Node.'
        Write-Host 'Dependency setup: https://github.com/Nightmare99/downtube/blob/main/docs/installation.md'
    }
    Write-Host 'Try: dtube --help'
} finally {
    Remove-Item -LiteralPath $scratch -Recurse -Force -ErrorAction SilentlyContinue
}
