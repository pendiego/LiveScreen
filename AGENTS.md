# LiveScreen

Compartilhamento de tela P2P via iroh, sem servidor de vídeo. Workspace Cargo com três partes:

- **Desktop (Windows):** um único `livescreen.exe` (Rust, edição 2024) que transmite ou assiste uma tela/janela.
- **Núcleo** (`crates/livescreen-core`): protocolo, presets, sessão do viewer e amigos, sem dependência de plataforma.
- **Android** (`android/`): só assiste. Kotlin/Compose + núcleo Rust via JNI (`android/rust`).

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

Android (de `android/`, JDK 21, `cargo-ndk` instalado, `ANDROID_NDK_HOME` definido):

```powershell
.\gradlew.bat assembleDebug testDebugUnitTest   # o mesmo que o job `android` do CI
```

O build release usa `windows_subsystem = "windows"`: sem console, a saída só aparece redirecionada. Logs vão para `%APPDATA%\LiveScreen\livescreen.log` (`tracing`); no Android, `logcat -s LiveScreen`.

## Mapa do código

### Desktop (`src/`)

|Caminho|Papel|
|---|---|
|`src/main.rs`|entrada: flags de diagnóstico, limpeza do updater, `ui::run()`|
|`src/share.rs`|lado de quem transmite: endpoint iroh, handshake, sala (limite, kick), fan-out de vídeo/áudio, troca de qualidade ao vivo, resposta de presença|
|`src/watch.rs`|lado de quem assiste: usa `livescreen_core::viewer`, recebe vídeo e áudio, publica `WatchStatus`|
|`src/capture/`|`wgc.rs` (Windows Graphics Capture), `dxgi.rs` (Desktop Duplication, fallback de tela), `sources.rs` (enumeração/miniaturas); `FrameSink` recebe texturas D3D11|
|`src/video/`|`encoder.rs`/`decoder.rs` H.264/HEVC/AV1 via Media Foundation, `mf.rs` helpers, `convert.rs` RGB→NV12 com escala no D3D11 Video Processor (GPU) e NV12→RGBA (pool compartilhável), `present.rs` viewer zero-copy (texturas D3D11 compartilhadas → wgpu DX12, fences `produced`/`consumed`), `probe.rs` detecção do encoder em processo filho|
|`src/audio/`|`capture.rs` loopback WASAPI → Opus; `playback.rs` datagram → Opus → cpal|
|`src/speedtest.rs`|medição de upload contra `speed.cloudflare.com`|
|`src/update.rs`|auto-update pelo GitHub Releases com conferência de sha256|
|`src/config.rs`|`%APPDATA%\LiveScreen\`, `config.json`, `LIVESCREEN_CAPTURE_OVERRIDES` e a identidade (`identity.key` + trava `identity.lock`)|
|`src/ui/`|egui/eframe: `home`, `share_view`, `watch_view`, `friends` (lista, prober de presença, modal de adicionar), `settings` (modal), `prefs` (`ui.json`), `theme`, `toast`|
|`src/gpu.rs`|device D3D11 compartilhado entre captura, encoder e viewer|
|`src/icon.rs`|ícone desenhado em código; incluído pelo `build.rs` e pela UI (fonte única)|
|`src/e2e_window.rs`|testes `#[ignore]` com janela e rede reais (`e2e_window`, `e2e_friends`)|

### Núcleo (`crates/livescreen-core/src/`)

|Arquivo|Papel|
|---|---|
|`protocol.rs`|wire format: ALPN, `PROTOCOL_VERSION` (5) e `LEGACY_PROTOCOL_VERSION` (4), mensagens JSON (`ViewerMsg`/`HostMsg`), quadros de vídeo, códigos de close|
|`preset.rs`|qualidades (480p..4K/nativa), limites H.264 nível 5.2, bitrate e recomendação|
|`viewer.rs`|sessão do viewer compartilhada por desktop e Android: convite (`Target::Invite`/`Target::Friend`), conexão, GOPs, keyframe, estatísticas, parada|
|`friends.rs`|`SecretKey` persistente, código de amigo (id hex), ALPN de presença, `Prober`, `PresenceState`|

### Android (`android/`)

|Caminho|Papel|
|---|---|
|`android/rust/src/lib.rs`|ponte JNI sobre o núcleo; contrato de threads/handles no `//!` do topo|
|`app/src/main/java/io/github/pendiego/livescreen/`|`MainActivity`, `Screens` (Compose), `ViewerModel` (estado, amigos, presença), `Native` (declarações JNI), `VideoDecoder`/`AudioPlayer` (MediaCodec/AudioTrack), `Updater` (GitHub Releases, asset `livescreen-android.apk`), `Theme`|
|`app/build.gradle.kts`|`versionName`/`versionCode` vêm do `version` do `Cargo.toml` da raiz (fonte única); minSdk 26|

## Regras

