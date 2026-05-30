param(
    $crate
)

$wasmPaths = @{
    "general" = "pkg/template_replacement_core_wasm_bg.wasm"
    "sign" = "pkg/template_replacement_sign_core_wasm_bg.wasm"
    "general-polyfill" = "pkg/template_replacement_core_wasm_polyfill_bg.wasm"
    "sign-polyfill" = "pkg/template_replacement_sign_core_wasm_polyfill_bg.wasm"
}

$repositoryUrl = "git+https://github.com/fushiliang321/template-replacement-core.git"

if ($crate -eq $null)
{
    $crates = @($wasmPaths.Keys)
}
else
{
    $crates = @($crate)
}

foreach ($crate in $crates)
{
    Write-Host "Building $crate..." -ForegroundColor Green
    Push-Location (Join-Path $PSScriptRoot $crate)
    try
    {
        wasm-pack build --release --target web
        if ($LASTEXITCODE -ne 0)
        {
            exit $LASTEXITCODE
        }
        wasm-strip $wasmPaths[$crate]
        if ($LASTEXITCODE -ne 0)
        {
            exit $LASTEXITCODE
        }
        $pkgJsonPath = Join-Path (Join-Path $PSScriptRoot $crate) "pkg/package.json"
        $pkgJson = Get-Content $pkgJsonPath -Raw -Encoding UTF8 | ConvertFrom-Json
        $pkgJson | Add-Member -NotePropertyName "repository" -NotePropertyValue @{ type = "git"; url = $repositoryUrl } -Force
        $jsonText = $pkgJson | ConvertTo-Json -Depth 3
        [System.IO.File]::WriteAllText($pkgJsonPath, $jsonText, [System.Text.UTF8Encoding]::new($false))
        node -e "const f=require('fs'),p=process.argv[1];f.writeFileSync(p,JSON.stringify(JSON.parse(f.readFileSync(p,'utf8')),null,2)+'\n','utf8')" $pkgJsonPath
    }
    finally
    {
        Pop-Location
    }
}

Write-Host "All WASM crates built successfully!" -ForegroundColor Yellow