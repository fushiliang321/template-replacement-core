param(
    $crate
)

$wasmPaths = @{
    "general"          = "pkg/template_replacement_core_wasm_bg.wasm"
    "sign"             = "pkg/template_replacement_sign_core_wasm_bg.wasm"
    "general-polyfill" = "pkg/template_replacement_core_wasm_polyfill_polyfill_bg.wasm"
    "sign-polyfill"    = "pkg/template_replacement_sign_core_wasm_polyfill_bg.wasm"
}

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
    }
    finally
    {
        Pop-Location
    }
}

Write-Host "All WASM crates built successfully!" -ForegroundColor Yellow