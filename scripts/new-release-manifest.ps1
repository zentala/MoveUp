[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$ArtifactDirectory,
    [Parameter(Mandatory)]
    [ValidatePattern('^[0-9a-fA-F]{40}$')]
    [string]$Revision,
    [Parameter(Mandatory)]
    [string]$OutputPath
)

$artifacts = Get-ChildItem -LiteralPath $ArtifactDirectory -Filter '*.exe' -File
if ($artifacts.Count -eq 0) {
    throw "No .exe artifacts found in $ArtifactDirectory."
}

$manifest = [ordered]@{
    schema_version = 1
    revision = $Revision.ToLowerInvariant()
    generated_at_utc = (Get-Date).ToUniversalTime().ToString('o')
    artifacts = @(
        $artifacts | Sort-Object Name | ForEach-Object {
            [ordered]@{
                file_name = $_.Name
                sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
                bytes = $_.Length
            }
        }
    )
}

$parent = Split-Path -Parent $OutputPath
if ($parent) {
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
}
$manifest | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $OutputPath -Encoding utf8
