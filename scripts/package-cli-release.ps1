param(
    [string]$OutputDirectory = ""
)

$ErrorActionPreference = "Stop"

$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $OutputDirectory = Join-Path $Root "dist/release"
}

New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$OutputDirectory = (Resolve-Path $OutputDirectory).Path

Push-Location $Root
try {
    Write-Host "== verify locked dependency graph =="
    cargo metadata --locked --format-version 1 | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "cargo metadata failed" }

    Write-Host "== build Presolve CLI release binary =="
    cargo build --locked --release -p presolve-cli
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

    $Binary = Join-Path $Root "target/release/presolve.exe"
    if (-not (Test-Path $Binary -PathType Leaf)) {
        throw "release binary was not produced at $Binary"
    }

    $VersionOutput = (& $Binary --version).Trim()
    $Parts = $VersionOutput -split '\s+'
    if ($Parts.Count -ne 2 -or $Parts[0] -ne "presolve" -or [string]::IsNullOrWhiteSpace($Parts[1])) {
        throw "unexpected version output: $VersionOutput"
    }
    $Version = $Parts[1]

    $HostLine = rustc -vV | Select-String '^host: '
    if (-not $HostLine) { throw "could not determine the native Rust target" }
    $Target = ($HostLine.ToString() -replace '^host:\s*', '').Trim()

    $PackageName = "presolve-v$Version-$Target"
    $StagingDirectory = Join-Path $OutputDirectory $PackageName
    $Archive = Join-Path $OutputDirectory "$PackageName.zip"
    $Checksum = "$Archive.sha256"

    Remove-Item $StagingDirectory -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item $Archive, $Checksum -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Path $StagingDirectory -Force | Out-Null

    Copy-Item $Binary (Join-Path $StagingDirectory "presolve.exe")
    Copy-Item (Join-Path $Root "LICENSE") (Join-Path $StagingDirectory "LICENSE")

    & (Join-Path $StagingDirectory "presolve.exe") --version
    if ($LASTEXITCODE -ne 0) { throw "packaged binary verification failed" }

    Compress-Archive -Path $StagingDirectory -DestinationPath $Archive -CompressionLevel Optimal
    Remove-Item $StagingDirectory -Recurse -Force

    $Digest = (Get-FileHash -Path $Archive -Algorithm SHA256).Hash.ToLowerInvariant()
    "$Digest  $([System.IO.Path]::GetFileName($Archive))" | Set-Content -Path $Checksum -Encoding utf8NoBOM

    $ChecksumParts = (Get-Content $Checksum -Raw).Trim() -split '\s+', 2
    if ($ChecksumParts[0] -ne $Digest -or $ChecksumParts[1] -ne [System.IO.Path]::GetFileName($Archive)) {
        throw "release archive checksum verification failed"
    }

    Write-Host "release package: $Archive"
    Write-Host "checksum:        $Checksum"
    Write-Host "version:         $Version"
    Write-Host "target:          $Target"

    if ($env:GITHUB_OUTPUT) {
        Add-Content -Path $env:GITHUB_OUTPUT -Value "archive=$Archive"
        Add-Content -Path $env:GITHUB_OUTPUT -Value "checksum=$Checksum"
        Add-Content -Path $env:GITHUB_OUTPUT -Value "version=$Version"
        Add-Content -Path $env:GITHUB_OUTPUT -Value "target=$Target"
        Add-Content -Path $env:GITHUB_OUTPUT -Value "package_name=$PackageName"
    }
}
finally {
    Pop-Location
}
