# LiveScreen

Compartilhamento de tela P2P via iroh (e WebRTC para o navegador), sem servidor de vídeo. Três partes:

- **Desktop (Windows):** um único `livescreen.exe` (Rust, edição 2024) que transmite ou assiste uma tela/janela.
- **Núcleo** (`crates/livescreen-core`): protocolo, host, presets, sessão do viewer, amigos, chat e sinalização web, sem dependência de plataforma.
- **Web** (`web/`): Cloudflare Worker que faz a sinalização e serve a página para assistir pelo navegador (`livescreen.pendiego.com/v/`).

Visão de usuário, requisitos e conexões externas: `README.md`.

## Comandos

```powershell
cargo build --release          # exige toolchain MSVC e CMake no PATH (compila o Opus)
cargo test --workspace         # testes do desktop e do núcleo; o CI (windows-latest) roda `cargo test`
cargo test --release codec_roundtrip -- --ignored --nocapture   # GPU real
cargo test --release e2e_window -- --ignored --nocapture        # janela real; python com tkinter no PATH
cargo test --release e2e_friends -- --ignored --nocapture       # amigos/presença com rede real
.\target\release\livescreen.exe --probe-encoder | Out-String   # diagnóstico do encoder
.\target\release\livescreen.exe --selftest-capture | Out-String
```

Web (de `web/`, Node 22):

```powershell
npm run dev     # wrangler dev em 127.0.0.1:8787; o host aponta para ele com LIVESCREEN_WEB_URL=http://127.0.0.1:8787
npm run check   # wrangler deploy --dry-run; parte do job `web` do CI
npm test        # vetor v3 Rust↔JS, signal3.js e verificador do manifesto
node scripts/verify-manifest.mjs   # página assinada confere (rode sign-web.ps1 depois de mexer em public/v)
```

O build release usa `windows_subsystem = "windows"`: sem console, a saída só aparece redirecionada. Logs vão para `%APPDATA%\LiveScreen\livescreen.log` (`tracing`).

## Mapa do código

### Desktop (`src/`)

|Caminho|Papel|
|---|---|
|`src/main.rs`|entrada: flags de diagnóstico, limpeza do updater, `ui::run()`|
|`src/share.rs`|lado Windows de quem transmite: captura, encoder e áudio entregues ao `livescreen_core::host` por `Feeds`; troca de qualidade e codec ao vivo|
|`src/watch.rs`|lado de quem assiste: usa `livescreen_core::viewer`, recebe vídeo e áudio, publica `WatchStatus`|
|`src/capture/`|`wgc.rs` (Windows Graphics Capture), `dxgi.rs` (Desktop Duplication, fallback de tela), `hdr.rs` (monitor em HDR: captura FP16 → BGRA8 SDR por shader, branco SDR do Windows), `sources.rs` (enumeração/miniaturas); `FrameSink` recebe sempre texturas D3D11 BGRA8 SDR|
|`src/video/`|`encoder.rs`/`decoder.rs` H.264/HEVC/AV1 via Media Foundation, `mf.rs` helpers, `convert.rs` RGB→NV12 com escala no D3D11 Video Processor (GPU) e NV12→RGBA (pool compartilhável), `present.rs` viewer zero-copy (texturas D3D11 compartilhadas → wgpu DX12, fences `produced`/`consumed`), `probe.rs` detecção do encoder em processo filho|
|`src/audio/`|`capture.rs` loopback WASAPI → Opus; `playback.rs` datagram → Opus → cpal|
|`src/speedtest.rs`|medição de upload contra `speed.cloudflare.com`|
|`src/update.rs`|auto-update pelo GitHub Releases com conferência de sha256 e assinatura Ed25519 (`livescreen_core::page_check::RELEASE_PUBLIC_KEY`)|
|`src/clipboard.rs`|cópia sensível do convite e do link do navegador (fora do histórico `Win+V` e da nuvem); cai para `ctx.copy_text` se falhar|
|`src/config.rs`|`%APPDATA%\LiveScreen\`, `config.json`, `LIVESCREEN_CAPTURE_OVERRIDES` e a identidade (`identity.key` + trava `identity.lock`)|
|`src/ui/`|egui/eframe: `home`, `share_view`, `watch_view`, `chat` (painel compartilhado por host e espectador), `friends` (lista, prober de presença, modal de adicionar), `settings` (modal), `prefs` (`ui.json`), `avatar` (desenho e escolha), `theme`, `toast`|
|`src/gpu.rs`|device D3D11 compartilhado entre captura, encoder e viewer|
|`src/icon.rs`|ícone desenhado em código; incluído pelo `build.rs` e pela UI (fonte única)|
|`src/e2e_window.rs`|testes `#[ignore]` com janela e rede reais (`e2e_window`, `e2e_friends`)|