- Desktop: só Windows 10 2004+ x64. A crate `windows` está fixada em `=0.62.2`: ao precisar de API nova, adicione a feature em `Cargo.toml` em vez de subir a versão.
- Código sem plataforma (protocolo, viewer, presets, amigos) vai em `livescreen-core`; o que depende de Windows fica em `src/`, o que depende de Android em `android/`. Não duplique lógica de rede entre os dois clientes.
- Protocolo: qualquer mudança incompatível em `protocol.rs` (campos obrigatórios, enum, framing) exige incrementar `PROTOCOL_VERSION`; versões diferentes são recusadas. Campos novos opcionais usam `#[serde(default)]` para não quebrar. O viewer ainda fala `LEGACY_PROTOCOL_VERSION` com hosts 0.4 (convite sem chave de sala): preserve esse caminho e a tabela de compatibilidade do `README.md` ("Versões").
- Limites de entrada da rede (`MAX_MSG`, `MAX_FRAME`, `MAX_HANDSHAKES`, `HANDSHAKE_TIMEOUT`) são fronteira de confiança: não afrouxe sem motivo.
- Captura e encoder têm fallback (WGC → DXGI, hardware → software) e overrides (`deny-graphics-capture`, `deny-dxgi`, `deny-hardware-encoder`, `deny-hardware-decoder`, `deny-zero-copy`). Mudanças nesses caminhos devem preservar os dois lados do fallback.
- Threads de captura/encoder não podem morrer por um quadro ruim (ver `SinkHandle::lock`, que recupera mutex envenenado).
- Updater: o asset `livescreen.exe` solto da release é o que o app desktop baixa e confere pelo sha256 do GitHub; o `livescreen-android.apk` solto é o do Android (`Updater.ASSET`). Não renomeie assets em `release.yml` sem ajustar `update.rs`/`Updater.kt`.
- Dependências: `reqwest`/`rustls` reaproveitam o que o iroh já traz (provider `ring`); evite adicionar crates para o que a stdlib, `windows` ou as dependências atuais resolvem.
- Novas conexões de rede externas precisam entrar na tabela "Conexões que o app faz" do `README.md`.
- Viewer desktop: decoder por hardware (`video/decoder.rs`, MFT com `IMFDXGIDeviceManager` + `Converter::new_rgb`) cai para software após 3 falhas seguidas; preservar os dois caminhos. O quadro vai à UI por textura compartilhada (`GpuOut::Shared`, `video/present.rs`) e cai para readback (`GpuOut::Readback`) quando o egui não roda em DX12, o adaptador difere, falta saída RGBA, a importação falha ou há `deny-zero-copy`; preservar os dois caminhos. `ViewerMsg::SetQuality` é pedido, não ordem: `share.rs::apply_quality_request` só aceita redução real (`preset::is_reduction`).
- Android: o Kotlin só chama `Native.stop` depois de dar join nas threads de vídeo e áudio e não chama nada depois dele; `presenceStop` só é chamado pela corrotina dona do handle. Em segundo plano o viewer pausa áudio e pede ao host (0.5) para parar o vídeo; hosts 0.4 ignoram o pedido.

### Amigos e presença

- **Invariante: nunca dois endpoints iroh vivos com a mesma `SecretKey`** (o relay derruba um deles). Toda abertura de endpoint espera o fechamento do anterior: no desktop, `take_prober` + `join_background` antes de assistir ou transmitir; no Android, `presenceJob`/`viewerStop` com `join` antes de `Native.start`/`presenceStart`.
- Uma identidade por janela: `config::identity()` reserva `identity.lock`; a segunda janela do desktop recebe chave temporária e fica sem amigos (`identity_temporary`). Arquivo ilegível (`identity.key`, `friends.json`) nunca é sobrescrito: vai para `.bad` ou a janela usa chave temporária.
- A presença usa ALPN próprio e só responde a amigos; quem foi banido da transmissão recebe `CLOSE_DENIED` com `REMOVED_REASON` (estado `Removed`). Endpoints que só discam não publicam no pkarr; só o host publica.

## Estilo

- Textos de UI, mensagens de erro, comentários e doc comments em português (pt-BR). Identificadores em inglês.
- `//!` no topo do módulo explica o porquê do desenho; comentários só para o que o código não mostra.
- Testes unitários ficam no próprio arquivo (`#[cfg(test)] mod tests`; no Android, `app/src/test`). Testes que exigem GPU, janela ou rede real são `#[ignore]` com o comando de execução no motivo ou no `//!`.
- Erros com `anyhow`; logs com `tracing` (`info!`/`warn!`), nunca `println!` fora dos modos de diagnóstico. Kotlin usa `Log` com a tag `LiveScreen`.
- `#![allow(dead_code)]` está ativo no crate: confira manualmente se código novo ficou sem uso.
- Mudou comportamento visível ao usuário (requisitos, conexões, versões, recursos)? Atualize o `README.md` no mesmo PR.

## CI

`.github/workflows/ci.yml` roda em PR e em push na `main`: `build` (`cargo build --release` + `cargo test`, windows-latest), `android` (`gradlew assembleDebug testDebugUnitTest`, ubuntu) e `changelog` (`.github/scripts/test-changelog.sh`, valida o `cliff.toml`).

## Release

1. Atualize `version` em `Cargo.toml` (o updater compara com `CARGO_PKG_VERSION`; o Android deriva `versionName`/`versionCode` dele).
2. Commit `chore(release): X.Y.Z`, tag `vX.Y.Z`, push da tag: `release.yml` compila o APK assinado (segredos `ANDROID_KEYSTORE_B64`/`ANDROID_KEYSTORE_PASSWORD`), o `.exe` e o `.zip`, publica os três e gera as notas com o `git-cliff` (`cliff.toml`).

Commits seguem Conventional Commits em português (`feat:`, `fix:`, `perf:`, `docs:`, `ci:`, `chore(release):`; escopo `android` para o app Android). As notas da release são esses commits desde a tag anterior, agrupados por tipo; itens `- ...` no corpo viram subitens e `!`/`BREAKING CHANGE:` entra em "Breaking Changes". Por isso PRs entram por rebase ou merge commit, nunca squash (o squash junta tudo num commit só e as notas perdem os itens).

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
