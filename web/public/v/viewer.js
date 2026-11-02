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
  const hashMatch = link.match(/^([0-9a-f]{32})\.([0-9a-f]{32})(\.v2)?$/i);
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
  const donateRow = document.getElementById("donateRow");
  let chatCh = null;
  let myChatId = null;
  const qualityBtn = document.getElementById("qualityBtn");
  const qualityMenu = document.getElementById("qualityMenu");
  const qualityNote = document.getElementById("qualityNote");
  let hostCtl = false; // host novo: entende set_visible e set_quality pelo canal do chat
  let sentVisible = true; // o host começa considerando a aba visível
  const unmuteBtn = document.getElementById("unmuteBtn");
  const waitOverlay = document.getElementById("waitOverlay");
  const netBadge = document.getElementById("netBadge");
  const elapsed = document.getElementById("elapsed");

  if (!hashMatch) {
    // sem link na barra: aceita o link inteiro ou só "sala.chave" colado; o fragmento nunca vai ao servidor
    const codeForm = document.getElementById("codeForm");
    const codeInput = document.getElementById("codeInput");
    const codeError = document.getElementById("codeError");
    joinForm.hidden = true;
    retryBtn.hidden = true;
    statusBox.hidden = true;
    codeForm.hidden = false;
    codeForm.addEventListener("submit", (e) => {
      e.preventDefault();
      const m = codeInput.value.match(/([0-9a-f]{32}\.[0-9a-f]{32}(?:\.v2)?)\s*$/i);
      if (!m) { codeError.hidden = false; return; }
      location.hash = m[1];
    });
    return;
  }

  // sala restaurada do sessionStorage: permite largar a anterior e colar outro código
  const changeBtn = document.getElementById("changeBtn");
  changeBtn.hidden = false;
  changeBtn.addEventListener("click", () => {
    try { sessionStorage.removeItem(LINK_KEY); } catch {}
    location.reload();
  });

  const roomId = hashMatch[1].toLowerCase();
  const roomKey = hashMatch[2].toLowerCase();
  const sealedSignal = !!hashMatch[3];

  const savedName = localStorage.getItem("livescreen-name") || "";
  nameInput.value = savedName.slice(0, 32);
  const AVATAR_KEY = "livescreen.avatar";
  let chosenAvatar = 0;
  try {
    const val = parseInt(localStorage.getItem(AVATAR_KEY) || localStorage[AVATAR_KEY], 10);
    if (Number.isInteger(val) && val >= 0 && val <= 7) {
      chosenAvatar = val;
    }
  } catch {}

  const SVG_NS = "http://www.w3.org/2000/svg";
  const FNV_OFFSET = 0x811c9dc5;
  const FNV_PRIME = 0x01000193;
  const textEncoder = new TextEncoder();

  function autoAvatar(key) {
    const bytes = textEncoder.encode(key || "");
    let hash = FNV_OFFSET >>> 0;
    for (let i = 0; i < bytes.length; i++) {
      hash ^= bytes[i];
      hash = Math.imul(hash, FNV_PRIME) >>> 0;
    }
    return (hash % 7) + 1;
  }

  let avatarData = null;
  const avatarMap = new Map();
  const pendingSvgs = new Set();
  const AVATAR_NAMES = {
    1: "Gato",
    2: "Raposa",
    3: "Urso",
    4: "Coelho",
    5: "Sapo",
    6: "Polvo",
    7: "Peixe",
  };

  function renderAvatarShapes(svg, id) {
    if (!avatarData) return;
    const av = avatarMap.get(id);
    if (!av || !Array.isArray(av.shapes)) return;

    const palette = {
      b: avatarData.back || "#1C2027",
      d: avatarData.dark || "#0B0D10",
      l: avatarData.light || "#F3F5F7",
    };
    const colorOf = (ink) => (ink === "a" ? (av.accent || palette.l) : (palette[ink] || palette.b));

    for (const s of av.shapes) {
      if (!Array.isArray(s) || s.length < 2) continue;
      const type = s[0];
      const ink = s[1];
      const c = colorOf(ink);
      const v = s.slice(2);

      let el = null;
      switch (type) {
        case "circle":
          if (v.length >= 3) {
            el = document.createElementNS(SVG_NS, "circle");
            el.setAttribute("cx", String(v[0]));
            el.setAttribute("cy", String(v[1]));
            el.setAttribute("r", String(v[2]));
            el.setAttribute("fill", c);
          }
          break;
        case "ellipse":
          if (v.length >= 4) {
            el = document.createElementNS(SVG_NS, "ellipse");
            el.setAttribute("cx", String(v[0]));
            el.setAttribute("cy", String(v[1]));
            el.setAttribute("rx", String(v[2]));
            el.setAttribute("ry", String(v[3]));
            el.setAttribute("fill", c);
          }
          break;
        case "rect":
          if (v.length >= 5) {
            el = document.createElementNS(SVG_NS, "rect");
            el.setAttribute("x", String(v[0]));
            el.setAttribute("y", String(v[1]));
            el.setAttribute("width", String(v[2]));
            el.setAttribute("height", String(v[3]));
            el.setAttribute("rx", String(v[4]));
            el.setAttribute("fill", c);
          }
          break;
        case "poly":
          if (v.length >= 4) {
            el = document.createElementNS(SVG_NS, "polygon");
            el.setAttribute("points", v.join(" "));
            el.setAttribute("fill", c);
            el.setAttribute("stroke", c);
            el.setAttribute("stroke-width", "1");
            el.setAttribute("stroke-linejoin", "round");
          }
          break;
        case "line":
          if (v.length >= 5) {
            el = document.createElementNS(SVG_NS, "line");
            el.setAttribute("stroke-width", String(v[0]));
            el.setAttribute("x1", String(v[1]));
            el.setAttribute("y1", String(v[2]));
            el.setAttribute("x2", String(v[3]));
            el.setAttribute("y2", String(v[4]));
            el.setAttribute("stroke", c);
            el.setAttribute("stroke-linecap", "round");
          }
          break;
        case "pline":
          if (v.length >= 3) {
            el = document.createElementNS(SVG_NS, "polyline");
            el.setAttribute("stroke-width", String(v[0]));
            el.setAttribute("points", v.slice(1).join(" "));
            el.setAttribute("fill", "none");
            el.setAttribute("stroke", c);
            el.setAttribute("stroke-linecap", "round");
            el.setAttribute("stroke-linejoin", "round");
          }
          break;
        case "ring":
          if (v.length >= 5) {
            el = document.createElementNS(SVG_NS, "ellipse");
            el.setAttribute("stroke-width", String(v[0]));
            el.setAttribute("cx", String(v[1]));
            el.setAttribute("cy", String(v[2]));
            el.setAttribute("rx", String(v[3]));
            el.setAttribute("ry", String(v[4]));
            el.setAttribute("fill", "none");
            el.setAttribute("stroke", c);
          }
          break;
      }
      if (el) svg.append(el);
    }
  }

  function updateAvatarSvg(svg, id) {
    svg.dataset.avatarId = String(id);
    while (svg.childNodes.length > 1) {
      svg.removeChild(svg.lastChild);
    }
    const bg = svg.firstChild;
    if (bg) {
      bg.setAttribute("fill", (avatarData && avatarData.back) || "#1C2027");
    }
    renderAvatarShapes(svg, id);
  }

  function createAvatarSvg(id, size = 20, ariaLabel) {
    const svg = document.createElementNS(SVG_NS, "svg");
    svg.setAttribute("viewBox", "0 0 64 64");
    svg.setAttribute("width", String(size));
    svg.setAttribute("height", String(size));
    svg.setAttribute("class", "avatar-icon");
    if (ariaLabel) {
      svg.setAttribute("role", "img");
      svg.setAttribute("aria-label", ariaLabel);
    } else {
      svg.setAttribute("aria-hidden", "true");
    }

    const bg = document.createElementNS(SVG_NS, "circle");
    bg.setAttribute("cx", "32");
    bg.setAttribute("cy", "32");
    bg.setAttribute("r", "32");
    bg.setAttribute("fill", (avatarData && avatarData.back) || "#1C2027");
    svg.append(bg);

    if (avatarData) {
      renderAvatarShapes(svg, id);
    } else {
      svg.dataset.avatarId = String(id);
      pendingSvgs.add(svg);
    }
    return svg;
  }

  const avatarGrid = document.getElementById("avatarGrid");
  let autoPreviewSvg = null;

  function updateAutoAvatarPreview() {
    if (!autoPreviewSvg) return;
    const name = nameInput.value.trim().slice(0, 32);
    const autoId = autoAvatar(name);
    updateAvatarSvg(autoPreviewSvg, autoId);
  }

  function selectAvatar(id) {
    chosenAvatar = id;
    try { localStorage.setItem(AVATAR_KEY, String(id)); } catch {}
    if (!avatarGrid) return;
    const buttons = avatarGrid.querySelectorAll(".avatar-opt");
    for (const btn of buttons) {
      const btnId = parseInt(btn.dataset.avatarId, 10);
      const isSelected = btnId === id;
      btn.setAttribute("aria-checked", String(isSelected));
      btn.setAttribute("aria-pressed", String(isSelected));
      btn.classList.toggle("selected", isSelected);
    }
  }

  function initAvatarGrid() {
    if (!avatarGrid) return;
    const btnAuto = document.createElement("button");
    btnAuto.type = "button";
    btnAuto.className = "avatar-opt" + (chosenAvatar === 0 ? " selected" : "");
    btnAuto.dataset.avatarId = "0";
    btnAuto.setAttribute("role", "radio");
    btnAuto.setAttribute("aria-checked", String(chosenAvatar === 0));
    btnAuto.setAttribute("aria-pressed", String(chosenAvatar === 0));
    btnAuto.setAttribute("aria-label", "Avatar automático");
    btnAuto.setAttribute("title", "Avatar automático");

    const currentAutoId = autoAvatar(nameInput.value.trim().slice(0, 32));
    autoPreviewSvg = createAvatarSvg(currentAutoId, 32);
    btnAuto.append(autoPreviewSvg);

    const badge = document.createElement("span");
    badge.className = "avatar-badge";
    badge.textContent = "Auto";
    btnAuto.append(badge);

    btnAuto.addEventListener("click", () => selectAvatar(0));
    avatarGrid.append(btnAuto);

    for (let id = 1; id <= 7; id++) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "avatar-opt" + (chosenAvatar === id ? " selected" : "");
      btn.dataset.avatarId = String(id);
      btn.setAttribute("role", "radio");
      btn.setAttribute("aria-checked", String(chosenAvatar === id));
      btn.setAttribute("aria-pressed", String(chosenAvatar === id));
      const avName = AVATAR_NAMES[id] || ("#" + id);
      btn.setAttribute("aria-label", "Avatar " + avName);
      btn.setAttribute("title", "Avatar " + avName);

      const svg = createAvatarSvg(id, 32);
      btn.append(svg);

      btn.addEventListener("click", () => selectAvatar(id));
      avatarGrid.append(btn);
    }

    avatarGrid.addEventListener("keydown", (e) => {
      const buttons = Array.from(avatarGrid.querySelectorAll(".avatar-opt"));
      const idx = buttons.findIndex((b) => b === document.activeElement);
      if (idx === -1) return;
      let nextIdx = -1;
      if (e.key === "ArrowRight" || e.key === "ArrowDown") {
        nextIdx = (idx + 1) % buttons.length;
      } else if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
        nextIdx = (idx - 1 + buttons.length) % buttons.length;
      }
      if (nextIdx !== -1) {
        e.preventDefault();
        const nextBtn = buttons[nextIdx];
        nextBtn.focus();
        selectAvatar(Number(nextBtn.dataset.avatarId));
      }
    });

    nameInput.addEventListener("input", updateAutoAvatarPreview);
  }
  initAvatarGrid();

  async function loadAvatars() {
    try {
      const res = await fetch("avatars.json");
      if (!res.ok) return;
      avatarData = await res.json();
      if (avatarData && Array.isArray(avatarData.avatars)) {
        for (const av of avatarData.avatars) {
          avatarMap.set(av.id, av);
        }
      }
      for (const svg of pendingSvgs) {
        if (svg.isConnected) {
          const id = parseInt(svg.dataset.avatarId, 10);
          if (Number.isInteger(id)) {
            updateAvatarSvg(svg, id);
          }
        }
      }
      pendingSvgs.clear();
      updateAutoAvatarPreview();
    } catch {}
  }
  loadAvatars();

  let ws = null;
  let pc = null;
  let answered = false;
  let everConnected = false;
  let pingTimer = null;
  let answerTimer = null;
  let offerNonce = null;
  let currentName = "";
  let hostName = "";
  let hostAvatar = 0;
  let reconnectAttempts = 0;
  let reconnectTimer = null;
  let reconnecting = false;
  const MAX_RECONNECT_ATTEMPTS = 5;
  let pipWindow = null;
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

  async function signalKey(info) {
    const enc = new TextEncoder();
    const base = await crypto.subtle.importKey("raw", hexToBytes(roomKey), "HKDF", false, ["deriveKey"]);
    return crypto.subtle.deriveKey(
      { name: "HKDF", hash: "SHA-256", salt: enc.encode("livescreen/web-signal/v2"), info: enc.encode(info) },
      base, { name: "AES-GCM", length: 256 }, false, ["encrypt", "decrypt"]);
  }

  async function sealOffer(obj) {
    const enc = new TextEncoder();
    const nonce = crypto.getRandomValues(new Uint8Array(12));
    const ct = await crypto.subtle.encrypt(
      { name: "AES-GCM", iv: nonce, additionalData: enc.encode("offer:" + roomId) },
      await signalKey("offer"), enc.encode(JSON.stringify(obj)));
    return { nonce, sealed: bytesToHex(nonce) + bytesToHex(new Uint8Array(ct)) };
  }

  async function openAnswer(sealed, nonce) {
    if (typeof sealed !== "string" || !/^(?:[0-9a-f]{2}){28,}$/i.test(sealed) || !nonce) return null;
    const bytes = hexToBytes(sealed);
    try {
      const pt = await crypto.subtle.decrypt(
        { name: "AES-GCM", iv: bytes.slice(0, 12), additionalData: new TextEncoder().encode("answer:" + roomId + ":" + bytesToHex(nonce)) },
        await signalKey("answer"), bytes.slice(12));
      const obj = JSON.parse(new TextDecoder().decode(pt));
      return obj && typeof obj.sdp === "string" ? obj : null;
    } catch {
      return null;
    }
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

  function updateMediaMetadata() {
    if (!("mediaSession" in navigator)) return;
    try {
      navigator.mediaSession.metadata = new MediaMetadata({
        title: "Transmissão ao vivo",
        artist: hostName || "LiveScreen",
        album: "LiveScreen",
        artwork: [{ src: "logo.svg", sizes: "any", type: "image/svg+xml" }],
      });
    } catch {}
  }

  function updatePlaybackState(active = true) {
    if (!("mediaSession" in navigator)) return;
    try {
      if (!active || playerBox.hidden) {
        navigator.mediaSession.playbackState = "none";
        navigator.mediaSession.metadata = null;
        return;
      }
      navigator.mediaSession.playbackState = video.paused ? "paused" : "playing";
    } catch {}
  }

  function cleanup(isReconnect = false) {
    if (pingTimer) {
      clearInterval(pingTimer);
      pingTimer = null;
    }
    if (answerTimer) {
      clearTimeout(answerTimer);
      answerTimer = null;
    }
    offerNonce = null;
    if (elapsedTimer) {
      clearInterval(elapsedTimer);
      elapsedTimer = null;
    }
    if (idleTimer) {
      clearTimeout(idleTimer);
      idleTimer = null;
    }
    answered = false;
    if (!isReconnect) {
      everConnected = false;
      reconnecting = false;
      reconnectAttempts = 0;
      if (reconnectTimer) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
      }
    }
    stopStats();
    releaseAwake();
    hostCtl = false;
    sentVisible = true;
    qualityBtn.hidden = true;
    qualityMenu.hidden = true;
    qualityBtn.setAttribute("aria-expanded", "false");
    if (!isReconnect) {
      hostAvatar = 0;
      netBadge.hidden = true;
      updatePlaybackState(false);
    }
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
    if (reconnectTimer) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
    reconnecting = false;
    reconnectAttempts = 0;
    cleanup(false);
    if (pipWindow) {
      try { pipWindow.close(); } catch {}
      pipWindow = null;
    }
    if (document.fullscreenElement) document.exitFullscreen().catch(() => {});
    if (document.pictureInPictureElement) document.exitPictureInPicture().catch(() => {});
    playerBox.hidden = true;
    statsPanel.hidden = true;
    statsBtn.setAttribute("aria-pressed", "false");
    try { if (chatCh) chatCh.close(); } catch {}
    chatCh = null;
    myChatId = null;
    chatBtn.hidden = true;
    donateRow.hidden = true;
    chatBtn.setAttribute("aria-pressed", "false");
    delete chatBtn.dataset.unread;
    chatPanel.hidden = true;
    chatList.replaceChildren();
    card.hidden = false;
    joinForm.hidden = true;
    document.title = TITLE;
    showStatus(sentence(text), "error");
  }

  function scheduleReconnect() {
    if (reconnectTimer) return;
    if (reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
      reconnecting = false;
      reconnectAttempts = 0;
      stopWithReason("A conexão com quem transmite caiu. Tente entrar de novo.");
      return;
    }
    reconnecting = true;
    reconnectAttempts++;
    netBadge.textContent = "Reconectando…";
    netBadge.hidden = false;
    const base = Math.min(1000 * Math.pow(2, reconnectAttempts - 1), 16000);
    const delay = base + Math.random() * 1000;
    reconnectTimer = setTimeout(() => {
      reconnectTimer = null;
      connect(true);
    }, delay);
  }

  // sem TURN, só o caminho direto existe: explica o que o espectador pode tentar
  // srflx só prova que o STUN respondeu; a causa real pode ser NAT de qualquer lado, firewall ou DTLS
  function connectTips() {
    const hasSrflx = /^a=candidate:.* typ srflx/m.test(pc?.localDescription?.sdp || "");
    const base = "Não foi possível estabelecer a conexão direta; a rede de um dos lados pode estar bloqueando (comum em 4G e redes de empresa ou escola). Tente trocar entre Wi-Fi e dados móveis ou desligar a VPN; quem transmite também pode liberar o LiveScreen no firewall do Windows.";
    return hasSrflx
      ? base
      : `${base} Seu navegador também não reportou endereço público, o que pode indicar VPN, bloqueador de WebRTC ou economia de dados.`;
  }

  async function start(iceServers) {
    try {
      pc = new RTCPeerConnection({ iceServers });

      const ms = new MediaStream();
      pc.ontrack = (e) => {
        ms.addTrack(e.track);
        if (video.srcObject !== ms) {
          video.srcObject = ms;
        }
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
          const types = {};
          for (const m of (pc.localDescription?.sdp || "").matchAll(/^a=candidate:.* typ (\w+)/gm)) types[m[1]] = (types[m[1]] || 0) + 1;
          console.warn("LiveScreen: conexão falhou", { ice: pc.iceConnectionState, gathering: pc.iceGatheringState, candidatos: types });
          if (everConnected) {
            scheduleReconnect();
          } else {
            stopWithReason(`${connectTips()} (ICE ${pc.iceConnectionState}; candidatos ${JSON.stringify(types)})`);
          }
        } else if (st === "disconnected") {
          netBadge.textContent = "Sinal instável, esperando a conexão voltar…";
          netBadge.hidden = false;
        } else if (st === "connected") {
          everConnected = true;
          reconnecting = false;
          reconnectAttempts = 0;
          if (reconnectTimer) {
            clearTimeout(reconnectTimer);
            reconnectTimer = null;
          }
          netBadge.hidden = true;
          updatePlaybackState(true);
        }
      };

      const vtc = pc.addTransceiver("video", { direction: "recvonly" });
      // como no app, o host só usa AV1 se todos decodificam pela placa: sem isso, tira o AV1 da oferta
      try {
        const codecs = RTCRtpReceiver.getCapabilities("video")?.codecs || [];
        const hw = await navigator.mediaCapabilities
          ?.decodingInfo({ type: "webrtc", video: { contentType: "video/AV1", width: 1920, height: 1080, bitrate: 8e6, framerate: 60 } })
          .then((i) => i.supported && i.powerEfficient, () => false);
        const isAv1 = (c) => c.mimeType.toLowerCase() === "video/av1";
        if (codecs.length && vtc.setCodecPreferences) vtc.setCodecPreferences(hw ? [...codecs.filter(isAv1), ...codecs.filter((c) => !isAv1(c))] : codecs.filter((c) => !isAv1(c)));
      } catch (e) {
        console.warn("LiveScreen: setCodecPreferences falhou", e);
      }
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
      // (offerFp removido: a AEAD amarra a resposta ao nonce do offer)
      if (!fp) {
        stopWithReason("Não foi possível identificar a impressão digital DTLS da conexão.");
        return;
      }

      let msg;
      if (sealedSignal) {
        const s = await sealOffer({ sdp: localSdp, name: currentName, avatar: chosenAvatar });
        offerNonce = s.nonce;
        msg = { t: "offer", sealed: s.sealed };
      } else {
        msg = { t: "offer", sdp: localSdp, proof: await computeProof(roomKey, fp), name: currentName, avatar: chosenAvatar };
      }

      if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(msg));
        showStatus("Esperando quem transmite aceitar…", "busy");
      }

      answerTimer = setTimeout(() => {
        if (reconnecting) {
          scheduleReconnect();
        } else {
          stopWithReason("Sem resposta de quem transmite.");
        }
      }, 15000);
    } catch (err) {
      // pc nulo: a conexão já foi encerrada por outro caminho
      if (pc) {
        if (reconnecting) {
          scheduleReconnect();
        } else {
          stopWithReason("Erro ao iniciar conexão: " + (err?.message || err));
        }
      }
    }
  }

  function connect(isReconnect = false) {
    if (!currentName) currentName = nameInput.value.trim().slice(0, 32);
    if (!isReconnect) {
      if (reconnectTimer) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
      }
      reconnecting = false;
      reconnectAttempts = 0;
    }
    cleanup(isReconnect);
    joinForm.hidden = true;
    if (!isReconnect) {
      showStatus("Conectando…", "busy");
    } else {
      netBadge.textContent = "Reconectando…";
      netBadge.hidden = false;
    }

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
        if (reconnecting) {
          scheduleReconnect();
          return;
        }
        cleanup();
        playerBox.hidden = true;
        card.hidden = false;
        showStatus("A transmissão não está no ar. Quando quem transmite voltar a compartilhar, tente de novo.", "error");
        return;
      }

      if (data.t === "hello") {
        if (pc) return;
        if (!isReconnect) showStatus("Pedindo para entrar…", "busy");
        await start(data.iceServers || []);
        return;
      }

      if (data.t === "answer") {
        if (answerTimer) {
          clearTimeout(answerTimer);
          answerTimer = null;
        }
        if (pc) {
          let sdp = data.sdp;
          if (sealedSignal) {
            const ans = await openAnswer(data.sealed, offerNonce);
            if (!pc) return;
            if (!ans) {
              stopWithReason("A resposta não veio de quem transmite: a chave da sala não confere. Por segurança, a conexão foi recusada.");
              return;
            }
            sdp = ans.sdp;
          }
          if (typeof sdp !== "string") return;
          try {
            await pc.setRemoteDescription({ type: "answer", sdp });
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
            if (reconnecting) {
              scheduleReconnect();
            } else {
              stopWithReason("Erro ao aplicar resposta da transmissão.");
            }
          }
        }
        return;
      }

      if (data.t === "reject" || data.t === "closed") {
        let reason = typeof data.reason === "string" ? data.reason.slice(0, 200) : "";
        // o host dá o timeout de ICE antes do navegador: acrescenta a orientação
        if (!everConnected && /^não foi possível conectar/i.test(reason)) reason = connectTips();
        stopWithReason(reason);
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
      if (reconnecting) {
        scheduleReconnect();
        return;
      }
      stopWithReason(playerBox.hidden ? "A conexão com o servidor do link caiu." : "A transmissão terminou.");
    };
  }

  joinForm.addEventListener("submit", (e) => {
    e.preventDefault();
    const name = nameInput.value.trim().slice(0, 32);
    currentName = name;
    localStorage.setItem("livescreen-name", name);
    try { localStorage.setItem(AVATAR_KEY, String(chosenAvatar)); } catch {}
    connect();
  });

  retryBtn.addEventListener("click", connect);

  // Primeiro quadro: tira a espera, inicia o relógio e o título.
  video.addEventListener("playing", () => {
    updateMediaMetadata();
    updatePlaybackState(true);
    if (elapsedTimer) return;
    waitOverlay.hidden = true;
    const startedAt = performance.now();
    elapsed.textContent = "0:00";
    elapsedTimer = setInterval(() => {
      elapsed.textContent = formatElapsed(Math.floor((performance.now() - startedAt) / 1000));
    }, 1000);
    document.title = "● Ao vivo — LiveScreen";
  });
  video.addEventListener("play", () => updatePlaybackState(true));
  video.addEventListener("pause", () => updatePlaybackState(true));

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
    // a janela do Document PiP não pode ir para tela cheia e o player está nela, não em #watch
    if (pipWindow) return;
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
    if (on) {
      if (video.videoWidth > video.videoHeight && screen.orientation && screen.orientation.lock) {
        screen.orientation.lock("landscape").catch(() => {});
      }
    } else {
      if (screen.orientation && screen.orientation.unlock) {
        try { screen.orientation.unlock(); } catch {}
      }
    }
  });

  // Picture-in-Picture
  if ((window.documentPictureInPicture || document.pictureInPictureEnabled) && !video.disablePictureInPicture) {
    pipBtn.hidden = false;
  }
  async function openPip() {
    if (playerBox.hidden || pipWindow) return;
    if (document.fullscreenElement) {
      try { await document.exitFullscreen(); } catch {}
    }
    if (window.documentPictureInPicture && window.documentPictureInPicture.requestWindow) {
      try {
        const chatW = !chatPanel.hidden ? 320 : 0;
        const w = Math.max(480, Math.min(800, (video.videoWidth ? Math.round(video.videoWidth / 2) : 480) + chatW));
        const h = Math.max(300, Math.min(600, video.videoHeight ? Math.round(video.videoHeight / 2) : 360));
        const win = await window.documentPictureInPicture.requestWindow({ width: w, height: h });
        pipWindow = win;
        const link = win.document.createElement("link");
        link.rel = "stylesheet";
        link.href = new URL("viewer.css", location.href).href; // o documento PiP não herda a base da página
        win.document.head.append(link);

        const stage = win.document.createElement("div");
        stage.className = "watch pip-stage";
        stage.append(playerBox, chatPanel);
        win.document.body.append(stage);
        fsBtn.hidden = true;

        win.document.addEventListener("keydown", onKeyDown);
        win.addEventListener("pagehide", () => {
          pipWindow = null;
          fsBtn.hidden = false;
          watch.append(playerBox, chatPanel);
          sendVisibility();
        });
        sendVisibility();
        return;
      } catch (err) {
        console.warn("LiveScreen: falha ao abrir Document PiP", err);
      }
    }
    if (document.pictureInPictureElement) {
      document.exitPictureInPicture().catch(() => {});
    } else {
      video.requestPictureInPicture().catch(() => {});
    }
  }

  async function togglePip() {
    if (pipWindow) {
      pipWindow.close();
      return;
    }
    if (document.pictureInPictureElement) {
      document.exitPictureInPicture().catch(() => {});
      return;
    }
    await openPip();
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
    const remotes = new Map();
    report.forEach((r) => {
      if (r.type === "inbound-rtp") {
        bytes += r.bytesReceived || 0;
        ts = Math.max(ts, r.timestamp || 0);
        if (r.kind === "video") vid = r;
      } else if (r.type === "candidate-pair" && r.state === "succeeded" && r.nominated) {
        pair = r;
      } else if (r.type === "remote-candidate") {
        remotes.set(r.id, r);
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
    const remote = pair ? remotes.get(pair.remoteCandidateId) : null;
    const path = pair ? "Direto" + (remote && remote.address ? (remote.address.includes(":") ? " · IPv6" : " · IPv4") : "") : "—";

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
      if (Number.isInteger(data.host_avatar) && data.host_avatar >= 1 && data.host_avatar <= 7) {
        hostAvatar = data.host_avatar;
      }
      chatBtn.hidden = false;
      donateRow.hidden = data.donate !== true;
      if (data.ctl === true) {
        hostCtl = true;
        qualityBtn.hidden = false;
        sendVisibility();
      }
    } else if (data.type === "chat") {
      appendChat(data);
    }
  }

  function getLineAvatarId(m, host, mine) {
    if (Number.isInteger(m.avatar) && m.avatar >= 1 && m.avatar <= 7) {
      return m.avatar;
    }
    const name = typeof m.name === "string" ? m.name.slice(0, 32) : "";
    if (host) {
      if (Number.isInteger(hostAvatar) && hostAvatar >= 1 && hostAvatar <= 7) {
        return hostAvatar;
      }
      return autoAvatar(name || hostName || "#0");
    }
    if (mine && Number.isInteger(chosenAvatar) && chosenAvatar >= 1 && chosenAvatar <= 7) {
      return chosenAvatar;
    }
    return autoAvatar(name || (Number.isInteger(m.sender) ? "#" + m.sender : "#0"));
  }

  function appendChat(m) {
    if (typeof m.text !== "string" || (!Number.isInteger(m.sender) && m.role !== "host")) return;
    const mine = m.role === "viewer" && m.sender === myChatId;
    const host = m.role === "host";
    const donate = Number.isInteger(m.donate) && m.donate > 0;
    const name = typeof m.name === "string" ? m.name.slice(0, 32) : "";
    if (host && name && name !== hostName) {
      hostName = name;
      updateMediaMetadata();
    }
    const label = mine ? "Você" : host ? (name || "Anfitrião") : `${name || "Anônimo"} #${m.sender}`;
    const li = document.createElement("li");
    li.className = (host ? "chat-msg host" : "chat-msg") + (donate ? " donate" : "");
    const avatarId = getLineAvatarId(m, host, mine);
    const av = createAvatarSvg(avatarId, 20);
    av.classList.add("chat-avatar");
    li.append(av);
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
    if (donate) {
      const t = document.createElement("span");
      t.className = "chat-tag";
      t.textContent = "doação";
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
  donateRow.addEventListener("click", (e) => {
    const b = e.target.closest("button[data-amount]");
    if (!b || !chatCh || chatCh.readyState !== "open") return;
    chatCh.send(JSON.stringify({ type: "chat", text: chatInput.value.trim(), donate: Number(b.dataset.amount) }));
    chatInput.value = "";
  });

  // Controles que somem sozinhos
  function wake() {
    playerBox.classList.remove("idle");
    clearTimeout(idleTimer);
    if (playerBox.hidden) return;
    idleTimer = setTimeout(() => {
      if (!statsPanel.hidden || !qualityMenu.hidden || controls.matches(":focus-within") || chatPanel.matches(":focus-within")) return;
      playerBox.classList.add("idle");
    }, touchUi ? 4000 : 2500);
  }
  const controls = playerBox.querySelector(".controls");
  // Toque: mostrar os controles com um toque e escondê-los com outro; some mais devagar que com mouse.
  let touchUi = false;
  let idleAtDown = false;
  playerBox.addEventListener("pointerdown", (e) => {
    touchUi = e.pointerType === "touch";
    idleAtDown = playerBox.classList.contains("idle");
  }, true);
  ["pointermove", "pointerdown", "keydown"].forEach((ev) => playerBox.addEventListener(ev, wake));
  video.addEventListener("click", () => {
    if (touchUi && !idleAtDown) playerBox.classList.add("idle");
  });
  playerBox.addEventListener("pointerleave", (e) => {
    if (e.pointerType === "mouse") playerBox.classList.add("idle");
  });

  // Mantém a tela do celular acesa enquanto assiste.
  let wakeLock = null;
  let wakeAsk = -1; // geração com request em voo: impede pedidos simultâneos
  let wakeGen = 0; // muda em releaseAwake: pedidos em voo de sessões encerradas são soltos ao chegar
  async function holdAwake() {
    if (!navigator.wakeLock || wakeLock || wakeAsk === wakeGen || playerBox.hidden || document.visibilityState !== "visible") return;
    const gen = wakeGen;
    wakeAsk = gen;
    try {
      const s = await navigator.wakeLock.request("screen");
      if (gen !== wakeGen) {
        s.release().catch(() => {});
        return;
      }
      wakeLock = s;
      s.addEventListener("release", () => { if (wakeLock === s) wakeLock = null; });
    } catch {} finally {
      if (wakeAsk === gen) wakeAsk = -1;
    }
  }
  function releaseAwake() {
    wakeGen++;
    if (wakeLock) wakeLock.release().catch(() => {});
    wakeLock = null;
  }
  video.addEventListener("playing", holdAwake);
  document.addEventListener("visibilitychange", holdAwake);

  // Aba escondida (sem PiP): avisa o host para parar o vídeo e economizar dados e bateria; o áudio segue.
  function pageVisible() {
    return document.visibilityState === "visible" || document.pictureInPictureElement === video || !!pipWindow;
  }
  function sendVisibility() {
    if (!hostCtl || !chatCh || chatCh.readyState !== "open") return;
    const v = pageVisible();
    if (v === sentVisible) return;
    sentVisible = v;
    chatCh.send(JSON.stringify({ type: "set_visible", visible: v }));
  }
  document.addEventListener("visibilitychange", sendVisibility);
  video.addEventListener("enterpictureinpicture", sendVisibility);
  video.addEventListener("leavepictureinpicture", sendVisibility);

  // Media Session
  if ("mediaSession" in navigator && navigator.mediaSession.setActionHandler) {
    try {
      navigator.mediaSession.setActionHandler("play", () => {
        video.muted = false;
        video.play().catch(() => {});
      });
      navigator.mediaSession.setActionHandler("pause", () => {
        toggleMute();
      });
    } catch {}
    try {
      navigator.mediaSession.setActionHandler("enterpictureinpicture", () => {
        openPip();
      });
    } catch {}
  }

  // Qualidade: pedido ao host, que só aplica se for mais leve e no máximo um a cada 5 s.
  let lastQualityAt = -Infinity;
  let qualityNoteTimer = null;
  function showQualityNote(text) {
    qualityNote.textContent = text;
    qualityNote.hidden = false;
    clearTimeout(qualityNoteTimer);
    qualityNoteTimer = setTimeout(() => { qualityNote.hidden = true; }, 4000);
  }
  function setQualityMenu(open) {
    qualityMenu.hidden = !open;
    qualityBtn.setAttribute("aria-expanded", String(open));
    if (open) qualityMenu.querySelector("button").focus();
  }
  qualityBtn.addEventListener("click", () => setQualityMenu(qualityMenu.hidden));
  qualityMenu.addEventListener("click", (e) => {
    const b = e.target.closest("button[data-preset]");
    if (!b) return;
    setQualityMenu(false);
    qualityBtn.focus();
    if (!chatCh || chatCh.readyState !== "open") return;
    if (performance.now() - lastQualityAt < 5000) {
      showQualityNote("Espere alguns segundos antes de pedir de novo.");
      return;
    }
    lastQualityAt = performance.now();
    chatCh.send(JSON.stringify({ type: "set_quality", preset: b.dataset.preset }));
    showQualityNote(`Pedido enviado: ${b.textContent}. O host aplica se for mais leve que a atual.`);
  });
  qualityMenu.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      e.stopPropagation();
      setQualityMenu(false);
      qualityBtn.focus();
    }
  });
  document.addEventListener("pointerdown", (e) => {
    if (!qualityMenu.hidden && !qualityMenu.contains(e.target) && !qualityBtn.contains(e.target)) setQualityMenu(false);
  });

  // Atalhos
  function onKeyDown(e) {
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
  }
  document.addEventListener("keydown", onKeyDown);
})();