### Núcleo (`crates/livescreen-core/src/`)

|Arquivo|Papel|
|---|---|
|`protocol.rs`|wire format: ALPN, `PROTOCOL_VERSION` (5) e `LEGACY_PROTOCOL_VERSION` (4), mensagens JSON (`ViewerMsg`/`HostMsg`), quadros de vídeo, códigos de close, `MAX_MSG`/`MAX_FRAME`|
|`preset.rs`|qualidades (480p..4K/nativa), limites H.264 nível 5.2, bitrate e recomendação|
|`viewer.rs`|sessão do viewer do desktop: convite (`Target::Invite`/`Target::Friend`), conexão, GOPs, keyframe, estatísticas, parada|
|`friends.rs`|`SecretKey` persistente, código de amigo (id hex), ALPN de presença, `Prober`, `PresenceState`|
|`host/mod.rs`|host sem plataforma: endpoint iroh (ALPN + presença), handshake, sala (limite, kick, ban), fan-out de vídeo/áudio/chat, ABR; a plataforma entrega quadros codificados por `Feeds`|
|`host/web.rs`|espectadores do navegador: WebRTC com `str0m` (uma sessão e um socket UDP por peer; H.264 sempre, AV1 quando o navegador oferece), sinalização com o Worker (`LIVESCREEN_WEB_URL`, padrão `https://livescreen.pendiego.com`), reconexão com backoff|
|`web.rs`|link do navegador `.v3` (`WebKeys`: chave de 256 bits + X25519/ML-KEM-768 do host), offer/answer híbridos (`open_offer`, `AnswerSealer`), STUN manual (srflx); desenho em `docs/rfc-sinalizacao-web-v3.md`|
|`page_check.rs`|antes de conectar a sinalização, confere a página do espectador contra `web/public/v/manifest.json` assinado com a chave de release (loopback pula)|
|`chat.rs`|mensagens do chat, papéis e limites|
|`avatar.rs`|os 7 avatares como dados (formas num quadro 64×64), `auto`/`resolve` e `json()` para o navegador|

### Web (`web/`)

|Caminho|Papel|
|---|---|
|`web/src/worker.js`|Cloudflare Worker + Durable Object `Room` (um por sala): sinalização WebSocket entre host (`/ws/host/<sala>`) e espectadores (`/ws/view/<sala>`), sem TURN (só entrega STUN), limites e expiração de sockets|
|`web/public/v/`|página do espectador (`index.html`, `viewer.js`, `viewer.css`, `avatars.json`, fontes): WebRTC, player, estatísticas e chat; `signal3.js` (cripto v3) e `pq.js` (noble ML-KEM/X25519 empacotado por `web/scripts/build-pq.mjs`) só carregam para links `.v3`; `manifest.json(.sig)` assinados por `.github/scripts/sign-web.ps1`; sem build, servida como asset estático|
|`web/public/_headers`|CSP e cabeçalhos de segurança; cache longo só para `/v/fonts/*`|
|`web/wrangler.jsonc`|configuração do Worker (assets, binding `ROOM`, migrations)|

## Regras

