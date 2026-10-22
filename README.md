# LiveScreen

Compartilhe uma tela ou janela do Windows com seus amigos, direto de um PC para o outro, sem cadastro e sem servidor de vídeo no meio. Quem assiste só precisa do LiveScreen e de um código de convite.

## Como usar

**Para compartilhar**

1. Clique em **Compartilhar tela**.
2. Clique na tela ou janela que quer mostrar.
3. Confira a qualidade e o áudio no popup e clique em **Iniciar transmissão**.
4. O código de convite já vai para a área de transferência: cole (Ctrl+V) no chat para quem vai assistir.

**Para assistir**

1. Copie o código que recebeu.
2. Clique em **Assistir**: o código já entra preenchido.
3. Clique em **Conectar**.

Durante a transmissão dá para trocar a qualidade, mudar o limite de pessoas da sala e remover alguém sem derrubar os outros. A janela do LiveScreen some da própria transmissão, então ela não aparece para quem assiste.

## Qualidade, sala e internet

Cada pessoa que assiste recebe uma cópia do vídeo enviada pelo seu computador. Quem limita o tamanho da sala é o **upload** da sua internet:

| Qualidade | Upload por pessoa (máximo, com muito movimento) |
|---|---|
| 480p 30fps | 1,5 Mbps |
| 720p 30fps | 4 Mbps |
| 1080p 60fps | 8 Mbps |
| Nativa na taxa do monitor (ex.: 144 Hz) | até 40 Mbps |

Telas paradas (slides, código, navegador) gastam bem menos que isso.

Para ajudar na escolha:

- **Upload medido:** ao abrir a escolha de tela, o app mede o seu upload (~5 s) e mostra quantas pessoas cada qualidade aguenta.
- **Qualidade recomendada:** o app marca a mais pesada que o seu PC e a sua internet sustentam para o tamanho da sala, até 1080p 60fps.
- **Limite da sala:** começa em 4 pessoas e pode ser aumentado durante a transmissão. Se o limite passar do que a internet aguenta, aparece um aviso. Quem tenta entrar com a sala cheia recebe "Recusado: sala cheia".
- **Opções conforme a fonte:** vão de 480p a 4K, mas só aparecem as que cabem na tela ou janela escolhida. Monitores acima de 60 Hz ganham a opção na taxa do monitor (até 144 fps).
- **Aviso de PC sobrecarregado:** se o computador não der conta da qualidade, o app avisa e sugere uma mais leve.

## Recursos

- **Captura:** Windows Graphics Capture, de tela inteira ou janela. Para telas inteiras, há fallback para DXGI Desktop Duplication.
- **Vídeo:**
  - codificação H.264 pela placa de vídeo (NVIDIA, AMD ou Intel, via Media Foundation), com fallback para codificação por software;
  - limitado ao H.264 nível 5.2: lado de até 4096 px, e a "Nativa" é reduzida quando passa disso;
  - telas ultrawide e em pé mantêm a área de imagem da qualidade escolhida, sem virar faixa fina.
- **Áudio:** Opus, com todo o som do computador (tela) ou só o do aplicativo (janela).
- **Conexão:** direta entre os PCs via [iroh](https://iroh.computer), com servidor de retransmissão quando a conexão direta não é possível. A conexão é criptografada de ponta a ponta.
- **Atualização:** pelo próprio app, com conferência do arquivo baixado.

## Requisitos

- Windows 10 versão 2004 ou superior, 64 bits, para quem transmite e para quem assiste.
- Todos precisam estar na mesma versão do LiveScreen. Versões com protocolos diferentes recusam a conexão com "versão incompatível".

## Download e atualização

Baixe o `livescreen.exe` (ou o `.zip`) em [Releases](../../releases). Não precisa instalar: é só abrir.

Ao abrir, o app verifica se há versão nova. Se houver, aparece o botão **Atualizar para a versão X** no rodapé da tela inicial. Ele:

1. baixa o `livescreen.exe` da release;
2. confere o sha256 publicado pelo GitHub;
3. troca o executável e reabre o app.

Para atualizar, o executável precisa estar numa pasta com permissão de escrita (fora de `Program Files`, por exemplo).

## Conexões que o app faz

| Destino | Quando | Para quê |
|---|---|---|
| Servidores públicos da n0 (equipe do iroh) | ao transmitir e ao assistir | Encontrar o outro PC e retransmitir quando não há conexão direta. Os servidores repassam pacotes criptografados e não veem o vídeo. |
| `api.github.com` / `github.com` | ao abrir o app; ao clicar em Atualizar | Verificar e baixar versões novas |
| `speed.cloudflare.com` | ao abrir a escolha de tela, uma vez por execução, e ao clicar em "Medir de novo" | Medir o upload. O teste dura ~5 s e envia dados de teste: cerca de 60 MB numa fibra de 100 Mbps, com teto de 150 MB. |

## Arquivos e configuração

Ficam em `%APPDATA%\LiveScreen\`:

- **`ui.json`:** preferências (qualidade, áudio, nome, volume, limite da sala, posição da janela).
- **`livescreen.log`:** registro para diagnóstico.
- **`config.json`** (opcional): desliga partes da captura ou do encoder para contornar problemas de driver:

  ```json
  { "capture_overrides": ["deny-hardware-encoder"] }
  ```

  Os valores aceitos são `deny-graphics-capture`, `deny-dxgi` e `deny-hardware-encoder`. A variável de ambiente `LIVESCREEN_CAPTURE_OVERRIDES` (valores separados por vírgula) substitui o arquivo.

Diagnóstico pela linha de comando (o `.exe` é um app de janela: para ver a saída, redirecione, ex.: `livescreen.exe --probe-encoder | Out-String` no PowerShell ou `| more` no cmd):

- `livescreen.exe --probe-encoder`: testa os encoders de placa e imprime o que funcionou;
- `livescreen.exe --selftest-capture`: captura a tela principal por 3 s com cada backend e conta os quadros.

## Compilar

Requisitos: Rust estável (edição 2024), toolchain MSVC e CMake no `PATH` (usado para compilar o Opus).

```powershell
cargo build --release
.\target\release\livescreen.exe
```

O `build.rs` embute o ícone no `.exe` sem precisar de `rc.exe`.

## Testes

```powershell
cargo test
```

Testes manuais (`#[ignore]`), que usam GPU e janelas reais:

```powershell
cargo test --release codec_roundtrip -- --ignored --nocapture   # codifica e decodifica na GPU
cargo test --release e2e_window -- --ignored --nocapture        # transmite uma janela tkinter; exige python com tkinter no PATH
```

## Licença

Dual: [MIT](LICENSE-MIT) ou [Apache-2.0](LICENSE-APACHE), à sua escolha.
