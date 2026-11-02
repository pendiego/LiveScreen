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

const ICE_SERVERS = [{ urls: "stun:stun.cloudflare.com:3478" }];

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
    // a página é servida por este mesmo Worker; host nativo não manda Origin
    const origin = req.headers.get("Origin");
    if (origin && origin !== url.origin) return new Response("Forbidden", { status: 403 });
    const { success } = await env.WS_LIMIT.limit({ key: req.headers.get("cf-connecting-ip") || "local" });
    if (!success) return new Response("Too Many Requests", { status: 429 });
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
    this.offerTimes = [];
    this.pk = null;
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
    const existing = this.ctx.getWebSockets(tag);
    if (existing.length >= (role === "host" ? 4 : 64)) {
      // host sem autenticação não segura vaga contra o host de verdade
      const stale = role === "host" && existing.find((h) => {
        try { const a = h.deserializeAttachment(); return !a || a.ok !== true; } catch { return true; }
      });
      if (!stale) return new Response("Too Many Requests", { status: 429 });
      try { stale.close(4008, "tempo esgotado"); } catch {}
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
      const host = this.ctx.getWebSockets("host").find((h) => {
        try {
          const a = h.deserializeAttachment();
          return a && a.ok === true;
        } catch {
          return false;
        }
      });
      if (host) {
        const pk = this.pk ?? (await this.ctx.storage.get("pk"));
        const hello = { t: "hello", iceServers: ICE_SERVERS };
        if (typeof pk === "string" && pk) hello.pk = pk;
        server.send(JSON.stringify(hello));
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

      if (data.t === "pk") {
        if (typeof data.pk === "string" && /^[0-9a-f]{2432}$/i.test(data.pk)) {
          const pk = data.pk.toLowerCase();
          const stored = this.pk ?? (await this.ctx.storage.get("pk"));
          if (pk !== stored) {
            this.pk = pk;
            await this.ctx.storage.put("pk", pk);
            for (const v of this.ctx.getWebSockets("view")) {
              try {
                const a = v.deserializeAttachment();
                if (a && !a.offered) {
                  v.send(JSON.stringify({ t: "pk", pk }));
                }
              } catch {}
            }
          }
        }
        return;
      }

      if (!data.id || typeof data.id !== "string" || !/^[0-9a-f]{8}$/i.test(data.id)) return;
      const viewers = this.ctx.getWebSockets("v:" + data.id.toLowerCase());
      if (data.t === "answer" && (typeof data.sdp === "string" || typeof data.sealed === "string")) {
        for (const v of viewers) {
          try {
            v.send(JSON.stringify({ t: "answer", sdp: typeof data.sdp === "string" ? data.sdp : "", sealed: typeof data.sealed === "string" ? data.sealed : "" }));
            const a = v.deserializeAttachment();
            if (a) { a.answered = true; v.serializeAttachment(a); }
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
      const sealed = typeof data.sealed === "string" ? data.sealed : "";
      if (!sealed && (typeof data.sdp !== "string" || typeof data.proof !== "string")) return;
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

      const now = Date.now();
      if (!this.offerTimes) this.offerTimes = [];
      this.offerTimes = this.offerTimes.filter((t) => now - t < 10000);
      if (this.offerTimes.length >= 20) {
        const reason = "muitas entradas ao mesmo tempo; tente de novo em alguns segundos";
        ws.send(JSON.stringify({ t: "closed", reason }));
        try { ws.close(4029, reason); } catch {}
        return;
      }
      this.offerTimes.push(now);
      att.offered = true;
      ws.serializeAttachment(att);
      verifiedHost.send(
        JSON.stringify({
          t: "offer",
          id: att.id,
          sdp: typeof data.sdp === "string" ? data.sdp : "",
          proof: typeof data.proof === "string" ? data.proof : "",
          name,
          sealed,
        })
      );
    }
  }

  async webSocketClose(ws, code, reason, wasClean) {
    await this._handleLeave(ws);
  }

  async webSocketError(ws, error) {
    await this._handleLeave(ws);
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

  async _handleLeave(ws) {
    let att;
    try {
      att = ws.deserializeAttachment();
    } catch {
      return;
    }
    if (att.role === "host" && att.ok && !att.left) {
      att.left = true;
      try { ws.serializeAttachment(att); } catch {}
      const hasOther = this.ctx.getWebSockets("host").some((h) => {
        if (h === ws) return false;
        try {
          const a = h.deserializeAttachment();
          return a && a.ok === true && !a.left;
        } catch {
          return false;
        }
      });
      if (!hasOther) {
        await this.ctx.storage.delete("pk");
        this.pk = null;
        this.offerTimes = [];
      }
      for (const v of this.ctx.getWebSockets("view")) {
        try {
          const a = v.deserializeAttachment();
          if (a && a.role === "view" && !a.answered) {
            const why = "a conexão com quem transmite caiu; tente de novo";
            v.send(JSON.stringify({ t: "closed", reason: why }));
            v.close(1000, why);
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
