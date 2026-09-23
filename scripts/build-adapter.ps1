# Rebuilds adapter\ from Cortex-Debug sources + patches\*.patch (Windows PowerShell).
# Usage: .\scripts\build-adapter.ps1 [-Src C:\path\to\cortex-debug]
param([string]$Src = "")
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
if (-not $Src) { $Src = Join-Path $Root ".build\cortex-debug" }
$Base = (Get-Content (Join-Path $Root "patches\BASE_COMMIT")).Trim()

if (-not (Test-Path (Join-Path $Src ".git"))) {
    git clone https://github.com/Marus/cortex-debug.git $Src
    git -C $Src checkout -q $Base
    $patches = Get-ChildItem (Join-Path $Root "patches\*.patch") | ForEach-Object { $_.FullName }
    git -C $Src -c user.name=build -c user.email=build@localhost am -q @patches
}

Push-Location $Src
try {
    npm ci --ignore-scripts
    npx webpack --config-name zedAdapter --mode production
} finally { Pop-Location }

New-Item -ItemType Directory -Force (Join-Path $Root "adapter\dist"), (Join-Path $Root "adapter\support") | Out-Null
Copy-Item (Join-Path $Src "dist\zedadapter.js") (Join-Path $Root "adapter\dist\") -Force
Copy-Item (Join-Path $Src "support\gdbsupport.init"), (Join-Path $Src "support\gdb-swo.init") (Join-Path $Root "adapter\support\") -Force
Copy-Item (Join-Path $Src "LICENSE") (Join-Path $Root "adapter\LICENSE-cortex-debug") -Force
Write-Host "adapter\ updated. Rebuild the extension in Zed (Extensions -> Rebuild)."
