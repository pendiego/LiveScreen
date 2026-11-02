// web/test/signal3.test.mjs
import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import "../public/v/pq.js";
import "../public/v/signal3.js";

const { ml_kem768, x25519 } = globalThis.LiveScreenPQ;
const {
  parseLink,
  checkHostPk,
  sealOffer,
  openAnswer,
  hexToBytes,
  bytesToHex,
  padPlaintext,
  constantTimeEqual,
} = globalThis.LiveScreenSignal3;

const __dirname = path.dirname(fileURLToPath(import.meta.url));

test("parseLink: valid and invalid v3 links", () => {
  const room = "0123456789abcdef0123456789abcdef";
  const webKey = "a".repeat(64);
  const pkHash = "b".repeat(64);

  // With fragment #
  const p1 = parseLink(`#${room}.${webKey}.${pkHash}.v3`);
  assert.ok(p1);
  assert.equal(p1.room, room);
  assert.equal(p1.webKey, webKey);
  assert.equal(p1.pkHash, pkHash);

  // Full URL
  const p2 = parseLink(`https://livescreen.pendiego.com/v/#${room}.${webKey}.${pkHash}.v3`);
  assert.ok(p2);
  assert.equal(p2.room, room);
  assert.equal(p2.webKey, webKey);
  assert.equal(p2.pkHash, pkHash);

  // Uppercase hex normalized to lowercase
  const p3 = parseLink(`${room.toUpperCase()}.${webKey.toUpperCase()}.${pkHash.toUpperCase()}.v3`);
  assert.ok(p3);
  assert.equal(p3.room, room);
  assert.equal(p3.webKey, webKey);
  assert.equal(p3.pkHash, pkHash);

  // Rejects v2 and v1 links
  assert.equal(parseLink(`${room}.${"a".repeat(32)}.v2`), null);
  assert.equal(parseLink(`${room}.${"a".repeat(32)}`), null);

  // Rejects invalid lengths or garbage
  assert.equal(parseLink(""), null);
  assert.equal(parseLink("not-a-link"), null);
  assert.equal(parseLink(`${room}.${"a".repeat(63)}.${pkHash}.v3`), null);
  assert.equal(parseLink(`${room}.${webKey}.${"b".repeat(65)}.v3`), null);
  assert.equal(parseLink(`${room}.${webKey}.${pkHash}.v4`), null);
});

test("checkHostPk: valid pk and adversarial 1-bit tamper", async () => {
  const kp_m = ml_kem768.keygen(new Uint8Array(64).fill(1));
  const hx_sk = new Uint8Array(32).fill(2);
  const hx_pk = x25519.getPublicKey(hx_sk);
  const host_pk = new Uint8Array(1216);
  host_pk.set(hx_pk, 0);
  host_pk.set(kp_m.publicKey, 32);

  const prefix = new TextEncoder().encode("livescreen/web-host-pk/v3");
  const hashBuf = await crypto.subtle.digest(
    "SHA-256",
    new Uint8Array([...prefix, ...host_pk])
  );
  const pk_hash = new Uint8Array(hashBuf);
  const pkHex = bytesToHex(host_pk);
  const hashHex = bytesToHex(pk_hash);

  // Valid pk
  const checked = await checkHostPk(pkHex, hashHex);
  assert.ok(checked instanceof Uint8Array);
  assert.equal(checked.length, 1216);
  assert.ok(constantTimeEqual(checked, host_pk));

  // Adversarial: 1 bit changed in host_pk must be rejected
  const tamperedPk = new Uint8Array(host_pk);
  tamperedPk[0] ^= 1;
  const rejected1 = await checkHostPk(bytesToHex(tamperedPk), hashHex);
  assert.equal(rejected1, null);

  // 1 bit changed in pk_hash must be rejected
  const tamperedHash = new Uint8Array(pk_hash);
  tamperedHash[31] ^= 1;
  const rejected2 = await checkHostPk(pkHex, bytesToHex(tamperedHash));
  assert.equal(rejected2, null);

  // Invalid length or malformed hex
  assert.equal(await checkHostPk(pkHex.slice(0, 2430), hashHex), null);
  assert.equal(await checkHostPk(pkHex, hashHex.slice(0, 62)), null);
  assert.equal(await checkHostPk("zz".repeat(1216), hashHex), null);
});

