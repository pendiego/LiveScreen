<div align="center">

<img src="site/assets/logo.svg" width="96" height="96" alt="LiveScreen">

# LiveScreen

**Compartilhe a tela do Windows com seus amigos, direto de um PC para o outro.**<br>
Sem cadastro e sem servidor de vídeo no meio: quem assiste só precisa do LiveScreen e de um código de convite.

[![Versão](https://img.shields.io/github/v/release/pendiego/LiveScreen?label=vers%C3%A3o&color=C8F135)](https://github.com/pendiego/LiveScreen/releases/latest)
[![CI](https://github.com/pendiego/LiveScreen/actions/workflows/ci.yml/badge.svg)](https://github.com/pendiego/LiveScreen/actions/workflows/ci.yml)
![Windows 10 2004+ x64](https://img.shields.io/badge/Windows-10%202004%2B%20x64-C8F135)
[![Licença: MIT ou Apache-2.0](https://img.shields.io/badge/licen%C3%A7a-MIT%20ou%20Apache--2.0-555)](#licença)

**[Baixar o livescreen.exe](https://github.com/pendiego/LiveScreen/releases/download/v0.6.0-beta.1/livescreen.exe)** · [todas as versões](https://github.com/pendiego/LiveScreen/releases) · [site](https://pendiego.github.io/LiveScreen/)

<img src="site/assets/img/home.webp" width="820" alt="Tela inicial do LiveScreen, com as opções Compartilhar tela e Assistir">

</div>

## Como usar

<table>
<tr>
<th width="50%">Para compartilhar</th>
<th width="50%">Para assistir</th>
</tr>
<tr>
<td valign="top">

1. Clique em **Compartilhar tela**.
2. Clique na tela ou janela que quer mostrar.
3. Confira a qualidade e o áudio no popup e clique em **Iniciar transmissão**.
4. O código de convite já vai para a área de transferência: cole (Ctrl+V) no chat para quem vai assistir.

</td>
<td valign="top">

1. Copie o código que recebeu.
2. Cole no campo **Assistir** da tela inicial e clique em **Assistir** (ou Enter): conecta direto.
3. Com o campo vazio, **Assistir** abre a tela de convite com o código da área de transferência já preenchido; clique em **Conectar**.

Amigos na lista entram sem convite: o botão **Assistir** aparece na linha do amigo só enquanto ele está transmitindo.

</td>
</tr>
</table>

Durante a transmissão dá para trocar a qualidade, mudar o limite de pessoas da sala e remover alguém sem derrubar os outros. A janela do LiveScreen some da própria transmissão, então ela não aparece para quem assiste.

## Como funciona

```mermaid
flowchart LR
    host["Seu PC<br/>captura, codifica e envia"]
    a["Amigo 1"]
    b["Amigo 2"]
    relay["Servidor de retransmissão<br/>só sem conexão direta"]
    c["Amigo 3"]
    host -- "conexão direta criptografada" --> a
    host -- "conexão direta criptografada" --> b
    host -.-> relay -.-> c
```

Cada pessoa que assiste recebe do seu PC a própria cópia do vídeo e do áudio. Quando a conexão direta não é possível, os pacotes passam por um servidor de retransmissão, que só repassa dados criptografados de ponta a ponta.

> [!NOTE]
> O servidor de retransmissão padrão é a infraestrutura pública e gratuita da n0 (equipe do iroh). Ela é compartilhada com outros apps, [feita para desenvolvimento e testes e tem limite de taxa](https://docs.iroh.computer/add-a-relay). Com conexão direta, nada passa por ela. Quando alguém aparece como **Via servidor**, qualidades altas podem travar: baixe a qualidade da transmissão ou peça uma mais leve no menu **Qualidade** de quem assiste.

## Qualidade, sala e internet

Quem limita o tamanho da sala é o **upload** da sua internet:

| Qualidade | Upload por pessoa (máximo, com muito movimento) |
|---|---|
| 480p 30fps | 1,5 Mbps |
| 720p 30fps | 4 Mbps |
| 1080p 60fps | 8 Mbps |

> [!TIP]
> Telas paradas (slides, código, navegador) gastam bem menos que isso.

Para ajudar na escolha:

- **Upload medido:** ao abrir a escolha de tela, o app mede o seu upload (~5 s) e mostra quantas pessoas cada qualidade aguenta.
- **Qualidade recomendada:** o app marca a mais pesada que o seu PC e a sua internet sustentam para o tamanho da sala, até 1080p 60fps.
- **Limite da sala:** começa em 4 pessoas e pode ser aumentado durante a transmissão. Se o limite passar do que a internet aguenta, aparece um aviso. Quem tenta entrar com a sala cheia recebe "Recusado: sala cheia".
- **Opções conforme a fonte:** só aparecem as que cabem na tela ou janela escolhida. Por enquanto a transmissão vai até 1080p 60fps, mesmo em monitores maiores ou acima de 60 Hz.
- **PC sobrecarregado:** se o computador não der conta da qualidade, o app avisa e, com "Baixar a qualidade sozinho" ligado (padrão), reduz uma faixa sozinho, no máximo a cada 10 s. Só reduz, e só por sobrecarga do PC; a rede continua só com o bitrate adaptativo. Trocas de qualidade ou codec rodam em segundo plano e não travam a janela.
- **Sem custo de servidor:** não há servidor de vídeo para pagar. Cada pessoa recebe a própria cópia, então o upload necessário é o valor da tabela vezes o número de pessoas. Para caber mais gente, baixe a qualidade ou o limite da sala.
- **Leve para quem assiste:** o codec é escolhido automaticamente entre os que todos na sala decodificam. HEVC e AV1 só entram se todo mundo os decodifica pela placa de vídeo; senão vai H.264, que o Android sempre usa. Se o PC ou o celular de quem assiste esquentar ou travar, peça uma qualidade mais leve no menu **Qualidade**.

## Recursos

| Recurso | Como funciona |
|---|---|
| **Captura** | Windows Graphics Capture, de tela inteira ou janela. Para telas inteiras, há fallback para DXGI Desktop Duplication, que no Windows 10 é o padrão para telas (sem a borda amarela do WGC) e desenha o cursor. |
| **Vídeo** | Codificação H.264, HEVC e AV1 por hardware (NVIDIA, AMD ou Intel, via Media Foundation), negociados automaticamente entre os participantes, com fallback para codificação H.264 por software. Limitado ao nível 5.2 no H.264: lado de até 4096 px, e a "Nativa" é reduzida quando passa disso. Telas ultrawide e em pé mantêm a área de imagem da qualidade escolhida, sem virar faixa fina. |
| **Áudio** | Opus, com todo o som do computador (tela) ou só o do aplicativo (janela), e o microfone opcional (desligado por padrão, com escolha do dispositivo ao iniciar), mixados num só stream. |
| **Conexão** | Direta entre os PCs via [iroh](https://iroh.computer), com servidor de retransmissão quando a conexão direta não é possível. A conexão é criptografada de ponta a ponta. |
| **Rede** | Controle de congestionamento BBR e bitrate adaptativo: quando a fila de algum espectador cresce, o bitrate cai na hora e volta aos poucos até o teto da qualidade. Vídeo atrasado é descartado em vez de acumular. |
| **Atualização** | Pelo próprio app, com conferência do arquivo baixado. |
| **Android** | Tema escuro Sinal, fontes Space Grotesk, Inter e JetBrains Mono, ícones Lucide, popup de atualização e decodificação H.264 por hardware (com fallback para software). |
| **Interface** | Tema escuro único "Sinal" (verde-limão sobre grafite), com as fontes Space Grotesk, Inter e JetBrains Mono embutidas e ícones Lucide, anel de foco verde-limão para navegar pelo teclado e nomes acessíveis nos botões só com ícone. Quando as animações estão desligadas no Windows, o app desativa as animações de transição e de rolagem; os indicadores de carregamento continuam girando. |
| **Navegador** | "Copiar link do navegador" gera um link para assistir sem instalar nada. A página usa o tema Sinal e mostra cada etapa (conectando, esperando o host aceitar, recebendo vídeo, sinal instável, encerrado, com "Tentar de novo" nos erros). O player tem selo AO VIVO com tempo, painel de estatísticas (resolução, fps, taxa, perda, latência e caminho direto ou TURN), mudo, Picture-in-Picture e tela cheia, com controles que somem sozinhos e atalhos `M`, `P`, `I` e `F`. |

## Configurações

Abra pela engrenagem em qualquer tela ou com `Ctrl+,`: decodificar e codificar pela placa de vídeo, mostrar o vídeo direto da placa de vídeo sem cópia pelo processador, estatísticas de vídeo ao assistir (overlay com fps e ms por quadro), qualidade e áudio padrão, baixar a qualidade sozinho quando o PC não der conta, aceitar pedidos de qualidade de quem assiste, limite da sala, nome, verificação de atualização e backends de captura. Tudo fica em `ui.json`.

<div align="center">
<img src="site/assets/img/settings.webp" width="820" alt="Janela de Configurações do LiveScreen">
</div>

**Qualidade pelo espectador:** quem assiste pode pedir ao host uma qualidade mais leve (menu "Qualidade"), inclusive as faixas 240p 30, 360p 30 e 480p 15 (hosts mais antigos, que não conhecem essas faixas, recebem 480p 30). O host só aplica se o stream de fato ficar mais leve (largura, altura, fps e bitrate não sobem), no máximo um pedido a cada 5 s, e vale para a sala toda. O host pode desligar os pedidos em Configurações ("Aceitar pedidos de qualidade mais leve de quem assiste"); desligado, eles só aparecem como aviso.

## Requisitos

- **Host (quem transmite):** Windows 10 versão 2004 ou superior, 64 bits.
- **Viewer (quem assiste):** Windows 10 2004+ x64 ou Android 8.0 (API 26) ou superior. O Android não transmite.
- **Versões:** o host 0.5 só aceita viewers 0.5 (protocolo 5). Viewers 0.5.x conectam também a hosts 0.4 (protocolo 4), mas versões com protocolos incompatíveis recusam a conexão com "versão incompatível".

## Download e atualização

Baixe o `livescreen.exe` (ou o `.zip`) em [Releases](https://github.com/pendiego/LiveScreen/releases). Não precisa instalar: é só abrir.

Ao abrir, o app verifica se há versão nova. Se houver, aparece o botão **Atualizar para a versão X** no rodapé da tela inicial. Ele:

1. baixa o `livescreen.exe` da release;
2. confere o sha256 publicado pelo GitHub;
3. troca o executável e reabre o app.

> [!NOTE]
> Para atualizar, o executável precisa estar numa pasta com permissão de escrita (fora de `Program Files`, por exemplo).

### Android (só para assistir)

O app Android só **assiste**: baixe o `livescreen-android.apk` em [Releases](https://github.com/pendiego/LiveScreen/releases). Cole o código de convite e toque em **Assistir**. Ele conecta a hosts 0.5 e também a hosts 0.4 (convite sem chave de sala).

- **Bateria:** com o app em segundo plano o áudio pausa e, em hosts 0.5, o host para de enviar vídeo; tudo volta ao reabrir, sem reconectar. Hosts 0.4 não entendem esse pedido e continuam enviando vídeo. Mudo também pausa a decodificação do áudio.
- **Atualização:** o app confere a release pelo GitHub e baixa o APK novo.

## Conexões que o app faz

| Destino | Quando | Para quê |
|---|---|---|
| Servidores públicos da n0 (equipe do iroh) | ao transmitir, ao assistir e com o app aberto fora de uma transmissão quando há amigos na lista | Encontrar o outro PC e retransmitir quando não há conexão direta, e ver quais amigos estão transmitindo (uma conexão curta por amigo a cada 15 s). Os servidores repassam pacotes criptografados e não veem o vídeo. É infraestrutura pública compartilhada, para testes e com limite de taxa: não há garantia de banda quando o vídeo passa por ela. |
| `api.github.com` / `github.com` | ao abrir o app; ao clicar em Atualizar (desktop e Android) | Verificar e baixar versões novas |
| `speed.cloudflare.com` | ao abrir a escolha de tela, uma vez por execução, e ao clicar em "Medir de novo" | Medir o upload. O teste dura ~5 s e envia dados de teste: cerca de 60 MB numa fibra de 100 Mbps, com teto de 150 MB. |
| `livescreen.pendiego.com` (Cloudflare Worker) e `stun.cloudflare.com` | ao clicar em "Copiar link do navegador" durante uma transmissão | Sinalização do espectador no navegador (só o offer/answer WebRTC, sem vídeo) e descoberta do endereço público por STUN. O vídeo vai direto do PC para o navegador (WebRTC); o TURN da Cloudflare só entra se o operador do Worker o configurar. |

## Avançado

<details>
<summary><b>Arquivos em <code>%APPDATA%\LiveScreen\</code> e <code>config.json</code></b></summary>

- **`ui.json`:** preferências (qualidade, áudio, nome, volume, limite da sala, posição da janela, decodificação/codificação por hardware, estatísticas, atualização, backends de captura).
- **`livescreen.log`:** registro para diagnóstico.
- **`config.json`** (opcional): desliga partes da captura ou do encoder para contornar problemas de driver:

  ```json
  { "capture_overrides": ["deny-hardware-encoder"] }
  ```

  Os valores aceitos são `deny-graphics-capture`, `deny-dxgi`, `deny-hardware-encoder`, `deny-hardware-decoder` e `deny-zero-copy` (o viewer volta a ler o quadro pela CPU em vez de compartilhar a textura com a placa de vídeo; o overlay de estatísticas mostra `sem cópia` ou `cópia via CPU`). A variável de ambiente `LIVESCREEN_CAPTURE_OVERRIDES` (valores separados por vírgula) substitui o arquivo. O que o arquivo ou a variável desligam não pode ser religado em Configurações. A janela usa o backend DX12 do wgpu; `WGPU_BACKEND` troca o backend e, fora do DX12, o viewer usa a cópia via CPU.

</details>

<details>
<summary><b>Diagnóstico pela linha de comando</b></summary>

O `.exe` é um app de janela: para ver a saída, redirecione, ex.: `livescreen.exe --probe-encoder | Out-String` no PowerShell ou `| more` no cmd.

- `livescreen.exe --probe-encoder`: testa os encoders de placa e imprime o que funcionou;
- `livescreen.exe --selftest-capture`: captura a tela principal por 3 s com cada backend e conta os quadros.

</details>

<details>
<summary><b>Compilar</b></summary>

Requisitos: Rust estável (edição 2024), toolchain MSVC e CMake no `PATH` (usado para compilar o Opus).

```powershell
cargo build --release
.\target\release\livescreen.exe
```

O `build.rs` embute o ícone no `.exe` sem precisar de `rc.exe`.

</details>

<details>
<summary><b>Testes</b></summary>

```powershell
cargo test
```

Testes manuais (`#[ignore]`), que usam GPU e janelas reais:

```powershell
cargo test --release codec_roundtrip -- --ignored --nocapture   # codifica e decodifica na GPU
cargo test --release e2e_window -- --ignored --nocapture        # transmite uma janela tkinter; exige python com tkinter no PATH
```

</details>

## Licença

Dual: [MIT](LICENSE-MIT) ou [Apache-2.0](LICENSE-APACHE), à sua escolha.

As fontes [Space Grotesk](assets/fonts/OFL-SpaceGrotesk.txt), [Inter](assets/fonts/OFL-Inter.txt) e [JetBrains Mono](assets/fonts/OFL-JetBrainsMono.txt), embutidas no `.exe` e no app Android, são distribuídas sob a SIL Open Font License 1.1. Os ícones são do [Lucide](assets/fonts/LICENSE-Lucide.txt).