- Desktop: só Windows 10 2004+ x64. A crate `windows` está fixada em `=0.62.2`: ao precisar de API nova, adicione a feature em `Cargo.toml` em vez de subir a versão.
- Código sem plataforma (protocolo, host, viewer, presets, amigos, chat, sinalização web) vai em `livescreen-core`; o que depende de Windows fica em `src/`.
- Protocolo: qualquer mudança incompatível em `protocol.rs` (campos obrigatórios, enum, framing) exige incrementar `PROTOCOL_VERSION`; versões diferentes são recusadas. Campos novos opcionais usam `#[serde(default)]` para não quebrar. O viewer ainda fala `LEGACY_PROTOCOL_VERSION` com hosts 0.4 (convite sem chave de sala): preserve esse caminho e a tabela de compatibilidade do `README.md` ("Versões"). Apps Android 0.5–0.7 já instalados continuam conectando como viewers: o app saiu do repositório, mas o protocolo não muda por isso.
- Limites de entrada da rede (`MAX_MSG`, `MAX_FRAME` em `protocol.rs`; `MAX_HANDSHAKES`, `HANDSHAKE_TIMEOUT` em `host/mod.rs`; `MAX_SDP` em `host/web.rs`; limites do Worker) são fronteira de confiança: não afrouxe sem motivo.
- Captura e encoder têm fallback (WGC → DXGI, hardware → software) e overrides (`deny-graphics-capture`, `deny-dxgi`, `deny-hardware-encoder`, `deny-hardware-decoder`, `deny-zero-copy`). Mudanças nesses caminhos devem preservar os dois lados do fallback.
- Threads de captura/encoder não podem morrer por um quadro ruim (ver `SinkHandle::lock`, que recupera mutex envenenado).
- Updater: o asset `livescreen.exe` solto da release é o que o app desktop baixa e confere pelo sha256 do GitHub e pela assinatura Ed25519 do asset `livescreen.exe.sig` (chave pública `UPDATE_KEY` em `update.rs`). Não renomeie assets em `release.yml` sem ajustar `update.rs`.
- Dependências: `reqwest`/`rustls` reaproveitam o que o iroh já traz (provider `ring`); evite adicionar crates para o que a stdlib, `windows` ou as dependências atuais resolvem.
- Novas conexões de rede externas precisam entrar na tabela "Conexões que o app faz" do `README.md`.
- Viewer desktop: decoder por hardware (`video/decoder.rs`, MFT com `IMFDXGIDeviceManager` + `Converter::new_rgb`) cai para software após 3 falhas seguidas; preservar os dois caminhos. O quadro vai à UI por textura compartilhada (`GpuOut::Shared`, `video/present.rs`) e cai para readback (`GpuOut::Readback`) quando o egui não roda em DX12, o adaptador difere, falta saída RGBA, a importação falha ou há `deny-zero-copy`; preservar os dois caminhos. `ViewerMsg::SetQuality` é pedido, não ordem: `share.rs::apply_quality_request` só aceita redução real (`preset::is_reduction`).
- ABR (`host/mod.rs`, tick de 1 s): congestionamento de qualquer espectador (iroh ou navegador) reduz o bitrate da sala inteira até o teto do preset. Abaixo de 50%/25% do teto, `HostStats.target_fps` corta o fps (com histerese) e o encoder desktop compensa o `MeanBitRate` pela razão de fps, senão o MFT gasta só a fração do fps nominal.

### Navegador (web)