test("sealOffer: padding sizes equal for short and long names (B10)", async () => {
  const kp_m = ml_kem768.keygen(new Uint8Array(64).fill(10));
  const hx_sk = new Uint8Array(32).fill(20);
  const hx_pk = x25519.getPublicKey(hx_sk);
  const host_pk = new Uint8Array(1216);
  host_pk.set(hx_pk, 0);
  host_pk.set(kp_m.publicKey, 32);

  const room = "0123456789abcdef0123456789abcdef";
  const webKey = new Uint8Array(32).fill(30);

  // 1-char name
  const s1 = await sealOffer({
    room,
    webKey,
    hostPk: host_pk,
    obj: { sdp: "v=0\r\ntest\r\n", name: "A", avatar: 1 },
  });

  // 32-char name
  const s2 = await sealOffer({
    room,
    webKey,
    hostPk: host_pk,
    obj: { sdp: "v=0\r\ntest\r\n", name: "A".repeat(32), avatar: 1 },
  });

  assert.equal(s1.sealed.length, s2.sealed.length);
  // Sealed hex length: 2 * (1 + 32 + 1088 + 16 + 12 + (2048 + 16)) = 6426
  assert.equal(s1.sealed.length, 6426);
});

test("sealOffer & openAnswer: roundtrip and adversarial tampering", async () => {
  const host_m_seed = new Uint8Array(64).fill(40);
  const kp_m = ml_kem768.keygen(host_m_seed);
  const hx_sk = new Uint8Array(32).fill(50);
  const hx_pk = x25519.getPublicKey(hx_sk);
  const host_pk = new Uint8Array(1216);
  host_pk.set(hx_pk, 0);
  host_pk.set(kp_m.publicKey, 32);

  const room = "0123456789abcdef0123456789abcdef";
  const webKey = new Uint8Array(32).fill(60);

  const offerObj = { sdp: "v=0\r\no=- viewer offer\r\n", name: "Alice", avatar: 2 };
  const { sealed: offerSealed, state } = await sealOffer({
    room,
    webKey,
    hostPk: host_pk,
    obj: offerObj,
  });

  assert.equal(offerSealed.slice(0, 2), "03");

  // Simulate host building valid answer
  const offerBytes = hexToBytes(offerSealed);
  const vx_pk = offerBytes.subarray(1, 33);
  const ct_m = offerBytes.subarray(33, 1121);
  const ss_m = ml_kem768.decapsulate(ct_m, kp_m.secretKey);
  const ss_x = x25519.getSharedSecret(hx_sk, vx_pk);

  const ctxDomain = new TextEncoder().encode("livescreen/web-signal/v3");
  const ctxInput = new Uint8Array([
    ...ctxDomain,
    ...new TextEncoder().encode(room),
    ...host_pk,
    ...vx_pk,
    ...ct_m,
  ]);
  const ctx = new Uint8Array(await crypto.subtle.digest("SHA-256", ctxInput));

  const ikm = new Uint8Array([...ss_m, ...ss_x, ...webKey]);
  const baseKey = await crypto.subtle.importKey("raw", ikm, "HKDF", false, ["deriveBits"]);
  const s_chain_bits = await crypto.subtle.deriveBits(
    { name: "HKDF", hash: "SHA-256", salt: ctxDomain, info: new Uint8Array([...new TextEncoder().encode("chain"), ...ctx]) },
    baseKey,
    256
  );
  const s_chain = new Uint8Array(s_chain_bits);

  const ax_sk = new Uint8Array(32).fill(70);
  const ax_pk = x25519.getPublicKey(ax_sk);
  const ss_a = x25519.getSharedSecret(ax_sk, vx_pk);

  const ikm_ans = new Uint8Array([...s_chain, ...ss_a]);
  const baseKeyAns = await crypto.subtle.importKey("raw", ikm_ans, "HKDF", false, ["deriveBits"]);
  const k_ans_bits = await crypto.subtle.deriveBits(
    {
      name: "HKDF",
      hash: "SHA-256",
      salt: new TextEncoder().encode("livescreen/web-signal/v3/answer"),
      info: new Uint8Array([...new TextEncoder().encode("answer"), ...ctx, ...ax_pk]),
    },
    baseKeyAns,
    256
  );
  const k_ans = new Uint8Array(k_ans_bits);

  const answerObj = { sdp: "v=0\r\no=- host answer\r\n" };
  const paddedAns = padPlaintext(new TextEncoder().encode(JSON.stringify(answerObj)));
  const ansNonce = new Uint8Array(12).fill(80);
  const ansAad = new Uint8Array([0x03, ...ctx, ...ax_pk]);
  const ansKey = await crypto.subtle.importKey("raw", k_ans, "AES-GCM", false, ["encrypt"]);
  const ansCt = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv: ansNonce, additionalData: ansAad, tagLength: 128 },
    ansKey,
    paddedAns
  );

  const answerSealedBytes = new Uint8Array([
    0x03,
    ...ax_pk,
    ...ansNonce,
    ...new Uint8Array(ansCt),
  ]);
  const answerSealedHex = bytesToHex(answerSealedBytes);

  // 1. Valid answer decrypts cleanly
  const opened = await openAnswer(state, answerSealedHex);
  assert.ok(opened);
  assert.equal(opened.sdp, answerObj.sdp);

  // 2. Tampered ciphertext returns null (adversarial)
  const tamperedAns = new Uint8Array(answerSealedBytes);
  tamperedAns[tamperedAns.length - 1] ^= 1;
  assert.equal(await openAnswer(state, bytesToHex(tamperedAns)), null);

  // 3. Tampered nonce returns null
  const tamperedNonce = new Uint8Array(answerSealedBytes);
  tamperedNonce[35] ^= 1;
  assert.equal(await openAnswer(state, bytesToHex(tamperedNonce)), null);

  // 4. Tampered ax_pk returns null
  const tamperedAx = new Uint8Array(answerSealedBytes);
  tamperedAx[5] ^= 1;
  assert.equal(await openAnswer(state, bytesToHex(tamperedAx)), null);

  // 5. Wrong prefix returns null
  const wrongPrefix = new Uint8Array(answerSealedBytes);
  wrongPrefix[0] = 0x02;
  assert.equal(await openAnswer(state, bytesToHex(wrongPrefix)), null);

  // 6. Truncated answer returns null
  assert.equal(await openAnswer(state, bytesToHex(answerSealedBytes.subarray(0, 50))), null);
});

