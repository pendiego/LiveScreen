# RFC: sinalização web v3 — ECDH efêmero + híbrido pós-quântico

Status: implementado na branch `feat/web-signal-v3` (ver "Decisões na implementação" no fim; elas prevalecem sobre o texto original). Escopo principal: sinalização do espectador no navegador (`web.rs`, `host/web.rs`, `web/src/worker.js`, `web/public/v/viewer.js`). O RFC também cobre o que está em volta do link e pode anular a cifra: área de transferência, página servida, deploy e análise de tráfego. Fora do escopo: QUIC do iroh, DTLS-SRTP da mídia e chat (ver seção 10).

## 1. Como é hoje (v2)

- Link: `/v/#<sala 32 hex>.<chave 32 hex>.v2`. A chave é a `room_key` (`ROOM_KEY_LEN` = 16 bytes, `protocol.rs:24`), a mesma do convite iroh. Fica só no fragmento.
- A sala é `SHA-256(web_secret)[..16]`. O `web_secret` é novo a cada transmissão (`host/mod.rs:761`), e o Worker autentica o host por ele (`worker.js:133`).
- Offer e answer usam AES-256-GCM com chave `HKDF-SHA256(salt="livescreen/web-signal/v2", ikm=room_key, info="offer"|"answer")` (`web.rs::signal_key`). É a mesma chave para todos os espectadores até o próximo `new_invite`.
- `new_invite` (botão "Novo convite" e todo `kick`, `host/mod.rs:985`) troca a `room_key` e limpa `web_offers`.
- Replay: o host guarda os nonces de offer (`web_offers`, até `MAX_WEB_OFFERS` = 1024).
- O fingerprint DTLS vai dentro do SDP cifrado. Quem forja o SDP controla a mídia.

### Fraquezas

|#|Problema|Consequência|
|---|---|---|
|F1|Sem sigilo futuro: tudo deriva de uma chave estática que fica no link|Quem gravou a sinalização (Worker, proxy, log) e depois vê o link abre todos os offers e answers: IPs, fingerprints, nomes|
|F2|Chave simétrica compartilhada|Portador do link + Worker forjam a answer de outro espectador e trocam o fingerprint: MITM da mídia dele|
|F3|Chave igual para todos|Um espectador com acesso ao tráfego do Worker lê a sinalização dos outros|
|F4|Chave de 128 bits e nenhuma troca de chaves|Contra busca quântica (Grover), sobra margem de ~64 bits; o sigilo depende só de o link não vazar|

## 2. Quem usa e o que cada um arrisca

|Perfil|Situação típica|O que pode vazar ou dar errado|Onde o RFC trata|
|---|---|---|---|
|Host desktop (Windows)|Transmite tela cheia para amigos e copia o link no botão|Histórico da área de transferência (Win+V) e sincronização na nuvem guardam o link; a própria tela transmitida pode mostrar a janela do LiveScreen com o link|B3, 7.7|
|Host Android|`HostService` em primeiro plano; o link sai por `ClipData` (`Screens.kt:1164`)|Teclados com histórico de área de transferência; o sistema mata o processo e as chaves somem; espectadores reconectando veem "convite trocado"|B3, B19|
|Host com público grande|Posta o link num grupo ou servidor do Discord|O link vira senha conhecida por muitos; fica guardado na plataforma de mensagens para sempre|B3, seção 9|
|Espectador em navegador de computador|Abre o link numa aba|Extensões leem a página; o histórico do navegador pode guardar a URL com fragmento antes do `replaceState`; computador compartilhado|B1, B3, B15|
|Espectador no celular ou em navegador embutido (WhatsApp, Instagram, Telegram)|Toca no link dentro do app|O app vê a URL inteira; WebView antiga sem X25519 no WebCrypto; CPU fraca|B3, 4.8, seção 3|
|Espectador em rede corporativa ou escolar|Proxy com interceptação TLS|O proxy injeta JS e lê a chave: nenhuma cifra na página resolve|B15|
|Espectador removido (kick)|Tenta voltar com o mesmo link|Hoje já é barrado pela troca do convite; a v3 troca também as chaves do host|B12|
|Espectador do app (desktop ou Android, iroh)|Convite iroh com a `room_key` de 16 bytes|Não muda nada neste RFC|seção 10|
|Mantenedor / operador|Conta Cloudflare, `CLOUDFLARE_API_TOKEN`, `deploy-web.yml`|Quem controla o deploy serve outro `viewer.js` e lê as chaves de todos|B1, B14|

