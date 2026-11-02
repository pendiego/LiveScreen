# Assina offline o manifesto dos arquivos servidos em /v/ (web/public/v/).
# A chave privada fica em $env:USERPROFILE\.livescreen-release\update-key.pk8 (fora do repositório).
param(
    [string]$Key = "$env:USERPROFILE\.livescreen-release\update-key.pk8",
    [string]$WebDir = ""
)

$ErrorActionPreference = "Stop"

if (-not $WebDir) {
    $WebDir = Join-Path $PSScriptRoot "..\..\web\public\v"
}
$WebDir = (Resolve-Path $WebDir).Path

if (-not (Test-Path $WebDir)) {
    throw "Diretório não encontrado: $WebDir"
}

if (-not (Test-Path $Key)) {
    throw "Chave privada não encontrada: $Key"
}

# Coleta todos os arquivos recursivamente, ignorando manifest.json e manifest.json.sig
$files = Get-ChildItem -Path $WebDir -Recurse -File | Where-Object {
    $_.Name -ne "manifest.json" -and $_.Name -ne "manifest.json.sig"
}

if ($files.Count -eq 0) {
    throw "Nenhum arquivo encontrado em $WebDir"
}

# Ordena os arquivos pelo caminho relativo com barra normal ('/')
$sortedFiles = $files | Sort-Object {
    $rel = $_.FullName.Substring($WebDir.Length).TrimStart('\', '/').Replace('\', '/')
    $rel
}

$map = [ordered]@{}
foreach ($f in $sortedFiles) {
    $rel = $f.FullName.Substring($WebDir.Length).TrimStart('\', '/').Replace('\', '/')
    if ($rel -match '\.\.' -or $rel -notmatch '^[A-Za-z0-9._/-]+$') {
        throw "Caminho inválido detectado: $rel"
    }
    $sha = (Get-FileHash -Path $f.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    $map[$rel] = $sha
}

$manifestObj = [ordered]@{
    "files" = $map
}

# LF puro: o git (eol=lf) gravaria CRLF como LF e a assinatura deixaria de conferir no CI
$manifestJson = (($manifestObj | ConvertTo-Json -Depth 5) -replace "`r`n", "`n") + "`n"
$manifestPath = Join-Path $WebDir "manifest.json"
$sigPath = Join-Path $WebDir "manifest.json.sig"

# Grava manifest.json em UTF-8 sem BOM
[System.IO.File]::WriteAllText($manifestPath, $manifestJson, (New-Object System.Text.UTF8Encoding($false)))

# Assina usando a ferramenta sign_release
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Push-Location $repoRoot
try {
    $sig = cargo run -p livescreen-core --release --quiet --example sign_release -- sign-web $Key $manifestPath
    if ($LASTEXITCODE -ne 0) {
        throw "Assinatura do manifesto falhou (código $LASTEXITCODE)"
    }
} finally {
    Pop-Location
}

$sigHex = ($sig | Out-String).Trim()
[System.IO.File]::WriteAllText($sigPath, $sigHex + "`n", (New-Object System.Text.UTF8Encoding($false)))

Write-Host "Manifesto assinado com sucesso ($($files.Count) arquivos):"
Write-Host "  Manifesto:  $manifestPath"
Write-Host "  Assinatura: $sigPath"
