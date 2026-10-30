(() => {
  window.addEventListener("hashchange", () => location.reload());
  const LINK_KEY = "livescreen-room";
  let link = location.hash.slice(1);
  if (link) {
    // tira a chave da barra de endereço e do histórico; o analytics roda depois deste script
    history.replaceState(null, "", location.pathname + location.search);
  } else {
    try { link = sessionStorage.getItem(LINK_KEY) || ""; } catch { link = ""; }
  }
  const hashMatch = link.match(/^([0-9a-f]{32})\.([0-9a-f]{32})$/i);
  if (hashMatch) { try { sessionStorage.setItem(LINK_KEY, link); } catch {} }

  const card = document.getElementById("card");
  const joinForm = document.getElementById("joinForm");
  const nameInput = document.getElementById("nameInput");
  const statusBox = document.getElementById("statusBox");
  const statusMsg = document.getElementById("statusMsg");
  const retryBtn = document.getElementById("retryBtn");
  const playerBox = document.getElementById("playerBox");
  const watch = document.getElementById("watch");
  const video = document.getElementById("video");
  const cardLead = document.getElementById("cardLead");
  const muteBtn = document.getElementById("muteBtn");
  const fsBtn = document.getElementById("fsBtn");
  const pipBtn = document.getElementById("pipBtn");
  const statsBtn = document.getElementById("statsBtn");
  const statsPanel = document.getElementById("statsPanel");
  const chatBtn = document.getElementById("chatBtn");
  const chatPanel = document.getElementById("chatPanel");
  const chatList = document.getElementById("chatList");
  const chatForm = document.getElementById("chatForm");
  const chatInput = document.getElementById("chatInput");
  let chatCh = null;
  let myChatId = null;
  const unmuteBtn = document.getElementById("unmuteBtn");
  const waitOverlay = document.getElementById("waitOverlay");
  const netBadge = document.getElementById("netBadge");
  const elapsed = document.getElementById("elapsed");

  if (!hashMatch) {
    joinForm.hidden = true;
    retryBtn.hidden = true;
    showStatus("Link incompleto: peça um novo link para quem está transmitindo.", "info");
    cardLead.hidden = true;
    return;
  }

  const roomId = hashMatch[1].toLowerCase();
  const roomKey = hashMatch[2].toLowerCase();

  const savedName = localStorage.getItem("livescreen-name") || "";
  nameInput.value = savedName.slice(0, 32);

  let ws = null;
  let pc = null;
  let answered = false;
  let everConnected = false;
  let pingTimer = null;
  let answerTimer = null;
  let currentName = "";

  function hexToBytes(hex) {
    const bytes = new Uint8Array(hex.length / 2);
    for (let i = 0; i < bytes.length; i++) {
      bytes[i] = parseInt(hex.substr(i * 2, 2), 16);
    }
    return bytes;
  }

  function bytesToHex(bytes) {
    let s = "";
    for (let i = 0; i < bytes.length; i++) {
      s += bytes[i].toString(16).padStart(2, "0");
    }
    return s;
  }

  function sdpFingerprint(sdp) {
    for (const line of sdp.split("\n")) {
      const trimmed = line.trim();
      if (trimmed.startsWith("a=fingerprint:")) {
        return trimmed.slice("a=fingerprint:".length).trim();
      }
    }
    return null;
  }

  async function computeProof(keyHex, fingerprint) {
    const rawKey = hexToBytes(keyHex);
    const cryptoKey = await crypto.subtle.importKey(
      "raw",
      rawKey,
      { name: "HMAC", hash: "SHA-256" },
      false,
      ["sign"]
    );
    const data = new TextEncoder().encode("livescreen/web/v1" + fingerprint);
    const sig = await crypto.subtle.sign("HMAC", cryptoKey, data);
    return bytesToHex(new Uint8Array(sig));
  }

  function addOpusStereo(sdp) {
    const match = sdp.match(/^a=rtpmap:(\d+)\s+opus\/48000\/2/mi);
    if (!match) return sdp;
    const pt = match[1];
    const fmtpRegex = new RegExp(`^(a=fmtp:${pt}\\b[^\r\n]*)`, "m");
    if (fmtpRegex.test(sdp)) {
      return sdp.replace(fmtpRegex, "$1;stereo=1");
    }
    const rtpmapRegex = new RegExp(`^(a=rtpmap:${pt}\\s+opus\/48000\/2[^\r\n]*)`, "mi");
    return sdp.replace(rtpmapRegex, `$1\r\na=fmtp:${pt} stereo=1`);
  }

  const TITLE = "LiveScreen — Assistir transmissão";
  const nf = new Intl.NumberFormat("pt-BR", { maximumFractionDigits: 1 });
  let elapsedTimer = null;
  let statsTimer = null;
  let idleTimer = null;
  let prevBytes = null;
  let prevTs = null;

  function sentence(s) {
    s = String(s || "").trim();
    if (!s) return "Conexão encerrada.";
    s = s.charAt(0).toUpperCase() + s.slice(1);
    return /[.!?:]$/.test(s) ? s : s + ".";
  }

  function formatElapsed(sec) {
    const h = Math.floor(sec / 3600);
    const m = Math.floor((sec % 3600) / 60);
    const s = String(sec % 60).padStart(2, "0");
    return h > 0 ? `${h}:${String(m).padStart(2, "0")}:${s}` : `${m}:${s}`;
  }

  function stopStats() {
    if (statsTimer) {
      clearInterval(statsTimer);
      statsTimer = null;
    }
    prevBytes = null;
    prevTs = null;
    if (statsDd) statsDd.forEach((d) => { d.textContent = "—"; });
  }

  function cleanup() {
    if (pingTimer) {
      clearInterval(pingTimer);
      pingTimer = null;
    }
    if (answerTimer) {
      clearTimeout(answerTimer);
      answerTimer = null;
    }
    if (elapsedTimer) {
      clearInterval(elapsedTimer);
      elapsedTimer = null;
    }
    if (idleTimer) {
      clearTimeout(idleTimer);
      idleTimer = null;
    }
    answered = false;
    everConnected = false;
    stopStats();
    netBadge.hidden = true;
    if (pc) {
      try { pc.close(); } catch {}
      pc = null;
    }
    if (ws) {
      const old = ws;
      ws = null;
      try { old.close(); } catch {}
    }
  }

  function showStatus(text, kind) {
    statusMsg.textContent = text;
    statusBox.dataset.kind = kind;
    statusBox.hidden = false;
    retryBtn.hidden = kind !== "error";
    cardLead.hidden = kind !== "info";
  }

  function stopWithReason(text) {
    cleanup();
    if (document.fullscreenElement) document.exitFullscreen().catch(() => {});
    if (document.pictureInPictureElement) document.exitPictureInPicture().catch(() => {});
    playerBox.hidden = true;
    statsPanel.hidden = true;
    statsBtn.setAttribute("aria-pressed", "false");
    try { if (chatCh) chatCh.close(); } catch {}
    chatCh = null;
    myChatId = null;
    chatBtn.hidden = true;
    chatBtn.setAttribute("aria-pressed", "false");
    delete chatBtn.dataset.unread;
    chatPanel.hidden = true;
    chatList.replaceChildren();
    card.hidden = false;
    joinForm.hidden = true;
    document.title = TITLE;
    showStatus(sentence(text), "error");
  }

  async function start(iceServers) {
    try {
      pc = new RTCPeerConnection({ iceServers });

      const ms = new MediaStream();
      video.srcObject = ms;

      pc.ontrack = (e) => {
        ms.addTrack(e.track);
        if (e.receiver && "jitterBufferTarget" in e.receiver) {
          try {
            e.receiver.jitterBufferTarget = 0;
          } catch {}
        }
      };

      pc.onconnectionstatechange = () => {
        if (!pc) return;
        const st = pc.connectionState;
        if (st === "failed") {
          stopWithReason(everConnected ? "A conexão com quem transmite caiu." : "Não foi possível conectar: a rede pode estar bloqueando a conexão direta.");
        } else if (st === "disconnected") {
          netBadge.hidden = false;
        } else if (st === "connected") {
          everConnected = true;
          netBadge.hidden = true;
        }
      };

      pc.addTransceiver("video", { direction: "recvonly" });
      pc.addTransceiver("audio", { direction: "recvonly" });

      chatCh = pc.createDataChannel("chat");
      chatCh.onmessage = onChatMessage;

      const offer = await pc.createOffer();
      const patchedSdp = addOpusStereo(offer.sdp);
      await pc.setLocalDescription({ type: "offer", sdp: patchedSdp });

      if (pc.iceGatheringState !== "complete") {
        await new Promise((resolve) => {
          const t = setTimeout(resolve, 3000);
          const onGather = () => {
            if (pc.iceGatheringState === "complete") {
              clearTimeout(t);
              pc.removeEventListener("icegatheringstatechange", onGather);
              resolve();
            }
          };
          pc.addEventListener("icegatheringstatechange", onGather);
        });
      }

      const localSdp = pc.localDescription?.sdp || patchedSdp;
      const fp = sdpFingerprint(localSdp);
      if (!fp) {
        stopWithReason("Não foi possível identificar a impressão digital DTLS da conexão.");
        return;
      }

      const proof = await computeProof(roomKey, fp);

      if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({
          t: "offer",
          sdp: localSdp,
          proof,
          name: currentName,
        }));
        showStatus("Esperando quem transmite aceitar…", "busy");
      }

      answerTimer = setTimeout(() => {
        stopWithReason("Sem resposta de quem transmite.");
      }, 15000);
    } catch (err) {
      // pc nulo: a conexão já foi encerrada por outro caminho
      if (pc) stopWithReason("Erro ao iniciar conexão: " + (err?.message || err));
    }
  }

  function connect() {
    cleanup();
    joinForm.hidden = true;
    showStatus("Conectando…", "busy");

    const wsProto = location.protocol === "https:" ? "wss:" : "ws:";
    const wsUrl = `${wsProto}//${location.host}/ws/view/${roomId}`;

    const sock = new WebSocket(wsUrl);
    ws = sock;

    sock.onopen = () => {
      pingTimer = setInterval(() => {
        if (ws === sock && sock.readyState === WebSocket.OPEN) {
          sock.send(JSON.stringify({ t: "ping" }));
        }
      }, 30000);
    };

    sock.onmessage = async (evt) => {
      if (ws !== sock) return;
      let data;
      try {
        data = JSON.parse(evt.data);
      } catch {
        return;
      }

      if (data.t === "nohost") {
        cleanup();
        playerBox.hidden = true;
        card.hidden = false;
        showStatus("A transmissão não está no ar. Quando quem transmite voltar a compartilhar, tente de novo.", "error");
        return;
      }

      if (data.t === "hello") {
        if (pc) return;
        showStatus("Pedindo para entrar…", "busy");
        await start(data.iceServers || []);
        return;
      }

      if (data.t === "answer") {
        if (answerTimer) {
          clearTimeout(answerTimer);
          answerTimer = null;
        }
        if (pc && typeof data.sdp === "string") {
          try {
            await pc.setRemoteDescription({ type: "answer", sdp: data.sdp });
            answered = true;
            card.hidden = true;
            playerBox.hidden = false;
            waitOverlay.hidden = false;
            video.muted = false;
            video.play().catch(() => {
              video.muted = true;
              unmuteBtn.hidden = false;
              video.play().catch(() => {});
            });
            wake();
          } catch (e) {
            stopWithReason("Erro ao aplicar resposta da transmissão.");
          }
        }
        return;
      }

      if (data.t === "reject" || data.t === "closed") {
        stopWithReason(typeof data.reason === "string" ? data.reason.slice(0, 200) : "");
      }
    };

    sock.onerror = () => {
      // onclose trata o estado
    };

    sock.onclose = () => {
      if (ws !== sock) return;
      ws = null;
      if (pingTimer) { clearInterval(pingTimer); pingTimer = null; }
      // depois da resposta o vídeo é P2P: a sinalização pode cair sem encerrar
      if (answered && pc) return;
      stopWithReason(playerBox.hidden ? "A conexão com o servidor do link caiu." : "A transmissão terminou.");
    };
  }

  joinForm.addEventListener("submit", (e) => {
    e.preventDefault();
    const name = nameInput.value.trim().slice(0, 32);
    currentName = name;
    localStorage.setItem("livescreen-name", name);
    connect();
  });

  retryBtn.addEventListener("click", connect);

  // Primeiro quadro: tira a espera, inicia o relógio e o título.
  video.addEventListener("playing", () => {
    if (elapsedTimer) return;
    waitOverlay.hidden = true;
    const startedAt = performance.now();
    elapsed.textContent = "0:00";
    elapsedTimer = setInterval(() => {
      elapsed.textContent = formatElapsed(Math.floor((performance.now() - startedAt) / 1000));
    }, 1000);
    document.title = "● Ao vivo — LiveScreen";
  });

  // Mudo
  function toggleMute() {
    video.muted = !video.muted;
  }
  muteBtn.addEventListener("click", toggleMute);
  video.addEventListener("volumechange", () => {
    muteBtn.dataset.state = video.muted ? "off" : "on";
    muteBtn.setAttribute("aria-label", video.muted ? "Ativar som" : "Desativar som");
    if (!video.muted) unmuteBtn.hidden = true;
  });
  unmuteBtn.addEventListener("click", () => {
    video.muted = false;
    video.play().catch(() => {});
    unmuteBtn.hidden = true;
  });

  // Tela cheia
  function toggleFullscreen() {
    if (document.fullscreenElement) {
      document.exitFullscreen();
    } else if (watch.requestFullscreen) {
      watch.requestFullscreen().catch(() => {});
    } else if (video.webkitEnterFullscreen) {
      video.webkitEnterFullscreen();
    }
  }
  fsBtn.addEventListener("click", toggleFullscreen);
  video.addEventListener("dblclick", toggleFullscreen);
  document.addEventListener("fullscreenchange", () => {
    const on = !!document.fullscreenElement;
    fsBtn.dataset.state = on ? "on" : "off";
    fsBtn.setAttribute("aria-label", on ? "Sair da tela cheia" : "Tela cheia");
  });

  // Picture-in-Picture
  if (document.pictureInPictureEnabled && !video.disablePictureInPicture) pipBtn.hidden = false;
  function togglePip() {
    if (document.pictureInPictureElement) {
      document.exitPictureInPicture().catch(() => {});
    } else {
      video.requestPictureInPicture().catch(() => {});
    }
  }
  pipBtn.addEventListener("click", togglePip);

  // Estatísticas
  const STAT_LABELS = ["Resolução", "Quadros", "Taxa", "Perda", "Latência", "Caminho"];
  let statsDd = null;
  let statsBusy = false;
  function renderStats(values) {
    if (!statsDd) {
      statsDd = STAT_LABELS.map((label) => {
        const a = document.createElement("dt");
        a.textContent = label;
        const b = document.createElement("dd");
        statsPanel.append(a, b);
        return b;
      });
    }
    values.forEach((v, i) => { if (statsDd[i].textContent !== v) statsDd[i].textContent = v; });
  }

  async function refreshStats() {
    if (!pc || statsBusy) return;
    statsBusy = true;
    try {
      await collectStats();
    } finally {
      statsBusy = false;
    }
  }

  async function collectStats() {
    if (!pc) return;
    let report;
    try {
      report = await pc.getStats();
    } catch {
      return;
    }
    let vid = null;
    let bytes = 0;
    let ts = 0;
    let pair = null;
    const locals = new Map();
    report.forEach((r) => {
      if (r.type === "inbound-rtp") {
        bytes += r.bytesReceived || 0;
        ts = Math.max(ts, r.timestamp || 0);
        if (r.kind === "video") vid = r;
      } else if (r.type === "candidate-pair" && r.state === "succeeded" && r.nominated) {
        pair = r;
      } else if (r.type === "local-candidate") {
        locals.set(r.id, r);
      }
    });

    let rate = "—";
    if (prevBytes !== null && ts > prevTs) {
      const kbit = ((bytes - prevBytes) * 8) / (ts - prevTs);
      rate = kbit >= 1000 ? `${nf.format(kbit / 1000)} Mbit/s` : `${nf.format(kbit)} kbit/s`;
    }
    prevBytes = bytes;
    prevTs = ts;

    const res = vid && vid.frameWidth ? `${vid.frameWidth}×${vid.frameHeight}` : "—";
    const fps = vid && vid.framesPerSecond != null ? `${nf.format(vid.framesPerSecond)} fps` : "—";
    let loss = "—";
    if (vid && vid.packetsLost != null && vid.packetsReceived != null) {
      const total = vid.packetsLost + vid.packetsReceived;
      loss = `${nf.format(total ? (vid.packetsLost / total) * 100 : 0)} %`;
    }
    const rtt = pair && pair.currentRoundTripTime != null ? `${nf.format(pair.currentRoundTripTime * 1000)} ms` : "—";
    const local = pair ? locals.get(pair.localCandidateId) : null;
    const path = local && local.candidateType ? (local.candidateType === "relay" ? "Via TURN" : "Direto") : "—";

    renderStats([res, fps, rate, loss, rtt, path]);
  }

  function toggleStats() {
    const open = statsPanel.hidden;
    statsPanel.hidden = !open;
    statsBtn.setAttribute("aria-pressed", String(open));
    stopStats();
    if (open) {
      refreshStats();
      statsTimer = setInterval(refreshStats, 1000);
    }
  }
  statsBtn.addEventListener("click", toggleStats);

  function onChatMessage(evt) {
    let data;
    try { data = JSON.parse(evt.data); } catch { return; }
    if (!data || typeof data !== "object") return;
    if (data.type === "chat_ready") {
      if (!Number.isInteger(data.chat_id)) return;
      myChatId = data.chat_id;
      chatBtn.hidden = false;
    } else if (data.type === "chat") {
      appendChat(data);
    }
  }

  function appendChat(m) {
    if (typeof m.text !== "string" || (!Number.isInteger(m.sender) && m.role !== "host")) return;
    const mine = m.role === "viewer" && m.sender === myChatId;
    const host = m.role === "host";
    const name = typeof m.name === "string" ? m.name.slice(0, 32) : "";
    const label = mine ? "Você" : host ? (name || "Anfitrião") : `${name || "Anônimo"} #${m.sender}`;
    const li = document.createElement("li");
    li.className = host ? "chat-msg host" : "chat-msg";
    const who = document.createElement("span");
    who.className = "who";
    who.textContent = label;
    li.append(who);
    const tag = host ? "anfitrião" : m.friend ? "amigo" : null;
    if (tag) {
      const t = document.createElement("span");
      t.className = "chat-tag";
      t.textContent = tag;
      li.append(t);
    }
    const text = document.createElement("span");
    text.className = "text";
    text.textContent = " " + m.text.slice(0, 500);
    li.append(text);
    const atEnd = chatList.scrollHeight - chatList.scrollTop - chatList.clientHeight <= 40;
    chatList.append(li);
    while (chatList.children.length > 200) chatList.firstElementChild.remove();
    if (atEnd) chatList.scrollTop = chatList.scrollHeight;
    if (chatPanel.hidden && !mine) chatBtn.dataset.unread = "true";
  }

  function toggleChat() {
    const open = chatPanel.hidden;
    chatPanel.hidden = !open;
    chatBtn.setAttribute("aria-pressed", String(open));
    if (open) {
      delete chatBtn.dataset.unread;
      chatList.scrollTop = chatList.scrollHeight;
      chatInput.focus();
    }
  }
  chatBtn.addEventListener("click", toggleChat);
  document.getElementById("chatClose").addEventListener("click", toggleChat);
  chatInput.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && !chatPanel.hidden) toggleChat();
  });
  chatForm.addEventListener("submit", (e) => {
    e.preventDefault();
    const text = chatInput.value.trim();
    if (text && chatCh && chatCh.readyState === "open") {
      chatCh.send(JSON.stringify({ type: "chat", text }));
      chatInput.value = "";
    }
  });

  // Controles que somem sozinhos
  function wake() {
    playerBox.classList.remove("idle");
    clearTimeout(idleTimer);
    if (playerBox.hidden) return;
    idleTimer = setTimeout(() => {
      if (!statsPanel.hidden || controls.matches(":focus-within") || chatPanel.matches(":focus-within")) return;
      playerBox.classList.add("idle");
    }, 2500);
  }
  const controls = playerBox.querySelector(".controls");
  ["pointermove", "pointerdown", "keydown"].forEach((ev) => playerBox.addEventListener(ev, wake));
  playerBox.addEventListener("pointerleave", (e) => {
    if (e.pointerType === "mouse") playerBox.classList.add("idle");
  });

  // Atalhos
  document.addEventListener("keydown", (e) => {
    if (playerBox.hidden || e.ctrlKey || e.metaKey || e.altKey) return;
    if (e.target instanceof Element && e.target.closest("input")) return;
    const k = e.key.toLowerCase();
    if (k === "m") toggleMute();
    else if (k === "f") toggleFullscreen();
    else if (k === "p" && !pipBtn.hidden) togglePip();
    else if (k === "i") toggleStats();
    else if (k === "c" && !chatBtn.hidden) toggleChat();
    else return;
    e.preventDefault();
  });
})();