Consequência de projeto: contra quem **serve a página** (operador, conta comprometida, proxy que injeta JS), a cifra da página não protege. A v3 limita o dano em todos os outros cenários, e a seção 7 adiciona detecção e endurecimento para esse caso.

## 3. Primitivas e bibliotecas

|Função|Rust (host, desktop e Android)|Navegador|
|---|---|---|
|X25519|`curve25519-dalek` 5.0.0 (`MontgomeryPoint`), já na árvore pelo iroh|`x25519` do `@noble/curves`, em `web/public/v/pq.js`|
|ML-KEM-768 (FIPS 203)|`libcrux-ml-kem` 0.0.11 (Cryspen, verificado formalmente, Rust puro, só a feature `mlkem768`)|`ml_kem768` do `@noble/post-quantum`, em `web/public/v/pq.js` (B13)|
|HKDF-SHA256, SHA-256, HMAC|`ring` (já usado)|WebCrypto|
|AEAD|AES-256-GCM via `ring` (já usado)|WebCrypto `AES-GCM`|

A primeira versão deste RFC propunha `aws-lc-rs`, mas ele só é compilado no alvo Android (pelo `str0m` com `rust-crypto`); no desktop o `str0m` usa `wincrypto`. A única crate nova compilada é o `libcrux-ml-kem` com suas dependências Rust (`libcrux-sha3`, `-intrinsics`, `-platform`, `-secrets`, `-traits`); os `bindgen`/`crabgrind` que aparecem no `Cargo.lock` ficam atrás de `cfg(valgrind_ct_test)` e não compilam.

Na página, X25519 vem sempre do noble, sem caminho WebCrypto: um caminho só, determinístico para os vetores de teste e igual em todos os navegadores e WebViews. O `pq.js` tem ~90 KB (~18 KB com gzip) e só carrega para links `.v3`.

## 4. Protocolo v3

### 4.1 Material do host

A cada transmissão e a cada `new_invite` (incluindo `kick`), tudo junto:

- `web_key`: 32 bytes aleatórios (constante nova `WEB_KEY_LEN` em `web.rs`). A v3 não reaproveita a `room_key` de 16 bytes, que continua só no convite iroh. Isso corrige F4 sem mexer em `protocol.rs`.
- `hx_sk, hx_pk`: par X25519.
- `hm_dk, hm_ek`: par ML-KEM-768 (`ek` = 1184 bytes).
- `host_pk = hx_pk (32) || hm_ek (1184)`, 1216 bytes.
- `pk_hash = SHA-256("livescreen/web-host-pk/v3" || host_pk)`, **32 bytes inteiros**. Truncar em 16 deixaria a segunda pré-imagem em ~2⁶⁴ contra Grover.

As chaves privadas ficam só na memória do processo do host, são sobrescritas na troca e somem no fim da transmissão. Nunca vão para disco nem para log.

### 4.2 Link

```
/v/#<sala 32 hex>.<web_key 64 hex>.<pk_hash 64 hex>.v3
```

São cerca de 165 caracteres no fragmento. O link é copiado por botão, nunca digitado. O campo de colar da página aceita `.v3`.

### 4.3 Publicação da chave pública

O Worker **já** manda `{"t":"hello","iceServers":...}` ao espectador (`worker.js:103`). A v3 não cria outro `hello`:

- Host → Worker: `{"t":"pk","pk":"<hex host_pk>"}` depois do `ready`, em toda reconexão e em todo `new_invite`.
- O Worker aceita `pk` só de host com `ok === true` e só com exatamente 2432 hex. Guarda em `ctx.storage.put("pk", pk)`, porque o `serializeAttachment` tem limite de 2 KiB. Apaga o valor quando sai o último host autenticado. Vale o `pk` da mensagem mais recente.
- Worker → espectador: campo aditivo `pk` no `hello` que já existe (página antiga ignora). Quando o valor muda, o Worker manda `{"t":"pk","pk":...}` aos espectadores que ainda não ofertaram.
- Link `.v3` sem `pk` em 10 s → a página mostra erro. **Nunca** cai para v2 (B4).

