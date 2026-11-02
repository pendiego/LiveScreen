// web/public/v/signal3.js
// Browser-viewer signaling v3: ephemeral X25519 + ML-KEM-768 hybrid cryptography.
// Usable as a classic script in browser (sets globalThis.LiveScreenSignal3)
// and importable in Node.js for tests.

((root, factory) => {
  const api = factory();
  root.LiveScreenSignal3 = api;
  if (typeof module !== "undefined" && module.exports) {
    module.exports = api;
  }
})(typeof globalThis !== "undefined" ? globalThis : this, () => {
  const textEncoder = new TextEncoder();
  const textDecoder = new TextDecoder();

  function hexToBytes(hex) {
    if (typeof hex !== "string" || hex.length % 2 !== 0 || !/^[0-9a-fA-F]*$/.test(hex)) {
      throw new TypeError("invalid hex string");
    }
    const len = hex.length / 2;
    const bytes = new Uint8Array(len);
    for (let i = 0; i < len; i++) {
      bytes[i] = parseInt(hex.substring(i * 2, i * 2 + 2), 16);
    }
    return bytes;
  }

  function bytesToHex(bytes) {
    let hex = "";
    for (let i = 0; i < bytes.length; i++) {
      hex += bytes[i].toString(16).padStart(2, "0");
    }
    return hex;
  }

  function constantTimeEqual(a, b) {
    if (!a || !b || a.length !== b.length) return false;
    let diff = 0;
    for (let i = 0; i < a.length; i++) {
      diff |= a[i] ^ b[i];
    }
    return diff === 0;
  }

  function concatBytes(...arrays) {
    let totalLen = 0;
    for (const arr of arrays) totalLen += arr.length;
    const out = new Uint8Array(totalLen);
    let offset = 0;
    for (const arr of arrays) {
      out.set(arr, offset);
      offset += arr.length;
    }
    return out;
  }

  async function sha256(data) {
    const buf = await crypto.subtle.digest("SHA-256", data);
    return new Uint8Array(buf);
  }

  async function hkdfSha256(salt, ikm, info, length = 32) {
    const baseKey = await crypto.subtle.importKey("raw", ikm, "HKDF", false, ["deriveBits"]);
    const bits = await crypto.subtle.deriveBits(
      { name: "HKDF", hash: "SHA-256", salt, info },
      baseKey,
      length * 8
    );
    return new Uint8Array(bits);
  }

  async function hmacSha256(keyBytes, data) {
    const key = await crypto.subtle.importKey(
      "raw",
      keyBytes,
      { name: "HMAC", hash: "SHA-256" },
      false,
      ["sign"]
    );
    const sig = await crypto.subtle.sign("HMAC", key, data);
    return new Uint8Array(sig);
  }

  async function aesGcmEncrypt(keyBytes, nonce, aad, plaintext) {
    const key = await crypto.subtle.importKey("raw", keyBytes, "AES-GCM", false, ["encrypt"]);
    const ct = await crypto.subtle.encrypt(
      { name: "AES-GCM", iv: nonce, additionalData: aad, tagLength: 128 },
      key,
      plaintext
    );
    return new Uint8Array(ct);
  }

  async function aesGcmDecrypt(keyBytes, nonce, aad, ciphertext) {
    const key = await crypto.subtle.importKey("raw", keyBytes, "AES-GCM", false, ["decrypt"]);
    const pt = await crypto.subtle.decrypt(
      { name: "AES-GCM", iv: nonce, additionalData: aad, tagLength: 128 },
      key,
      ciphertext
    );
    return new Uint8Array(pt);
  }

  function padPlaintext(bytes) {
    const targetLen = Math.max(2048, Math.ceil(bytes.length / 2048) * 2048);
    const padded = new Uint8Array(targetLen);
    padded.set(bytes);
    padded.fill(0x20, bytes.length); // 0x20 = ASCII space
    return padded;
  }

  function parseLink(link) {
    if (typeof link !== "string") return null;
    const trimmed = link.trim();
    const m = trimmed.match(/(?:^|[#/])([0-9a-fA-F]{32})\.([0-9a-fA-F]{64})\.([0-9a-fA-F]{64})\.v3\s*$/);
    if (!m) return null;
    return {
      room: m[1].toLowerCase(),
      webKey: m[2].toLowerCase(),
      pkHash: m[3].toLowerCase(),
    };
  }

  async function checkHostPk(pkHex, pkHash) {
    if (typeof pkHex !== "string" || pkHex.length !== 2432) return null;
    let hostPk;
    try {
      hostPk = hexToBytes(pkHex);
    } catch {
      return null;
    }
    if (hostPk.length !== 1216) return null;

    let expectedHash;
    if (typeof pkHash === "string") {
      if (pkHash.length !== 64) return null;
      try {
        expectedHash = hexToBytes(pkHash);
      } catch {
        return null;
      }
    } else if (pkHash instanceof Uint8Array && pkHash.length === 32) {
      expectedHash = pkHash;
    } else {
      return null;
    }

    const prefix = textEncoder.encode("livescreen/web-host-pk/v3");
    const computedHash = await sha256(concatBytes(prefix, hostPk));
    if (!constantTimeEqual(computedHash, expectedHash)) {
      return null;
    }
    return hostPk;
  }

  async function sealOffer({ room, webKey, hostPk, obj }, rnd) {
    const PQ = globalThis.LiveScreenPQ;
    if (!PQ || !PQ.ml_kem768 || !PQ.x25519) {
      throw new Error("LiveScreenPQ not available");
    }

    const roomStr = typeof room === "string" ? room.toLowerCase() : "";
    if (roomStr.length !== 32) throw new Error("invalid room length");
    const roomBytes = textEncoder.encode(roomStr);

    const webKeyBytes = typeof webKey === "string" ? hexToBytes(webKey) : webKey;
    if (!webKeyBytes || webKeyBytes.length !== 32) throw new Error("invalid webKey length");

    const hostPkBytes = typeof hostPk === "string" ? hexToBytes(hostPk) : hostPk;
    if (!hostPkBytes || hostPkBytes.length !== 1216) throw new Error("invalid hostPk length");

    const hxPk = hostPkBytes.subarray(0, 32);
    const ek = hostPkBytes.subarray(32, 1216);

    // Validate ek via noble (encode/decode roundtrip per FIPS 203 §7.2) before encaps
    try {
      PQ.ml_kem768.prepare(ek);
    } catch (err) {
      throw new Error("invalid ek modulus: " + err.message);
    }

    const vxSk = (rnd && rnd.vxSk) || crypto.getRandomValues(new Uint8Array(32));
    const mRand = (rnd && rnd.mRand) || crypto.getRandomValues(new Uint8Array(32));
    const nonce = (rnd && rnd.nonce) || crypto.getRandomValues(new Uint8Array(12));

    // Ephemeral X25519 keypair and shared secret
    const vxPk = PQ.x25519.getPublicKey(vxSk);
    let ss_x;
    try {
      ss_x = PQ.x25519.getSharedSecret(vxSk, hxPk);
    } catch (err) {
      throw new Error("X25519 failed: " + err.message);
    }
    if (ss_x.every((b) => b === 0)) {
      throw new Error("zero shared secret");
    }

    // Encapsulate ML-KEM-768
    const { cipherText: ct_m, sharedSecret: ss_m } = PQ.ml_kem768.encapsulate(ek, mRand);
    if (ct_m.length !== 1088 || ss_m.length !== 32) {
      throw new Error("unexpected encapsulation output size");
    }

    // ctx = SHA-256("livescreen/web-signal/v3" || room || host_pk || vx_pk || ct_m)
    const ctxDomain = textEncoder.encode("livescreen/web-signal/v3");
    const ctx = await sha256(concatBytes(ctxDomain, roomBytes, hostPkBytes, vxPk, ct_m));

    // ikm = ss_m || ss_x || web_key
    const ikm = concatBytes(ss_m, ss_x, webKeyBytes);
    const salt = textEncoder.encode("livescreen/web-signal/v3");
    const k_offer = await hkdfSha256(salt, ikm, concatBytes(textEncoder.encode("offer"), ctx), 32);
    const s_chain = await hkdfSha256(salt, ikm, concatBytes(textEncoder.encode("chain"), ctx), 32);

    const preSalt = textEncoder.encode("livescreen/web-signal/v3/pre");
    const k_pre = await hkdfSha256(preSalt, webKeyBytes, roomBytes, 32);
    const preFull = await hmacSha256(k_pre, concatBytes(new Uint8Array([0x03]), vxPk, ct_m));
    const pre = preFull.subarray(0, 16);

    const plainJson = JSON.stringify(obj);
    const plainBytes = textEncoder.encode(plainJson);
    const padded = padPlaintext(plainBytes);

    const aad = concatBytes(new Uint8Array([0x03]), ctx);
    const ctWithTag = await aesGcmEncrypt(k_offer, nonce, aad, padded);

    const sealedBytes = concatBytes(new Uint8Array([0x03]), vxPk, ct_m, pre, nonce, ctWithTag);
    const sealed = bytesToHex(sealedBytes);

    return {
      sealed,
      state: {
        sChain: s_chain,
        ctx,
        vxSk,
        vxPk,
      },
    };
  }

  async function openAnswer(state, sealedHex) {
    if (!state || !state.sChain || !state.ctx || !state.vxSk) return null;
    const PQ = globalThis.LiveScreenPQ;
    if (!PQ || !PQ.x25519) return null;

    if (typeof sealedHex !== "string") return null;
    let bytes;
    try {
      bytes = hexToBytes(sealedHex);
    } catch {
      return null;
    }

    // 1 (prefix) + 32 (ax_pk) + 12 (nonce) + 16 (minimum GCM tag) = 61 bytes
    if (bytes.length < 61) return null;
    if (bytes[0] !== 0x03) return null;

    const ax_pk = bytes.subarray(1, 33);
    const nonce = bytes.subarray(33, 45);
    const ciphertext = bytes.subarray(45);

    let ss_a;
    try {
      ss_a = PQ.x25519.getSharedSecret(state.vxSk, ax_pk);
    } catch {
      return null;
    }
    if (ss_a.every((b) => b === 0)) return null;

    const ikm = concatBytes(state.sChain, ss_a);
    const salt = textEncoder.encode("livescreen/web-signal/v3/answer");
    const info = concatBytes(textEncoder.encode("answer"), state.ctx, ax_pk);
    const k_answer = await hkdfSha256(salt, ikm, info, 32);

    const aad = concatBytes(new Uint8Array([0x03]), state.ctx, ax_pk);
    let pt;
    try {
      pt = await aesGcmDecrypt(k_answer, nonce, aad, ciphertext);
    } catch {
      return null;
    }

    try {
      const ptStr = textDecoder.decode(pt);
      const obj = JSON.parse(ptStr);
      if (obj && typeof obj === "object") {
        return obj;
      }
    } catch {
      return null;
    }
    return null;
  }

  return {
    hexToBytes,
    bytesToHex,
    constantTimeEqual,
    concatBytes,
    padPlaintext,
    parseLink,
    checkHostPk,
    sealOffer,
    openAnswer,
  };
});
