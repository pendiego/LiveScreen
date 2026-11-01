# Assina offline o livescreen.exe de uma release em rascunho e a publica.
# A chave privada fica em $env:USERPROFILE\.livescreen-release\update-key.pk8 (fora do repositório).
param([Parameter(Mandatory)][string]$Tag, [string]$Key = "$env:USERPROFILE\.livescreen-release\update-key.pk8")
$ErrorActionPreference = "Stop"
$repo = "pendiego/LiveScreen"
$ver = $Tag.TrimStart("v")
$dir = Join-Path $env:TEMP "livescreen-sign-$Tag"
New-Item -ItemType Directory -Force $dir | Out-Null
gh release download $Tag -R $repo -p livescreen.exe -D $dir --clobber; if ($LASTEXITCODE) { throw "download falhou" }
$sig = cargo run -p livescreen-core --release --quiet --example sign_release -- sign $Key $ver (Join-Path $dir "livescreen.exe")
if ($LASTEXITCODE) { throw "assinatura falhou" }
[IO.File]::WriteAllText((Join-Path $dir "livescreen.exe.sig"), ($sig | Out-String).Trim())
gh release upload $Tag (Join-Path $dir "livescreen.exe.sig") -R $repo --clobber; if ($LASTEXITCODE) { throw "upload falhou" }
gh release edit $Tag -R $repo --draft=false; if ($LASTEXITCODE) { throw "publicação falhou" }