### 4.4 Offer (espectador → host)

A página compara `SHA-256("livescreen/web-host-pk/v3" || pk)` com o `pk_hash` do link em tempo constante e recusa se diferir (B5). Depois:

```
vx_sk, vx_pk  = X25519 efêmero
ss_x          = X25519(vx_sk, hx_pk)            ; ponto de ordem baixa/saída zero → aborta (B8)
(ct_m, ss_m)  = ML-KEM-768.Encaps(hm_ek)
ctx           = SHA-256("livescreen/web-signal/v3" || sala || host_pk || vx_pk || ct_m)
ikm           = ss_m || ss_x || web_key
k_offer       = HKDF(salt = "livescreen/web-signal/v3", ikm, info = "offer" || ctx, 32)
s_chain       = HKDF(salt = "livescreen/web-signal/v3", ikm, info = "chain" || ctx, 32)
k_pre         = HKDF(salt = "livescreen/web-signal/v3/pre", ikm = web_key, info = sala, 32)
pre           = HMAC-SHA256(k_pre, 0x03 || vx_pk || ct_m)[..16]
```

- `ctx` segue o combinador do X-Wing. O segredo híbrido aguenta enquanto X25519 **ou** ML-KEM-768 aguentar.
- `web_key` no `ikm` mantém o requisito de ter o link. Só com `host_pk`, o Worker não fabrica um offer válido.
- `pre` é um filtro barato. O host confere esse HMAC antes de decapsular, e quem não tem o link não força o host a fazer as contas de ML-KEM e X25519 (B7). Ele não substitui a tag do AEAD.

Texto claro: o JSON atual (`SealedOffer { sdp, name, avatar }`) completado com espaços até o próximo múltiplo de 2 KiB (B10). O `serde_json` aceita espaço no fim.

```
sealed = hex( 0x03 || vx_pk (32) || ct_m (1088) || pre (16) || nonce (12) || AES-256-GCM(k_offer, nonce, aad = 0x03 || ctx, json) )
```

Fica em ~5 KiB de hex com o padding. Cabe em `4 * MAX_SDP` (128 KiB) e nos 64 KiB do Worker.

### 4.5 Answer (host → espectador)

O host confere tamanho, prefixo e `pre`, consome o identificador de replay, só então decapsula, recalcula `ss_x`, `ctx`, `k_offer` e abre o offer.

```
ax_sk, ax_pk = X25519 efêmero do host para este espectador
ss_a         = X25519(ax_sk, vx_pk)
k_answer     = HKDF(salt = "livescreen/web-signal/v3/answer", ikm = s_chain || ss_a, info = "answer" || ctx || ax_pk, 32)
sealed       = hex( 0x03 || ax_pk (32) || nonce (12) || AES-256-GCM(k_answer, nonce, aad = 0x03 || ctx || ax_pk, {sdp} + padding) )
```

`ss_a` dá sigilo futuro à answer mesmo que `hx_sk`/`hm_dk` vazem durante a transmissão. A answer leva o fingerprint DTLS do host, e é esse dado que protege a mídia.

### 4.6 Regras do host

- O host novo só aceita offers com prefixo `0x03`. `sealed` vazio (caminho v1 com HMAC) e v2 recebem o mesmo `Reject` que adulteração. A `room_key` nem participa da v3, então um portador do link não contorna nada mandando um formato antigo (B4).
- Ordem de verificação, da mais barata para a mais cara: tamanho → prefixo → `pre` → decapsulação → AEAD. Toda falha dá o mesmo `Reject` genérico, sem dizer a etapa (B9).
- Replay: `web_offers` guarda `SHA-256(vx_pk || ct_m)[..16]`, com o mesmo teto `MAX_WEB_OFFERS`, e é limpo no `new_invite`.
- Logs: nunca registrar link, `web_key`, `sealed`, SDP nem as chaves privadas. Hoje nenhum `info!`/`warn!` de `host/web.rs` registra link ou `sealed` (conferido); a regra entra no `AGENTS.md`.