- O link é `#sala.chave.impressão.v3` e nunca chega ao servidor; a página o tira da barra de endereço e guarda em `sessionStorage`. A chave (`web_key`, 32 bytes) e o par X25519 + ML-KEM-768 do host (`WebKeys`) são trocados juntos em todo `new_invite` (inclui `kick`) e só vivem na memória. O host publica a chave pública ao Worker (`SignalOut::Pk`, `{"t":"pk"}`) depois do `ready`, a cada reconexão e a cada troca; a página confere `pk` contra a impressão do link. O Worker não conhece a chave: quem autentica o espectador é o host, abrindo o offer (`web.rs`, `WebKeys::open_offer`, ordem tamanho → prefixo `0x03` → `pre` → decapsulação → AEAD, falha sempre com o mesmo motivo). Não mova a verificação para o Worker. O formato é byte a byte o mesmo em `web.rs` e `signal3.js`, travado pelo vetor `web/test/v3-vector.json` (testes Rust e node).
- Host novo só aceita `.v3`; a página mantém os caminhos `.v2` (AES-GCM com a chave da sala) e v1 (prova HMAC) para hosts 0.6 e anteriores, e um link `.v3` nunca cai para eles.
- O Worker só repassa sinalização (offer/answer/reject/close/left/pk). O formato dessas mensagens (`SignalOut`/`SignalIn` em `web.rs`) é contrato com hosts já instalados: mudanças só aditivas, e Worker novo precisa funcionar com host antigo. Ordem de publicação: Worker antes da release do host.
- Limites do Worker (`Room`): até 4 sockets de host e 64 de espectador por sala, mensagem até 64 KiB, host sem autenticação em 10 s e espectador sem oferta em 30 s são fechados (código 4008, `alarm()`), no máximo 20 offers por sala a cada 10 s (código 4029), `pk` com exatamente 2432 hex e só de host autenticado (guardado em `ctx.storage`, apagado quando sai o último host). Viewer id tem 8 hex (`valid_peer_id`).
- Nunca registre em log link, `web_key`, `sealed`, SDP nem chaves privadas.
- Depois da resposta o vídeo é P2P: queda do WebSocket de sinalização não encerra quem já assiste. O host mantém o canal e o mapa de peers web durante a transmissão inteira e reconecta com backoff.
- CSP estrita em `web/public/_headers` (sem inline, só `'self'`, nenhum script de terceiros: ele leria o `sessionStorage` com o link). A página não usa script/estilo inline nem `innerHTML`; dados que chegam pela rede (nome, texto, motivo) são validados e cortados antes de ir ao DOM via `textContent`. Nova origem externa exige entrada na CSP e na tabela de conexões do `README.md`.
- Página assinada: qualquer mudança em `web/public/v/` exige rodar `.github/scripts/sign-web.ps1` (chave de release) e comitar `manifest.json` e `manifest.json.sig`; o job `web` do CI e o `deploy-web.yml` rodam `web/scripts/verify-manifest.mjs` e falham sem isso. Hosts instalados recusam a página se a assinatura não conferir.
- Sem TURN: o Worker só entrega STUN; o host fura o NAT com IPv6, portas previstas do navegador e do host.

### Chat

- O host é o centro: atribui `sender` (0 é o host) e `role`; o nome é só rótulo e nunca dá privilégio. Limites em `chat.rs` (`MAX_CHAT_CHARS` 500, `MAX_NAME_CHARS` 32). Exige host e viewer 0.6+; com ponta antiga a transmissão segue sem chat.
- Avatares: pela rede só vai o id (`u8`, 0 = automático); o host valida e resolve (`avatar::resolve`, chave = nome ou `#sender`) antes de repassar em `ChatLine`, `ViewerInfo` e `Welcome.host_avatar`. Os desenhos vivem só em `crates/livescreen-core/src/avatar.rs`: o navegador lê por `web/public/v/avatars.json`, que um teste compara com `avatar::json()` (mudou o desenho, regenere o arquivo). O desktop desenha com o egui, que só preenche polígono convexo: polígono côncavo vira triângulos. O automático (FNV-1a 32) existe em Rust e JS e precisa dar o mesmo número nas duas.

### Amigos e presença

- **Invariante: nunca dois endpoints iroh vivos com a mesma `SecretKey`** (o relay derruba um deles). Toda abertura de endpoint espera o fechamento do anterior: no desktop, `take_prober` + `join_background` antes de assistir ou transmitir.
- Uma identidade por janela: `config::identity()` reserva `identity.lock`; a segunda janela do desktop recebe chave temporária e fica sem amigos (`identity_temporary`). Arquivo ilegível (`identity.key`, `friends.json`) nunca é sobrescrito: vai para `.bad` ou a janela usa chave temporária.
- A presença usa ALPN próprio e só responde a amigos; quem foi banido da transmissão recebe `CLOSE_DENIED` com `REMOVED_REASON` (estado `Removed`). Endpoints que só discam não publicam no pkarr; só o host publica.

