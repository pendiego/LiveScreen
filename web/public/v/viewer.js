(() => {
  window.addEventListener("hashchange", () => location.reload());
  const hashMatch = location.hash.match(/^#([0-9a-f]{32})\.([0-9a-f]{32})$/i);

  const card = document.getElementById("card");
  const joinForm = document.getElementById("joinForm");
  const nameInput = document.getElementById("nameInput");
  const statusBox = document.getElementById("statusBox");
  const statusMsg = document.getElementById("statusMsg");
  const retryBtn = document.getElementById("retryBtn");
  const playerBox = document.getElementById("playerBox");
  const video = document.getElementById("video");
  const muteBtn = document.getElementById("muteBtn");
  const fsBtn = document.getElementById("fsBtn");

  if (!hashMatch) {
    joinForm.hidden = true;
    showStatus("Link incompleto: peça um novo link para quem está transmitindo.", false);
    return;
  }

  const roomId = hashMatch[1].toLowerCase();
  const roomKey = hashMatch[2].toLowerCase();

  const savedName = localStorage.getItem("livescreen-name") || "";
  nameInput.value = savedName.slice(0, 32);

  let ws = null;
  let pc = null;
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

  function cleanup() {
    if (pingTimer) {
      clearInterval(pingTimer);
      pingTimer = null;
    }
    if (answerTimer) {
      clearTimeout(answerTimer);
      answerTimer = null;
    }
    if (pc) {
      try { pc.close(); } catch {}
      pc = null;
    }
    if (ws) {
      try { ws.close(); } catch {}
      ws = null;
    }
  }

  function showStatus(text, canRetry) {
    statusMsg.textContent = text;
    statusBox.hidden = false;
    retryBtn.hidden = !canRetry;
  }

  function stopWithReason(reason) {
    console.log("[viewer stopWithReason]", reason);
    cleanup();
    playerBox.hidden = true;
    card.hidden = false;
    joinForm.hidden = true;
    showStatus(reason, false);
  }

  async function start(iceServers) {
    try {
      pc = new RTCPeerConnection({ iceServers });
      window.__pc = pc;

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
        if (pc && pc.connectionState === "failed") {
          stopWithReason("Não foi possível conectar: a rede pode estar bloqueando a conexão.");
        }
      };

      pc.addTransceiver("video", { direction: "recvonly" });
      pc.addTransceiver("audio", { direction: "recvonly" });

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
      }

      answerTimer = setTimeout(() => {
        stopWithReason("Sem resposta de quem transmite.");
      }, 15000);
    } catch (err) {
      stopWithReason("Erro ao iniciar conexão: " + (err?.message || err));
    }
  }

  function connect() {
    cleanup();
    statusBox.hidden = true;
    joinForm.hidden = true;

    const wsProto = location.protocol === "https:" ? "wss:" : "ws:";
    const wsUrl = `${wsProto}//${location.host}/ws/view/${roomId}`;

    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      pingTimer = setInterval(() => {
        if (ws && ws.readyState === WebSocket.OPEN) {
          ws.send(JSON.stringify({ t: "ping" }));
        }
      }, 30000);
    };

    ws.onmessage = async (evt) => {
      let data;
      try {
        data = JSON.parse(evt.data);
      } catch {
        return;
      }
      console.log("[viewer ws message]", data);

      if (data.t === "nohost") {
        cleanup();
        playerBox.hidden = true;
        card.hidden = false;
        showStatus("A transmissão não está no ar.", true);
        return;
      }

      if (data.t === "hello") {
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
            card.hidden = true;
            playerBox.hidden = false;
            video.muted = false;
            video.play().catch(() => {
              video.muted = true;
              video.play().catch(() => {});
            });
          } catch (e) {
            stopWithReason("Erro ao aplicar resposta da transmissão.");
          }
        }
        return;
      }

      if (data.t === "reject" || data.t === "closed") {
        stopWithReason(data.reason || "Conexão encerrada.");
      }
    };

    ws.onerror = () => {
      // Deixar onclose ou falha de timeout lidar com o estado
    };

    ws.onclose = () => {
      if (pingTimer) {
        clearInterval(pingTimer);
        pingTimer = null;
      }
      if (pc && !playerBox.hidden) {
        stopWithReason("a transmissão terminou");
      }
    };
  }

  joinForm.addEventListener("submit", (e) => {
    e.preventDefault();
    const name = nameInput.value.trim().slice(0, 32);
    currentName = name;
    localStorage.setItem("livescreen-name", name);
    connect();
  });

  retryBtn.addEventListener("click", () => {
    connect();
  });

  muteBtn.addEventListener("click", () => {
    video.muted = !video.muted;
    muteBtn.textContent = video.muted ? "Ativar som" : "Desativar som";
  });

  fsBtn.addEventListener("click", () => {
    if (!document.fullscreenElement) {
      if (playerBox.requestFullscreen) {
        playerBox.requestFullscreen();
      } else if (video.requestFullscreen) {
        video.requestFullscreen();
      }
    } else {
      document.exitFullscreen();
    }
  });
})();
