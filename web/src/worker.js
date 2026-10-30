import { DurableObject } from "cloudflare:workers";

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

async function iceServers(env) {
  const fallback = [{ urls: "stun:stun.cloudflare.com:3478" }];
  const keyId = env?.TURN_KEY_ID;
  const token = env?.TURN_KEY_API_TOKEN;
  if (!keyId || !token) {
    return { list: fallback, fresh: false };
  }
  try {
    const res = await fetch(`https://rtc.live.cloudflare.com/v1/turn/keys/${keyId}/credentials/generate-ice-servers`, {
      method: "POST",
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ ttl: 14400 }),
      signal: AbortSignal.timeout(3000),
    });
    if (!res.ok) return { list: fallback, fresh: false };
    const j = await res.json();
    if (!j || !j.iceServers) return { list: fallback, fresh: false };
    const list = [].concat(j.iceServers);
    const filtered = list
      .map((entry) => {
        if (!entry || !entry.urls) return entry;
        const urls = Array.isArray(entry.urls) ? entry.urls : [entry.urls];
        const cleaned = urls.filter((u) => typeof u === "string" && !/:53(\?|$)/.test(u));
        return { ...entry, urls: cleaned };
      })
      .filter((entry) => {
        if (!entry.urls) return false;
        return Array.isArray(entry.urls) ? entry.urls.length > 0 : true;
      });
    return filtered.length > 0 ? { list: filtered, fresh: true } : { list: fallback, fresh: false };
  } catch {
    return { list: fallback, fresh: false };
  }
}

export default {
  async fetch(req, env) {
    const url = new URL(req.url);
    if (url.pathname === "/") return Response.redirect("https://pendiego.github.io/LiveScreen/", 302);
    const match = url.pathname.match(/^\/ws\/(host|view)\/([0-9a-f]{32})$/);
    if (!match) {
      return env.ASSETS.fetch(req);
    }
    if (req.headers.get("Upgrade")?.toLowerCase() !== "websocket") {
      return new Response("Upgrade Required", { status: 426 });
    }
    const room = match[2];
    const id = env.ROOM.idFromName(room);
    return env.ROOM.get(id).fetch(req);
  },
};

export class Room extends DurableObject {
  constructor(ctx, env) {
    super(ctx, env);
    this.ctx = ctx;
    this.env = env;
    this.ctx.setWebSocketAutoResponse(
      new WebSocketRequestResponsePair('{"t":"ping"}', '{"t":"pong"}')
    );
  }

  async fetch(req) {
    const url = new URL(req.url);
    const match = url.pathname.match(/^\/ws\/(host|view)\/([0-9a-f]{32})$/);
    if (!match) {
      return new Response("Not Found", { status: 404 });
    }
    const role = match[1];
    const room = match[2];

    const tag = role === "host" ? "host" : "view";
    if (this.ctx.getWebSockets(tag).length >= (role === "host" ? 4 : 64)) {
      return new Response("Too Many Requests", { status: 429 });
    }

    const pair = new WebSocketPair();
    const [client, server] = Object.values(pair);

    let id;
    let tags;
    if (role === "host") {
      tags = ["host"];
    } else {
      const bytes = new Uint8Array(4);
      do {
        crypto.getRandomValues(bytes);
        id = bytesToHex(bytes);
      } while (this.ctx.getWebSockets("v:" + id).length > 0);
      tags = ["view", "v:" + id];
    }

    this.ctx.acceptWebSocket(server, tags);
    server.serializeAttachment({ role, room, id, ok: false, offered: false, at: Date.now() });
    await this._armReaper();

    if (role === "view") {
      const hosts = this.ctx.getWebSockets("host");
      const hostOk = hosts.some((h) => {
        try {
          const a = h.deserializeAttachment();
          return a && a.ok === true;
        } catch {
          return false;
        }
      });
      if (hostOk) {
        const ice = await this._iceServers();
        server.send(JSON.stringify({ t: "hello", iceServers: ice }));
      } else {
        server.send(JSON.stringify({ t: "nohost" }));
        server.close(4004, "nohost");
      }
    }

    return new Response(null, { status: 101, webSocket: client });
  }