### 4.7 Diagrama

```mermaid
sequenceDiagram
    participant H as Host (núcleo Rust)
    participant W as Worker (Room)
    participant V as Navegador
    H->>W: host {secret} → ready
    H->>W: pk {host_pk}
    V->>W: conecta /ws/view/<sala>
    W->>V: hello {iceServers, pk}
    V->>V: confere pk_hash; X25519 + Encaps; k_offer; pre
    V->>W: offer {sealed = 03‖vx_pk‖ct_m‖pre‖nonce‖cifra}
    W->>H: offer {id, sealed}
    H->>H: pre → Decaps + X25519 → abre; ax efêmero; k_answer
    H->>W: answer {id, sealed = 03‖ax_pk‖nonce‖cifra}
    W->>V: answer {sealed}
    V-->>H: DTLS-SRTP P2P com fingerprint autenticado
```

### 4.8 Erros para o usuário

Cada mensagem diz o que fazer, sem jargão, e nenhum erro de segurança tem botão de "continuar mesmo assim":

|Situação|Mensagem (rascunho)|
|---|---|
|`pk_hash` não confere ou `pk` mudou|"O convite foi trocado ou não é de quem transmite. Peça o link novo."|
|Sem `pk` em 10 s|"Quem transmite não respondeu. Confira se a transmissão ainda está no ar."|
|Navegador sem WebCrypto e sem como carregar `pq.js`|"Este navegador não consegue abrir a transmissão com segurança. Abra o link no Chrome, Edge, Firefox ou Safari atualizados."|
|`Reject` do host|motivo do host, cortado e por `textContent`, como hoje|

## 5. Compatibilidade

|Host|Página (sempre a do Worker atual)|Resultado|
|---|---|---|
|≤ 0.6 (`.v1`/`.v2`)|nova|igual hoje|
|novo (`.v3`)|nova|v3|
|novo|Worker antigo|`hello` sem `pk` → erro na página. **Deploy do Worker antes da release.**|

- `SignalOut`/`SignalIn` continuam aditivos: só ganham `SignalOut::Pk { pk }`.
- O host novo só gera e só aceita `.v3`. Não há modo misto para negociar e, portanto, nada para forçar.
- `protocol.rs` e `PROTOCOL_VERSION` ficam iguais.
- Fim dos caminhos antigos da página: o código v1/v2 de `viewer.js` sai duas versões minor depois da v3. A partir daí, links antigos mostram "quem transmite precisa atualizar o LiveScreen". O motivo é diminuir a superfície de ataque (B18).

## 6. Catálogo de brechas

Situação: **v3** = resolvida pelo protocolo; **entrega** = mitigação incluída na implementação deste RFC; **aceita** = documentada, sem solução aqui.