## Estilo

- Textos de UI, mensagens de erro, comentários e doc comments em português (pt-BR). Identificadores em inglês.
- `//!` no topo do módulo explica o porquê do desenho; comentários só para o que o código não mostra.
- Testes unitários ficam no próprio arquivo (`#[cfg(test)] mod tests`). Testes que exigem GPU, janela ou rede real são `#[ignore]` com o comando de execução no motivo ou no `//!`.
- Erros com `anyhow`; logs com `tracing` (`info!`/`warn!`), nunca `println!` fora dos modos de diagnóstico.
- `#![allow(dead_code)]` está ativo no crate: confira manualmente se código novo ficou sem uso.
- Mudou comportamento visível ao usuário (requisitos, conexões, versões, recursos)? Atualize o `README.md` no mesmo PR.

## CI

`.github/workflows/ci.yml` roda em push na `main` e à mão (`workflow_dispatch`; PR não dispara sozinho, rode em qualquer branch com `gh workflow run CI --ref <branch>`): `build` (`cargo build --release` + `cargo test`, windows-latest), `changelog` (`.github/scripts/test-changelog.sh`, valida o `cliff.toml`) e `web` (`npm run check`, `npm test` e `verify-manifest.mjs` em `web/`).

`deploy-web.yml` publica o Worker (`wrangler deploy`, segredos `CLOUDFLARE_API_TOKEN`/`CLOUDFLARE_ACCOUNT_ID`) em push que mexe em `web/**` na `main` ou por `workflow_dispatch` (só roda com ref `main`), depois de conferir o manifesto assinado. O job usa o environment `production` (só `main`, aprovação do mantenedor): o deploy fica esperando até alguém aprovar em Actions (ou `gh api -X POST repos/pendiego/LiveScreen/actions/runs/<id>/pending_deployments`). `pages.yml` publica o site estático de `site/` no GitHub Pages. O Worker não depende de release: entra no ar no deploy e precisa continuar compatível com hosts já instalados.

## Release

1. Atualize `version` em `Cargo.toml` (o updater compara com `CARGO_PKG_VERSION`).
2. Commit `chore(release): X.Y.Z`, tag `vX.Y.Z`, push da tag: `release.yml` compila o `.exe` e o `.zip`, publica os dois e gera as notas com o `git-cliff` (`cliff.toml`). O `release.yml` cria a release como rascunho. Depois rode `.github/scripts/sign-release.ps1 -Tag vX.Y.Z` (chave privada em `%USERPROFILE%\.livescreen-release\update-key.pk8`, fora do repositório): o script assina o exe, sobe `livescreen.exe.sig` e publica. Release em rascunho não chega a nenhum updater.

Commits seguem Conventional Commits em português (`feat:`, `fix:`, `perf:`, `docs:`, `ci:`, `chore(release):`). As notas da release são esses commits desde a tag anterior, agrupados por tipo; itens `- ...` no corpo viram subitens e `!`/`BREAKING CHANGE:` entra em "Breaking Changes". Por isso PRs entram por rebase ou merge commit, nunca squash (o squash junta tudo num commit só e as notas perdem os itens).

## graphify

This project has a knowledge graph at graphify-out/ with god nodes, community structure, and cross-file relationships.

When the user types `/graphify`, use the installed graphify skill or instructions before doing anything else.

Rules:
- When graphify-out/graph.json exists, run `graphify query "<question>"` before your first grep, search or source-file read for a codebase question, and put this rule in every subagent prompt that explores code. Read raw files only for lines the graph or an earlier read already located. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts. These return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep output.
- Dirty graphify-out/ files are expected after hooks or incremental updates; dirty graph files are not a reason to skip graphify. Only skip graphify if the task is about stale or incorrect graph output, or the user explicitly says not to use it.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).
- A committed graphify-out/graph.json is rewritten whole on every build, so a large diff to it in a pull request is expected. Do not flag it as churn or suggest skipping the graph rebuild or its hooks.
