param(
    [string]$Version = $(if ($env:PRESOLVE_VERSION) { $env:PRESOLVE_VERSION } else { "0.1.0-alpha.1" }),
    [string]$InstallDir = $(if ($env:PRESOLVE_INSTALL_DIR) { $env:PRESOLVE_INSTALL_DIR } else { Join-Path $HOME ".presolve\bin" })
)

$ErrorActionPreference = "Stop"
$Repository = "fierstdev/presolve"
$Tag = "v$Version"

$Architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
if ($Architecture -ne "X64") {
    throw "Unsupported Windows architecture: $Architecture. Presolve currently publishes Windows x86_64 binaries."
}

$Target = "x86_64-pc-windows-msvc"
$Package = "presolve-v$Version-$Target"
$Archive = "$Package.zip"
$BaseUrl = "https://github.com/$Repository/releases/download/$Tag"

$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("presolve-install-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

try {
    Write-Host "Installing Presolve $Version for $Target..."
    $ArchivePath = Join-Path $TempDir $Archive
    $ChecksumPath = "$ArchivePath.sha256"

    Invoke-WebRequest -Uri "$BaseUrl/$Archive" -OutFile $ArchivePath -UseBasicParsing
    Invoke-WebRequest -Uri "$BaseUrl/$Archive.sha256" -OutFile $ChecksumPath -UseBasicParsing

    $ChecksumParts = (Get-Content $ChecksumPath -Raw).Trim() -split '\s+', 2
    if ($ChecksumParts.Count -ne 2) {
        throw "Invalid release checksum file"
    }

    $Expected = $ChecksumParts[0].ToLowerInvariant()
    $Actual = (Get-FileHash -Path $ArchivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($Expected -ne $Actual) {
        throw "Release checksum verification failed"
    }

    $ExtractDir = Join-Path $TempDir "extract"
    Expand-Archive -Path $ArchivePath -DestinationPath $ExtractDir -Force
    $Binary = Join-Path $ExtractDir "$Package\presolve.exe"
    if (-not (Test-Path $Binary -PathType Leaf)) {
        throw "Release archive does not contain presolve.exe"
    }

    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    $InstalledBinary = Join-Path $InstallDir "presolve.exe"
    Copy-Item $Binary $InstalledBinary -Force

    & $InstalledBinary --version | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Installed Presolve binary failed verification"
    }

    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $PathEntries = @($UserPath -split ';' | Where-Object { $_ })
    if ($PathEntries -notcontains $InstallDir) {
        $NewUserPath = if ([string]::IsNullOrWhiteSpace($UserPath)) { $InstallDir } else { "$UserPath;$InstallDir" }
        [Environment]::SetEnvironmentVariable("Path", $NewUserPath, "User")
        Write-Host "Added $InstallDir to your user PATH."
    }

    if (($env:Path -split ';') -notcontains $InstallDir) {
        $env:Path = "$InstallDir;$env:Path"
    }

    Write-Host "Installed $InstalledBinary"
    Write-Host "Run 'presolve --version' to verify the installation. New terminals will inherit the updated PATH."
}
finally {
    Remove-Item $TempDir -Recurse -Force -ErrorAction SilentlyContinue
}