|#|Brecha|Quem afeta|Gravidade|Situação e tratamento|
|---|---|---|---|---|
|B1|Quem serve a página (operador, conta Cloudflare, deploy) serve outro `viewer.js` e lê as chaves|todos no navegador|crítica|**aceita + detecção**: 7.5 (sem JS de terceiros), 7.6 (deploy) e 7.8 (host confere o hash dos arquivos servidos). Detecta troca em massa, não troca dirigida a uma vítima. Para quem precisa de mais: o app (iroh).|
|B2|Script de terceiro na mesma origem da página lê `sessionStorage` (onde fica o link). A CSP permite `static.cloudflareinsights.com`|todos no navegador|alta|**entrega**: CSP própria para `/v/*` sem essa origem (7.5). [A CONFIRMAR] se o beacon é injetado hoje|
|B3|O link é credencial ao portador e vaza por mensageiros, prévia de link, histórico da área de transferência, nuvem, tela transmitida, histórico do navegador, navegador embutido|todos|alta|**v3** limita o dano à transmissão em curso e ao convite atual (sigilo futuro e troca no `new_invite`). **entrega**: área de transferência sensível (7.7) e aviso no README ("o link é a senha; troque o convite depois que todos entrarem")|
|B4|Downgrade: forçar v2/v1 ou fazer a página cair de `.v3` para `.v2`|todos|alta|**v3**: o host só aceita `0x03`, a página não volta à v2 e o `0x03` entra no AAD|
|B5|Worker troca `host_pk`|espectadores|alta|**v3**: compromisso SHA-256 completo no link, comparado em tempo constante|
|B6|Replay de offer gravado|host|média|**v3**: conjunto de hashes do material efêmero, limpo na troca do convite|
|B7|Flood de offers força o host a fazer contas pós-quânticas, ou estoura `MAX_WEB_OFFERS`|host|média|**entrega**: filtro `pre` antes da decapsulação; o Worker limita a 20 offers por sala a cada 10 s (limite novo, entra na lista do `AGENTS.md`). Portador do link ainda pode lotar o teto: **aceita** (aí a saída é "Novo convite")|
|B8|Chaves públicas inválidas (X25519 de ordem baixa, `ek` fora do módulo)|ambos|média|**entrega**: rejeitar saída X25519 toda zero (o WebCrypto já lança erro, o `aws-lc-rs` devolve erro). O `aws-lc-rs` valida `ek`; na página, conferir `ek` por decode/encode (FIPS 203 §7.2). O `ek` vem com compromisso no link, então só um host malicioso mandaria um inválido|
|B9|Oráculo de erro (qual etapa falhou)|host|baixa|**v3**: um só `Reject`, mesma ordem de checagem|
|B10|Metadados no Worker: IPs, horários, quantidade de espectadores, tamanho do SDP e do nome|todos|média|**entrega**: padding de 2 KiB esconde o tamanho. IPs, horários e contagem: **aceita** (sem TURN, a sinalização precisa ver as conexões)|
|B11|P2P expõe o IP do host aos espectadores e o dos espectadores ao host|todos|média|**aceita**: inerente sem relay de mídia; README explica|
|B12|Espectador removido volta|host|média|**v3**: `kick` → `new_invite` → novo `web_key` e novas chaves; o link antigo para de funcionar. Se alguém repassar o link novo, ele volta: **aceita** (sem identidade no navegador)|
|B13|Cadeia de suprimentos do `pq.js` (noble)|todos no navegador|alta|**entrega**: arquivo local, versão fixa, sha256 no topo, script que reproduz o pacote, revisão a cada atualização, nunca de CDN|
|B14|Deploy: `workflow_dispatch` aceita qualquer ref; o token publica em produção sem aprovação|todos no navegador|alta|**entrega**: environment `production` restrito à `main` com revisor obrigatório; token limitado a Workers da conta (7.6)|
|B15|Malware, extensão do navegador, proxy com interceptação TLS|o afetado|crítica|**aceita**: fora do alcance de qualquer cifra na página; README cita|
|B16|Busca quântica contra chaves de 128 bits|todos|média|**v3**: `web_key` e `pk_hash` com 256 bits. A `room_key` do iroh segue com 128 bits: seção 10|
|B17|Chave privada do host vaza durante uma transmissão longa|espectadores daquele período|média|answers protegidas por `ss_a`. Offers do período expostas: **aceita**; "Novo convite" troca tudo|
|B18|Código v1/v2 mantido na página para sempre|todos no navegador|baixa|**entrega**: prazo de remoção (seção 5)|
|B19|Reconexão automática depois de troca de convite ou morte do processo Android gera erro "de segurança" que treina o usuário a ignorar avisos|espectadores|baixa|**entrega**: mensagem única e acionável (4.8), sem "continuar mesmo assim"|
|B20|Segredos em log ou em disco|host|média|**entrega**: regra 4.6 + chaves só em memória|

## 7. Plano de implementação

Partes independentes, cada uma com testes no próprio arquivo.