  async webSocketMessage(ws, msg) {
    if (typeof msg !== "string" || msg.length > 65536) return;
    let data;
    try {
      data = JSON.parse(msg);
    } catch {
      return;
    }
    if (!data || typeof data !== "object" || typeof data.t !== "string") return;

    let att;
    try {
      att = ws.deserializeAttachment();
    } catch {
      return;
    }
    if (!att) return;

    if (att.role === "host") {
      if (!att.ok) {
        if (data.t !== "host" || typeof data.secret !== "string" || !/^[0-9a-f]{32}$/i.test(data.secret)) {
          return;
        }
        const secretBytes = hexToBytes(data.secret);
        const hashBuf = await crypto.subtle.digest("SHA-256", secretBytes);
        const hashHex = bytesToHex(new Uint8Array(hashBuf)).slice(0, 32);
        if (hashHex.toLowerCase() !== att.room.toLowerCase()) {
          ws.close(4003, "unauthorized");
          return;
        }
        for (const other of this.ctx.getWebSockets("host")) {
          if (other !== ws) {
            try {
              other.close(4001, "substituído");
            } catch {}
          }
        }
        att.ok = true;
        ws.serializeAttachment(att);
        ws.send(JSON.stringify({ t: "ready" }));
        return;
      }

      if (!data.id || typeof data.id !== "string" || !/^[0-9a-f]{8}$/i.test(data.id)) return;
      const viewers = this.ctx.getWebSockets("v:" + data.id.toLowerCase());
      if (data.t === "answer" && typeof data.sdp === "string") {
        for (const v of viewers) {
          try {
            v.send(JSON.stringify({ t: "answer", sdp: data.sdp }));
          } catch {}
        }
      } else if (data.t === "reject" && typeof data.reason === "string") {
        for (const v of viewers) {
          try {
            v.send(JSON.stringify({ t: "reject", reason: data.reason }));
            v.close(1000, data.reason.slice(0, 120));
          } catch {}
        }
      } else if (data.t === "close" && typeof data.reason === "string") {
        for (const v of viewers) {
          try {
            v.send(JSON.stringify({ t: "closed", reason: data.reason }));
            v.close(1000, data.reason.slice(0, 120));
          } catch {}
        }
      }
      return;
    }

    if (att.role === "view") {
      if (data.t !== "offer") return;
      if (att.offered) return;
      if (typeof data.sdp !== "string" || typeof data.proof !== "string") return;
      const name = typeof data.name === "string" ? data.name.slice(0, 64) : "";

      const hosts = this.ctx.getWebSockets("host");
      const verifiedHost = hosts.find((h) => {
        try {
          const a = h.deserializeAttachment();
          return a && a.ok === true;
        } catch {
          return false;
        }
      });
      if (!verifiedHost) {
        ws.send(JSON.stringify({ t: "nohost" }));
        return;
      }
      att.offered = true;
      ws.serializeAttachment(att);
      verifiedHost.send(
        JSON.stringify({
          t: "offer",
          id: att.id,
          sdp: data.sdp,
          proof: data.proof,
          name,
        })
      );
    }
  }

  async webSocketClose(ws, code, reason, wasClean) {
    this._handleLeave(ws);
  }

  async webSocketError(ws, error) {
    this._handleLeave(ws);
  }

  // Credencial vale 4 h (ttl 14400); reusar por 1 h garante ≥3 h de validade a quem entra.
  async _iceServers() {
    if (this.ice && Date.now() < this.ice.until) return this.ice.list;
    if (!this.icePending) {
      this.icePending = iceServers(this.env)
        .then(({ list, fresh }) => {
          if (fresh) this.ice = { list, until: Date.now() + 3600000 };
          return list;
        })
        .finally(() => { this.icePending = null; });
    }
    return this.icePending;
  }

  async _armReaper() {
    if ((await this.ctx.storage.getAlarm()) === null) await this.ctx.storage.setAlarm(Date.now() + 10000);
  }

  // Host que não se autentica em 10 s e espectador que não manda oferta em 30 s ocupam vaga à toa.
  async alarm() {
    const now = Date.now();
    let pending = false;
    for (const ws of this.ctx.getWebSockets()) {
      let a;
      try { a = ws.deserializeAttachment(); } catch { continue; }
      if (!a || (a.role === "host" ? a.ok : a.offered)) continue;
      if (now - (a.at || 0) >= (a.role === "host" ? 10000 : 30000)) {
        try { ws.close(4008, "tempo esgotado"); } catch {}
      } else {
        pending = true;
      }
    }
    if (pending) await this.ctx.storage.setAlarm(now + 10000);
  }

  _handleLeave(ws) {
    let att;
    try {
      att = ws.deserializeAttachment();
    } catch {
      return;
    }
    if (att.role === "host" && att.ok && !att.left) {
      att.left = true;
      try { ws.serializeAttachment(att); } catch {}
      for (const v of this.ctx.getWebSockets("view")) {
        try {
          const a = v.deserializeAttachment();
          if (a && a.role === "view") {
            v.send(JSON.stringify({ t: "closed", reason: "a transmissão terminou" }));
            v.close(1000, "a transmissão terminou");
          }
        } catch {}
      }
      return;
    }
    if (att.role !== "view" || !att.id || att.left) return;
    att.left = true;
    try {
      ws.serializeAttachment(att);
    } catch {}

    const hosts = this.ctx.getWebSockets("host");
    for (const h of hosts) {
      try {
        const a = h.deserializeAttachment();
        if (a && a.ok === true) {
          h.send(JSON.stringify({ t: "left", id: att.id }));
        }
      } catch {}
    }
  }
}
