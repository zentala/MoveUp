[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$ArtifactDirectory,
    [Parameter(Mandatory)][string]$ReleaseRoot,
    [Parameter(Mandatory)][string]$RunDirectory
)
$ErrorActionPreference = 'Stop'
foreach ($target in @($ArtifactDirectory, $ReleaseRoot, $RunDirectory)) {
    if (![IO.Path]::IsPathRooted($target)) { throw 'All paths must be absolute' }
}
$executable = Join-Path $ArtifactDirectory 'desk.exe'
# Refuse legacy binaries before invoking a command they do not implement.
$info = [Diagnostics.FileVersionInfo]::GetVersionInfo($executable)
if ($info.FileMajorPart -lt 0 -or ($info.FileMajorPart -eq 0 -and
    ($info.FileMinorPart -lt 6 -or ($info.FileMinorPart -eq 6 -and $info.FileBuildPart -lt 1)))) {
    throw 'Legacy binaries do not implement release maintenance; build the new patch first'
}
$metadataText = & $executable --release describe
if ($LASTEXITCODE -ne 0) { throw 'Build metadata command failed' }
$metadata = $metadataText | ConvertFrom-Json
if ($metadata.schema_version -ne 1 -or $metadata.readiness_contract -ne 1 -or
    $metadata.build_mode -ne 'release' -or $metadata.app_id -ne 'io.zntl.desk') {
    throw 'Not a supported release build'
}
# Build output only. The store CLI validates, copies and publishes the complete tree.
[IO.File]::WriteAllText((Join-Path $ArtifactDirectory 'release.json'), $metadataText, [Text.UTF8Encoding]::new($false))
& $executable --release stage --root $ReleaseRoot --source $ArtifactDirectory --version $metadata.version --run-directory $RunDirectory
if ($LASTEXITCODE -ne 0) { throw 'Release staging failed; no cutover performed' }