1. **Núcleo, cripto pura** (`crates/livescreen-core/src/web.rs`): `WebKeys::generate()` (`web_key` + X25519 + ML-KEM), `pk_hash`, `viewer_url_v3`, `open_offer_v3`, `seal_answer_v3`, padding. A v2 sai do host; `open_offer`/`seal_answer` v2 só ficam enquanto algum teste ou cliente usar.
2. **Host** (`host/web.rs`, `host/mod.rs`): `WebKeys` em `Shared`, trocadas em `new_invite`; `SignalOut::Pk` após `ready`, na reconexão e na troca; regras 4.6.
3. **Worker** (`web/src/worker.js`): mensagem `pk` (validação, `ctx.storage`, limpeza), campo `pk` no `hello`, aviso `pk` aos espectadores sem oferta, limite de 20 offers / 10 s por sala. `npm run check`.
4. **Página** (`web/public/v/viewer.js`): regex `.v3` no fragmento e no campo de colar, espera do `pk`, compromisso, offer/answer v3, padding, mensagens 4.8, carregar `pq.js` só para `.v3`.
5. **CSP da página (B2)** em `web/public/_headers`: segunda regra `/v/*` com `script-src 'self'` e `connect-src 'self' wss://livescreen.pendiego.com`. Com duas políticas, o navegador aplica as duas, ou seja, a interseção. Desligar o Web Analytics para `/v/` se estiver ligado.
6. **Deploy (B14)**: `deploy-web.yml` com `environment: production` (só `main`, revisor obrigatório); token da Cloudflare com permissão só de edição de Workers.
7. **Área de transferência sensível (B3)**:
   - Desktop: no lugar de `ctx.copy_text` para o link do navegador e o convite (`src/ui/share_view.rs`), cópia Win32 com os formatos `ExcludeClipboardContentFromMonitorProcessing`, `CanIncludeInClipboardHistory` = 0 e `CanUploadToCloudClipboard` = 0. Feature `Win32_System_DataExchange` da crate `windows` fixada (regra do `AGENTS.md`). O código de amigo segue normal.
   - Android: `ClipDescription.EXTRA_IS_SENSITIVE` no `ClipData` do link (`Screens.kt:1164`; API 33+, ignorado antes).
   - Lembrete no `share_view`: o link aparece só quando o usuário pede, e a captura de tela do próprio LiveScreen continua excluída.
8. **Verificação da página pelo host (B1, detecção)**: o release grava em `livescreen.exe` e no APK o sha256 de `viewer.js`, `pq.js` e `index.html` gerados no mesmo commit. Ao ligar o link do navegador, o host baixa esses arquivos e, se não conferirem, mostra "A página do espectador está diferente da esperada" e não gera o link. Precisa de entrada na tabela "Conexões que o app faz" (mesma origem do Worker). Limite: o Worker pode servir o arquivo certo ao host e o adulterado à vítima. Por isso isto é só detecção.
9. **Teste cruzado Rust↔JS**: vetores com sementes determinísticas gerados em Rust (como `test_signal_vectors_match_webcrypto`) e conferidos por teste node no job `web`.
10. **Docs**: README (Versões, "o link é a senha", IPs no P2P, o que a cifra não cobre), `AGENTS.md` (link `.v3`, mensagem `pk`, limite de offers, regra de logs, ordem de deploy).

### Testes adversariais (obrigatórios)

|Teste|Brecha|
|---|---|
|adulterar cada campo do `sealed` (prefixo, `vx_pk`, `ct_m`, `pre`, nonce, tag) → `Reject` único|B4, B9|
|`pk` trocado no Worker → página recusa; `pk` com 1 bit diferente → recusa|B5|
|offer v2 e offer v1 (`sealed` vazio + `proof`) contra host novo → `Reject`|B4|
|mesmo offer duas vezes → segundo recusado; depois de `new_invite`, o antigo falha|B6, B12|
|offer com `pre` errado → recusado sem decapsular (contador no teste)|B7|
|`vx_pk` = zeros e pontos de ordem baixa → recusado nos dois lados|B8|
|tamanhos de `sealed` iguais para nomes de 1 e 32 caracteres|B10|
|link `.v3` com Worker sem `pk` → erro, nunca tentativa v2|B4|

Smoke: host descartável do núcleo + `wrangler dev` + página no navegador (skills `livescreen-web-e2ee-signal-smoke` e `livescreen-web-viewer-local-smoke`). Roteiro: `.v3` conecta no desktop e no celular emulado; `pk` adulterado é recusado; replay é recusado; `kick` derruba o link antigo; link `.v2` de host 0.6 conecta; área de transferência do Windows não guarda o link no Win+V.

## 8. Modelo de ameaça depois da v3

