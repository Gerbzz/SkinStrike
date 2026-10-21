#Requires -Version 7.0
[CmdletBinding()]
param(
    [ValidateSet('Info', 'Build', 'Run')][string]$Action = 'Info',
    [switch]$BuildSlotConfirmed,
    [string[]]$AppArgs = @()
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = $PSScriptRoot
$manifestFile = Join-Path $root 'foundation/candidate.json'
$contract = Get-Content -LiteralPath $manifestFile -Raw | ConvertFrom-Json
if ($contract.schema_version -ne 1 -or $contract.source_path -ne 'foundation/subcraft' -or
    $contract.manifest_path -ne 'foundation/subcraft/Cargo.toml' -or
    $contract.upstream_commit -ne '71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5' -or
    $contract.target -ne 'x86_64-pc-windows-msvc' -or
    $contract.package -ne 'subcraft' -or $contract.binary -ne 'subcraft' -or
    (@($contract.features) -join ',') -ne 'skinstrike-candidate') {
    throw 'Candidate contract does not match this entry point.'
}
$source = Join-Path $root $contract.source_path
$cargoManifest = Join-Path $root $contract.manifest_path
$target = Join-Path $root '.candidate/target-subcraft'
$candidateHome = Join-Path $root '.candidate/skinstrike-3d-candidate'
$executable = Join-Path $target 'x86_64-pc-windows-msvc/debug/subcraft.exe'
$receiptFile = Join-Path $root '.candidate/build-receipt.json'
if (-not (Test-Path -LiteralPath $cargoManifest -PathType Leaf)) {
    throw 'Candidate source is missing. Initialize the pinned foundation/subcraft submodule; there is no fallback to the legacy editor.'
}
$sourceHead = (& git -C $source rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0) { throw 'Cannot resolve candidate source commit.' }
& git -C $source merge-base --is-ancestor $contract.upstream_commit HEAD
if ($LASTEXITCODE -ne 0) { throw 'Candidate does not descend from the approved SubCraft 0.1.7 baseline.' }
$sourceDirty = -not [string]::IsNullOrWhiteSpace((& git -C $source status --porcelain=v1 | Out-String))
if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect candidate source state.' }
$treeEntry = (& git -C $root ls-tree HEAD -- 'foundation/subcraft' | Out-String).Trim()
if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect the parent repository foundation pin.' }
$pinnedSource = if ($treeEntry -match '^160000 commit ([0-9a-f]{40})\s') { $Matches[1] } else { $null }
$contractHash = (Get-FileHash -LiteralPath $manifestFile -Algorithm SHA256).Hash
$entryHash = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash

function Assert-NoCandidateRedirect([string]$path) {
    $stateRoot = Join-Path $root '.candidate'
    $cursor = [System.IO.Path]::GetFullPath($path)
    while ($cursor.StartsWith($stateRoot, [StringComparison]::OrdinalIgnoreCase)) {
        if (Test-Path -LiteralPath $cursor) {
            $item = Get-Item -LiteralPath $cursor -Force
            if (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Candidate state path is redirected by a reparse point: $cursor"
            }
        }
        $cursor = [System.IO.Path]::GetDirectoryName($cursor)
    }
}

function Assert-NoCandidateTreeRedirect([string]$path) {
    Assert-NoCandidateRedirect $path
    if (-not (Test-Path -LiteralPath $path -PathType Container)) { return }
    $pendingDirectories = [System.Collections.Generic.Stack[string]]::new()
    $pendingDirectories.Push($path)
    while ($pendingDirectories.Count -gt 0) {
        foreach ($item in Get-ChildItem -LiteralPath $pendingDirectories.Pop() -Force) {
            if (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Candidate state tree contains a redirected path: $($item.FullName)"
            }
            if ($item.PSIsContainer) { $pendingDirectories.Push($item.FullName) }
        }
    }
}

if ($Action -eq 'Info') {
    [pscustomobject]@{
        EntryPoint = $PSCommandPath
        FoundationManifest = $cargoManifest
        UpstreamCommit = $contract.upstream_commit
        CandidateCommit = $sourceHead
        ContractCandidateCommit = $contract.candidate_commit
        ParentPinnedCommit = $pinnedSource
        CandidateSourceDirty = $sourceDirty
        Package = $contract.package
        Feature = 'skinstrike-candidate'
        Target = $contract.target
        Executable = $executable
        ReceiptPresent = Test-Path -LiteralPath $receiptFile -PathType Leaf
        IsolatedHome = $candidateHome
        LegacyManifest = Join-Path $root 'Cargo.toml'
        Status = 'Source candidate; Info does not build, launch, or establish runtime acceptance.'
    }
    return
}

if ($Action -eq 'Build' -and -not $BuildSlotConfirmed) {
    throw 'Build deferred: coordinate release of the reserved CAD build slot, then explicitly pass -BuildSlotConfirmed.'
}
if ($sourceDirty) { throw 'Commit/review candidate source changes before Build or Run so the binary has an immutable source identity.' }
if ($contract.candidate_commit -ne $sourceHead) { throw 'Candidate source does not match the reviewed contract commit.' }
if ($pinnedSource -ne $sourceHead) { throw 'Candidate source does not match the parent repository gitlink. Commit/review the foundation pin before Build or Run.' }
Assert-NoCandidateTreeRedirect $target
Assert-NoCandidateTreeRedirect $candidateHome
Assert-NoCandidateRedirect $executable
Assert-NoCandidateRedirect (Join-Path $candidateHome 'data')
Assert-NoCandidateRedirect (Join-Path $candidateHome 'library')
Assert-NoCandidateRedirect (Join-Path $candidateHome 'ui-state.ron')
Assert-NoCandidateRedirect $receiptFile

if ($Action -eq 'Build') {
    $drive = [System.IO.DriveInfo]::new([System.IO.Path]::GetPathRoot($target))
    if ($drive.AvailableFreeSpace -lt 40GB) {
        throw 'Build requires 40 GiB free to allow build headroom above the 20 GiB reserve.'
    }
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    $cargoPath = if ($cargo) { $cargo.Source } else { Join-Path ([Environment]::GetFolderPath('UserProfile')) '.cargo/bin/cargo.exe' }
    if (-not (Test-Path -LiteralPath $cargoPath -PathType Leaf)) { throw 'Rust/Cargo 1.95 or newer is required for this foundation.' }
    $savedBuildEnvironment = @{}
    foreach ($name in @('CARGO_BUILD_JOBS', 'CARGO_INCREMENTAL', 'CARGO_PROFILE_DEV_DEBUG')) {
        $savedBuildEnvironment[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
    }
    try {
        $env:CARGO_BUILD_JOBS = '1'
        $env:CARGO_INCREMENTAL = '0'
        $env:CARGO_PROFILE_DEV_DEBUG = '0'
        $artifact = $null
        & $cargoPath build --locked --manifest-path $cargoManifest -p subcraft --bin subcraft --features skinstrike-candidate --target x86_64-pc-windows-msvc --target-dir $target -j 1 --message-format=json-render-diagnostics | ForEach-Object {
            $record = $_ | ConvertFrom-Json
            if ($record.reason -eq 'compiler-artifact' -and $record.target.name -eq 'subcraft' -and
                @($record.target.kind) -contains 'bin' -and $record.executable) {
                if ([System.IO.Path]::GetFullPath($record.manifest_path) -ne (Join-Path $source 'apps/subcraft/Cargo.toml') -or
                    [System.IO.Path]::GetFullPath($record.executable) -ne [System.IO.Path]::GetFullPath($executable) -or
                    @($record.features) -notcontains 'skinstrike-candidate' -or @($record.features) -contains 'update-fixture') {
                    throw 'Cargo reported an executable with the wrong manifest, target, or features.'
                }
                $artifact = $record
            }
            elseif ($record.reason -eq 'compiler-message' -and $record.message.rendered) {
                Write-Host $record.message.rendered
            }
        }
        if ($LASTEXITCODE -ne 0) { throw 'Candidate build failed; no launch or retry was attempted.' }
        if ($null -eq $artifact) { throw 'Cargo did not report the expected compiler artifact; no launch receipt was written.' }
        if ($drive.AvailableFreeSpace -lt 20GB) { throw '20 GiB reserve breached; stop further build work.' }
        if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) { throw 'Expected foundation executable was not produced.' }
        $afterHead = (& git -C $source rev-parse HEAD).Trim()
        if ($LASTEXITCODE -ne 0) { throw 'Cannot verify the post-build source commit.' }
        $afterStatus = & git -C $source status --porcelain=v1 | Out-String
        if ($LASTEXITCODE -ne 0) { throw 'Cannot verify the post-build source state.' }
        if ($afterHead -ne $sourceHead -or -not [string]::IsNullOrWhiteSpace($afterStatus)) {
            throw 'Candidate source changed during the build; no launch receipt was written.'
        }
        [pscustomobject]@{
            schema_version = 1
            candidate_commit = $sourceHead
            upstream_commit = $contract.upstream_commit
            contract_sha256 = $contractHash
            entry_sha256 = $entryHash
            binary_path = $executable
            binary_sha256 = (Get-FileHash -LiteralPath $executable -Algorithm SHA256).Hash
            feature = 'skinstrike-candidate'
            target = $contract.target
            built_at_utc = [DateTime]::UtcNow.ToString('o')
            runtime_verified = $false
        } | ConvertTo-Json | Set-Content -LiteralPath $receiptFile -Encoding utf8
        Write-Output 'Candidate built from recorded source; runtime verification remains pending.'
    }
    finally {
        foreach ($name in $savedBuildEnvironment.Keys) {
            [Environment]::SetEnvironmentVariable($name, $savedBuildEnvironment[$name], 'Process')
        }
    }
    return
}

if (-not (Test-Path -LiteralPath $receiptFile -PathType Leaf)) { throw 'No candidate build receipt exists. Run cannot select an old or unrelated executable.' }
$receipt = Get-Content -LiteralPath $receiptFile -Raw | ConvertFrom-Json
if ($receipt.schema_version -ne 1 -or $receipt.candidate_commit -ne $sourceHead -or
    $receipt.contract_sha256 -ne $contractHash -or $receipt.entry_sha256 -ne $entryHash -or
    $receipt.binary_path -ne $executable -or $receipt.feature -ne 'skinstrike-candidate' -or $receipt.target -ne $contract.target) {
    throw 'Candidate receipt does not match the current source and entry point; coordinate a new build.'
}
if (-not (Test-Path -LiteralPath $executable -PathType Leaf) -or
    (Get-FileHash -LiteralPath $executable -Algorithm SHA256).Hash -ne $receipt.binary_sha256) {
    throw 'Candidate executable is missing or differs from its build receipt.'
}
if ($AppArgs.Count -gt 0 -and $AppArgs[0] -notlike '--*' -and [System.IO.Path]::GetExtension($AppArgs[0]) -ieq '.json') {
    throw 'Existing SkinStrike JSON belongs to the preserved legacy editor. No automatic project conversion is implemented.'
}
$savedRunEnvironment = @{}
foreach ($name in @('SKINSTRIKE_CANDIDATE_HOME', 'SUBCRAFT_DATA', 'SUBCRAFT_LIBRARY')) {
    $savedRunEnvironment[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
}
try {
    New-Item -ItemType Directory -Path (Join-Path $candidateHome 'data'), (Join-Path $candidateHome 'library') -Force | Out-Null
    $env:SKINSTRIKE_CANDIDATE_HOME = $candidateHome
    $env:SUBCRAFT_DATA = Join-Path $candidateHome 'data'
    $env:SUBCRAFT_LIBRARY = Join-Path $candidateHome 'library'
    & $executable @AppArgs
    if ($LASTEXITCODE -ne 0) { throw 'Candidate exited unsuccessfully. Stop and report any platform policy block; this entry point never retries or changes policy.' }
}
finally {
    foreach ($name in $savedRunEnvironment.Keys) {
        [Environment]::SetEnvironmentVariable($name, $savedRunEnvironment[$name], 'Process')
    }
}
