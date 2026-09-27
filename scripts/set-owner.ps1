# Sets the GitHub repository owner and the author in extension.toml and src/lib.rs.
# Safe to run again: it rewrites the values, so a typo can simply be fixed by re-running.
# Usage: .\scripts\set-owner.ps1 -User <github-user> -Name "<Your Name>" -Email <you@example.com>
param(
    [Parameter(Mandatory = $true)][string]$User,
    [Parameter(Mandatory = $true)][string]$Name,
    [Parameter(Mandatory = $true)][string]$Email,
    [string]$Repo = "zed-cortex-debug"
)
$ErrorActionPreference = "Stop"
if ($User -notmatch '^[A-Za-z0-9](?:[A-Za-z0-9-]{0,38})$') {
    throw "'$User' is not a valid GitHub user name (Latin letters, digits and '-')."
}
$Root = Split-Path -Parent $PSScriptRoot
$utf8 = New-Object System.Text.UTF8Encoding($false)

function Update-File([string]$rel, [scriptblock]$transform) {
    $p = Join-Path $Root $rel
    if (-not (Test-Path $p)) { Write-Host "skip (not found): $rel"; return }
    $t = [System.IO.File]::ReadAllText($p)
    $n = & $transform $t
    [System.IO.File]::WriteAllText($p, $n, $utf8)
    Write-Host "updated: $rel"
}

Update-File "extension.toml" {
    param($t)
    $t = [regex]::Replace($t, '(?m)^authors = .*$', "authors = [""$Name <$Email>""]")
    [regex]::Replace($t, '(?m)^repository = .*$', "repository = ""https://github.com/$User/$Repo""")
}
Update-File "src\lib.rs" {
    param($t)
    [regex]::Replace($t, 'const GITHUB_REPO: &str = "[^"]*";', "const GITHUB_REPO: &str = ""$User/$Repo"";")
}
Update-File "README.md" {
    param($t)
    [regex]::Replace($t, 'https://github\.com/[^/\s]+/zed-cortex-debug', "https://github.com/$User/$Repo")
}

Get-Content (Join-Path $Root "extension.toml") | Where-Object { $_ -match '^(authors|repository)' } | ForEach-Object { Write-Host "  $_" }
Get-Content (Join-Path $Root "src\lib.rs") | Where-Object { $_ -match 'GITHUB_REPO: &str' } | ForEach-Object { Write-Host "  $_" }