|Atacante|v2|v3|
|---|---|---|
|Worker ou rede, sem o link|não lê nem forja|não lê nem forja|
|Worker que grava tudo e vê o link depois da transmissão|lê toda a sinalização|não lê (chaves apagadas)|
|Portador do link + Worker contra outro espectador|MITM da mídia|não consegue (precisa de `hm_dk`/`hx_sk`)|
|Portador do link sozinho|entra como espectador|entra como espectador (é o modelo de convite)|
|Quântico gravando hoje|chave de 128 bits (~2⁶⁴ com Grover) + qualquer vazamento do link|precisa quebrar ML-KEM-768 **e** ter o link (`web_key` de 256 bits)|
|Quem serve a página (operador, deploy, proxy TLS)|lê tudo|lê tudo; a v3 adiciona detecção de troca em massa (7.8)|
|Malware ou extensão no dispositivo|fora do escopo|fora do escopo|

## 9. Recomendado depois (fora desta entrega)

- **Aprovar entrada**: o host vê nome e um código curto derivado de `ctx` antes de aceitar cada espectador do navegador. Resolve link repassado (B3, B12) para quem transmite para poucos.
- **Convite com validade**: `web_key` que expira em N minutos ou depois de N entradas, com "Novo convite" automático.
- **Girar `host_pk` durante a transmissão** (B17) sem mudar o link: exigiria `pk` assinado por uma chave de longo prazo comprometida no link.

## 10. Riscos que continuam

- **Mídia sem PQ**: o DTLS-SRTP negocia ECDHE clássico, e isso depende do navegador e do `str0m`/`dimpl`. DTLS 1.3 com grupos híbridos entra quando os dois lados suportarem, sem mudança no nosso protocolo.
- **iroh (desktop/Android)**: QUIC/TLS 1.3 com provider `ring`, sem `X25519MLKEM768`; o convite usa a `room_key` de 128 bits. Trocar para o provider `aws-lc-rs` e para uma chave de 256 bits é outra mudança, ligada à versão do iroh e a `PROTOCOL_VERSION`.
- **Chat**: segue pelo canal de cada transporte. Não ganha nada além do que o transporte dá.

## Decisões na implementação

- **Bibliotecas**: seção 3 (libcrux + curve25519-dalek no Rust, noble no navegador).
- **Derivação**: as duas pontas usam só HKDF completo (extract + expand), sem `prk` cru: `k_offer` e `s_chain` saem do mesmo `ikm` com `info` diferentes, e `s_chain` entra no `ikm` da answer (4.4 e 4.5 já mostram a forma final). Mesmo efeito, e cabe na API do WebCrypto, que não expõe HKDF-Expand sozinho.
- **Replay antes da decapsulação**: depois de tamanho, prefixo e `pre`, o host consome o identificador de replay antes de qualquer conta de chave pública; um cabeçalho capturado e reenviado pelo servidor de sinalização não custa ML-KEM nem X25519.
- **`pk` no Worker**: o Worker guarda o último `pk` e ignora um `pk` igual ao guardado (sem nova escrita nem novo aviso). Não há limite de frequência: só o host autenticado envia, e um limite descartaria trocas legítimas (dois `kick` seguidos).
- **Página assinada (7.8)**: em vez de hashes fixos no binário, que quebrariam hosts instalados a cada atualização da página, o host confere `web/public/v/manifest.json` assinado com a chave de release (Ed25519, `RELEASE_PUBLIC_KEY`, a mesma do updater) por `.github/scripts/sign-web.ps1`. Página N+1 assinada funciona com host N. Mismatch bloqueia o link e o host tenta de novo com backoff; endereço loopback (dev) pula a checagem. O CI (`web`) e o `deploy-web.yml` rodam `verify-manifest.mjs` antes de publicar, então uma página sem assinatura não chega ao ar pelo pipeline.
- **Deploy (B14)**: `deploy-web.yml` só roda com ref `main` e usa o environment `production` do GitHub (branch `main`, revisor obrigatório). Os segredos da Cloudflare continuam no nível do repositório e o token não foi restringido: as duas coisas exigem o valor do token e o painel da Cloudflare, fora do alcance do repositório.
