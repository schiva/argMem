# Prints the body of the "## [<Version>]" section of CHANGELOG.md (used for release notes).
# Fails if the section is missing or empty.
param(
    [Parameter(Mandatory)][string]$Version,
    [string]$Path
)
$ErrorActionPreference = 'Stop'
if (-not $Path) {
    # Windows PowerShell 5.1 leaves $PSScriptRoot empty in param defaults, so resolve it here.
    $here = if ($PSScriptRoot) { $PSScriptRoot } else { Split-Path -Parent $MyInvocation.MyCommand.Path }
    $Path = Join-Path $here '..\CHANGELOG.md'
}
$text = [IO.File]::ReadAllText((Resolve-Path $Path), [Text.Encoding]::UTF8) -replace "`r`n", "`n"
$pattern = '(?ms)^## \[' + [regex]::Escape($Version.TrimStart('v')) + '\][^\n]*\n(.*?)(?=^## \[|\z)'
$m = [regex]::Match($text, $pattern)
if (-not $m.Success -or -not $m.Groups[1].Value.Trim()) {
    throw "CHANGELOG.md has no section for $Version"
}
$m.Groups[1].Value.Trim()
