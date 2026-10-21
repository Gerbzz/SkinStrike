#Requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{40}$')][string]$ExpectedHead
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path $PSScriptRoot -Parent
$source = Join-Path $root 'foundation/subcraft'
$contract = Get-Content (Join-Path $root 'foundation/candidate.json') -Raw | ConvertFrom-Json

function Read-Git([string]$directory, [string[]]$arguments) {
    $output = & git -C $directory @arguments
    if ($LASTEXITCODE -ne 0) { throw "Git inspection failed: $($arguments -join ' ')" }
    return ($output | Out-String).Trim()
}

$head = Read-Git $root @('rev-parse', 'HEAD')
if ($head -ne $ExpectedHead) { throw 'Parent checkout is not the exact requested PR head.' }
$entry = Read-Git $root @('ls-tree', 'HEAD', '--', 'foundation/subcraft')
if ($entry -notmatch '^160000 commit ([0-9a-f]{40})\s') { throw 'Foundation is not a pinned Git submodule.' }
$pinned = $Matches[1]
$actual = Read-Git $source @('rev-parse', 'HEAD')
if ($actual -ne $pinned -or $actual -ne $contract.candidate_commit) { throw 'Source, gitlink and contract disagree.' }
if (Read-Git $source @('status', '--porcelain=v1')) { throw 'Candidate source checkout is dirty.' }
$url = Read-Git $root @('config', '--file', '.gitmodules', '--get', 'submodule.subcraft-foundation.url')
if ($url -ne 'https://github.com/Gerbzz/SkinStrike.git' -or $url -ne $contract.source_repository_url) {
    throw 'Candidate source must be fetchable from its declared preserved source repository.'
}
if ($contract.upstream_url -ne 'https://github.com/Aalzard/subcraft.git' -or
    $contract.upstream_commit -ne '71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5') {
    throw 'Original SubCraft provenance changed.'
}
& git -C $source merge-base --is-ancestor $contract.upstream_commit HEAD
if ($LASTEXITCODE -ne 0) { throw 'Candidate lost its upstream ancestry.' }
$deleted = Read-Git $source @('diff', '--no-renames', '--name-only', '--diff-filter=D', $contract.upstream_commit, 'HEAD')
if ($deleted) { throw "Upstream files were deleted: $deleted" }
$baselinePaths = (Read-Git $source @('ls-tree', '-r', '--name-only', $contract.upstream_commit)) -split '\r?\n'
if ($baselinePaths.Count -ne 478) { throw 'Unexpected upstream baseline file inventory.' }
$immutable = @($baselinePaths | Where-Object {
    $_ -match '\.(png|jpg|jpeg|svg|ttf|otf|woff2?|hdr|exr)$' -or
    $_ -match '(?i)(^|/)(licen[cs]e[^/]*|copying[^/]*|notice[^/]*|credits[^/]*)$'
})
if ($immutable.Count -eq 0) { throw 'No original asset/license paths were checked.' }
$assetArguments = @('diff', '--name-only', $contract.upstream_commit, 'HEAD', '--') + $immutable
$changed = Read-Git $source $assetArguments
if ($changed) { throw "Original assets or notices changed: $changed" }

$launcher = Join-Path $root 'SkinStrike-3D.ps1'
$tokens = $null
$errors = $null
[void][System.Management.Automation.Language.Parser]::ParseFile($launcher, [ref]$tokens, [ref]$errors)
if ($errors.Count -ne 0) { throw ($errors | Out-String) }
$info = & $launcher -Action Info
if ($info.CandidateCommit -ne $actual -or $info.ParentPinnedCommit -ne $pinned -or
    $info.ContractCandidateCommit -ne $actual -or $info.CandidateSourceDirty -or
    $info.Package -ne 'subcraft' -or $info.Feature -ne 'skinstrike-candidate' -or
    $info.Target -ne 'x86_64-pc-windows-msvc') { throw 'Launcher Info selected the wrong candidate.' }

# Exercise refusals only with no receipt, executable or runtime home present.
# These checks cannot build or launch either app.
$receipt = Join-Path $root '.candidate/build-receipt.json'
$runtimeHome = Join-Path $root '.candidate/skinstrike-3d-candidate'
$executable = Join-Path $root '.candidate/target-subcraft/x86_64-pc-windows-msvc/debug/subcraft.exe'
foreach ($path in @($receipt, $runtimeHome, $executable)) {
    if (Test-Path -LiteralPath $path) { throw "Refusal check requires a fresh candidate state: $path" }
}
$buildError = $null
try { & $launcher -Action Build | Out-Null } catch { $buildError = $_.Exception.Message }
if ($buildError -notlike 'Build deferred:*') { throw "Missing build-slot refusal: $buildError" }
$runError = $null
try { & $launcher -Action Run | Out-Null } catch { $runError = $_.Exception.Message }
if ($runError -notlike 'No candidate build receipt exists.*') { throw "Missing receipt refusal: $runError" }
foreach ($path in @($receipt, $runtimeHome, $executable)) {
    if (Test-Path -LiteralPath $path) { throw "A refusal created candidate runtime state: $path" }
}

[pscustomobject]@{
    parent_head = $head
    candidate_commit = $actual
    upstream_commit = $contract.upstream_commit
    source_url = $url
    original_assets_and_notices_checked = $immutable.Count
    baseline_files_retained = $baselinePaths.Count
    launcher_parse_info_and_refusals = 'passed; no build or app launch'
    room_execution_policy = 'unchanged; prior Windows 4551 denial remains open'
} | ConvertTo-Json