test("sealOffer: rejects invalid ek modulus (FIPS 203 §7.2)", async () => {
  const kp_m = ml_kem768.keygen(new Uint8Array(64).fill(90));
  const hx_sk = new Uint8Array(32).fill(91);
  const hx_pk = x25519.getPublicKey(hx_sk);
  const bad_host_pk = new Uint8Array(1216);
  bad_host_pk.set(hx_pk, 0);
  bad_host_pk.set(kp_m.publicKey, 32);

  // Corrupt ek modulus
  bad_host_pk[32] = 0xff;
  bad_host_pk[33] = 0xff;

  await assert.rejects(
    async () => {
      await sealOffer({
        room: "0123456789abcdef0123456789abcdef",
        webKey: new Uint8Array(32).fill(92),
        hostPk: bad_host_pk,
        obj: { sdp: "v=0\r\n" },
      });
    },
    /invalid ek modulus/
  );
});

test("vector fixture: consistency and answer verification", async () => {
  const fixturePath = path.resolve(__dirname, "v3-vector.json");
  assert.ok(fs.existsSync(fixturePath), "v3-vector.json must exist");
  const vec = JSON.parse(fs.readFileSync(fixturePath, "utf8"));

  assert.equal(vec.room.length, 32);
  assert.equal(vec.web_key.length, 64);
  assert.equal(vec.host_pk.length, 2432);
  assert.equal(vec.pk_hash.length, 64);
  assert.equal(vec.offer_nonce.length, 24);
  assert.equal(vec.offer_sealed.length, 6426);

  // Re-verify that our JS sealed offer matches the fixture
  const kp_m = ml_kem768.keygen(hexToBytes(vec.host_m_seed));
  const hx_pk = x25519.getPublicKey(hexToBytes(vec.host_x_sk));
  const expectedHostPk = new Uint8Array(1216);
  expectedHostPk.set(hx_pk, 0);
  expectedHostPk.set(kp_m.publicKey, 32);
  assert.equal(vec.host_pk, bytesToHex(expectedHostPk));

  const checkedPk = await checkHostPk(vec.host_pk, vec.pk_hash);
  assert.ok(checkedPk);

  // The fixture is committed and read-only: answer must be present and open exactly.
  assert.ok(vec.answer_sealed && vec.answer_sealed.length > 0, "answer_sealed must be non-empty");

  const vxSk = hexToBytes(vec.viewer_x_sk);
  const mRand = hexToBytes(vec.viewer_m_rand);
  const nonce = hexToBytes(vec.offer_nonce);
  const offerObj = JSON.parse(vec.offer_plain);

  const { sealed, state } = await sealOffer(
    {
      room: vec.room,
      webKey: vec.web_key,
      hostPk: vec.host_pk,
      obj: offerObj,
    },
    { vxSk, mRand, nonce }
  );
  assert.equal(sealed, vec.offer_sealed, "JS offer must be byte-identical to Rust fixture");

  const opened = await openAnswer(state, vec.answer_sealed);
  assert.ok(opened, "Failed to open answer_sealed from RustCore");
  assert.equal(opened.sdp, vec.answer_sdp);
});
