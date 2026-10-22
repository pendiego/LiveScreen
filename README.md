# LiveScreen

Compartilhe uma tela ou janela do Windows com até 4 pessoas, direto (P2P via [iroh](https://iroh.computer)), sem cadastro. Quem assiste cola um código de convite curto.

## Recursos

- Captura por Windows Graphics Capture (com fallback GDI), tela inteira ou janela.
- Codificação H.264 por hardware (Media Foundation), áudio Opus (sistema ou só do app).
- Conexão direta com fallback por servidor de retransmissão.
- A janela do LiveScreen some da própria transmissão enquanto você compartilha.
- Lembra tamanho e posição da janela.

## Requisitos

- Windows 10 (2004) ou superior, x64.
- Rust estável (edição 2024) e CMake no `PATH` (dependência `opus`).

## Download

Binários `.exe` ficam em [Releases](../../releases).

## Compilar e executar

```powershell
cargo build --release
.\target\release\livescreen.exe
```

## Testes

```powershell
cargo test
```

Os testes `#[ignore]` (`e2e_*`) abrem janelas reais e exigem `python` com tkinter no `PATH`: `cargo test -- --ignored`.

## Licença

Dual: [MIT](LICENSE-MIT) ou [Apache-2.0](LICENSE-APACHE), à sua escolha.
