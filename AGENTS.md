# LiveScreen

Compartilhamento de tela P2P para Windows: um único `livescreen.exe` (Rust, edição 2024) que transmite ou assiste uma tela/janela via iroh, sem servidor de vídeo. Visão de usuário, requisitos e conexões externas: `README.md`.

## Comandos

```powershell
cargo build --release          # exige toolchain MSVC e CMake no PATH (compila o Opus)
cargo test                     # o mesmo que o CI (windows-latest) roda
cargo test --release codec_roundtrip -- --ignored --nocapture   # GPU real
cargo test --release e2e_window -- --ignored --nocapture        # janela real; python com tkinter no PATH
.\target\release\livescreen.exe --probe-encoder | Out-String   # diagnóstico do encoder
.\target\release\livescreen.exe --selftest-capture | Out-String
```

O build release usa `windows_subsystem = "windows"`: sem console, a saída só aparece redirecionada. Logs vão para `%APPDATA%\LiveScreen\livescreen.log` (`tracing`).

## Mapa do código

| Caminho | Papel |
|---|---|
| `src/main.rs` | entrada: flags de diagnóstico, limpeza do updater, `ui::run()` |
| `src/share.rs` | lado de quem transmite: endpoint iroh, handshake, sala (limite, kick), fan-out de vídeo/áudio, troca de qualidade ao vivo |
| `src/watch.rs` | lado de quem assiste: conecta pelo ticket, recebe vídeo e áudio, publica `WatchStatus` |
| `src/protocol.rs` | wire format: ALPN, `PROTOCOL_VERSION`, mensagens JSON (`ViewerMsg`/`HostMsg`), quadros de vídeo, códigos de close |
| `src/capture/` | `wgc.rs` (Windows Graphics Capture), `dxgi.rs` (Desktop Duplication, fallback de tela), `sources.rs` (enumeração/miniaturas); `FrameSink` recebe texturas D3D11 |
| `src/video/` | `encoder.rs`/`decoder.rs` H.264 via Media Foundation, `mf.rs` helpers, `convert.rs` RGB→NV12 com escala no D3D11 Video Processor (GPU), `probe.rs` detecção do encoder em processo filho |
| `src/audio/` | `capture.rs` loopback WASAPI → Opus; `playback.rs` datagram → Opus → cpal |
| `src/preset.rs` | qualidades (480p..4K/nativa), limites H.264 nível 5.2, bitrate e recomendação |
| `src/speedtest.rs` | medição de upload contra `speed.cloudflare.com` |
| `src/update.rs` | auto-update pelo GitHub Releases com conferência de sha256 |
| `src/config.rs` | `%APPDATA%\LiveScreen\`, `config.json` e `LIVESCREEN_CAPTURE_OVERRIDES` |
| `src/ui/` | egui/eframe: `home`, `share_view`, `watch_view`, `settings` (modal), `prefs` (`ui.json`), `theme`, `toast` |
| `src/icon.rs` | ícone desenhado em código; incluído pelo `build.rs` e pela UI (fonte única) |

## Regras

- Só Windows 10 2004+ x64. A crate `windows` está fixada em `=0.62.2`: ao precisar de API nova, adicione a feature em `Cargo.toml` em vez de subir a versão.
- Protocolo: qualquer mudança incompatível em `protocol.rs` (campos obrigatórios, enum, framing) exige incrementar `PROTOCOL_VERSION`; versões diferentes são recusadas. Campos novos opcionais usam `#[serde(default)]` para não quebrar.
- Limites de entrada da rede (`MAX_MSG`, `MAX_FRAME`, `MAX_HANDSHAKES`, `HANDSHAKE_TIMEOUT`) são fronteira de confiança: não afrouxe sem motivo.
- Captura e encoder têm fallback (WGC → DXGI, hardware → software) e overrides (`deny-graphics-capture`, `deny-dxgi`, `deny-hardware-encoder`, `deny-hardware-decoder`). Mudanças nesses caminhos devem preservar os dois lados do fallback.
- Threads de captura/encoder não podem morrer por um quadro ruim (ver `SinkHandle::lock`, que recupera mutex envenenado).
- Updater: o asset `livescreen.exe` solto da release é o que o app baixa e confere pelo sha256 do GitHub. Não renomeie o asset em `release.yml` sem ajustar `update.rs`.
- Dependências: `reqwest`/`rustls` reaproveitam o que o iroh já traz (provider `ring`); evite adicionar crates para o que a stdlib, `windows` ou as dependências atuais resolvem.
- Novas conexões de rede externas precisam entrar na tabela "Conexões que o app faz" do `README.md`.
- Viewer: decoder por hardware (`video/decoder.rs`, MFT com `IMFDXGIDeviceManager` + `Converter::new_rgb`) cai para software após 3 falhas seguidas; preservar os dois caminhos. `ViewerMsg::SetQuality` é pedido, não ordem: `share.rs::apply_quality_request` só aceita redução real (`preset::is_reduction`).

## Estilo

- Textos de UI, mensagens de erro, comentários e doc comments em português (pt-BR). Identificadores em inglês.
- `//!` no topo do módulo explica o porquê do desenho; comentários só para o que o código não mostra.
- Testes unitários ficam no próprio arquivo (`#[cfg(test)] mod tests`). Testes que exigem GPU, janela ou rede real são `#[ignore]` com o comando de execução no motivo ou no `//!`.
- Erros com `anyhow`; logs com `tracing` (`info!`/`warn!`), nunca `println!` fora dos modos de diagnóstico.
- `#![allow(dead_code)]` está ativo no crate: confira manualmente se código novo ficou sem uso.

## Release

1. Atualize `version` em `Cargo.toml` (o updater compara com `CARGO_PKG_VERSION`).
2. Commit `chore(release): X.Y.Z`, tag `vX.Y.Z`, push da tag: `release.yml` compila e publica `.exe` e `.zip`.

Commits seguem Conventional Commits em português (`feat:`, `ci:`, `chore(release):`).

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
