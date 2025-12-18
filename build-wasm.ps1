param(
    $crate
)

if ($crate -eq $null)
{
    $crates = @(
        "general",
        "sign"
    )
}
else
{
    $crates = @($crate)
}

foreach ($crate in $crates)
{
    Write-Host "Building $crate..." -ForegroundColor Green
    Push-Location $crate
    try
    {
        wasm-pack build --release --target web
        if ($LASTEXITCODE -ne 0)
        {
            exit $LASTEXITCODE
        }
        if ($crate -eq "sign")
        {
            wasm-strip pkg/template_replacement_sign_core_wasm_bg.wasm
        }
        else
        {
            wasm-strip pkg/template_replacement_core_wasm_bg.wasm
        }
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