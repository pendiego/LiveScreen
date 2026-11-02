// web/test/manifest.test.mjs
import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import crypto from "node:crypto";
import { verifyManifest, MANIFEST_PREFIX } from "../scripts/verify-manifest.mjs";

test("manifest: valid manifest with arbitrary files passes", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "manifest-test-"));
  try {
    const fontsDir = path.join(tmpDir, "fonts");
    fs.mkdirSync(fontsDir, { recursive: true });

    const f1 = path.join(tmpDir, "index.html");
    const f2 = path.join(fontsDir, "font.woff2");
    fs.writeFileSync(f1, "<html>valid</html>");
    fs.writeFileSync(f2, "fontdata");

    const h1 = crypto.createHash("sha256").update(fs.readFileSync(f1)).digest("hex");
    const h2 = crypto.createHash("sha256").update(fs.readFileSync(f2)).digest("hex");

    const { publicKey, privateKey } = crypto.generateKeyPairSync("ed25519");
    const rawPubHex = publicKey.export({ type: "spki", format: "der" }).subarray(-32).toString("hex");

    const manifestObj = {
      files: {
        "fonts/font.woff2": h2,
        "index.html": h1,
      },
    };
    const manifestBytes = Buffer.from(JSON.stringify(manifestObj, null, 2) + "\n");
    fs.writeFileSync(path.join(tmpDir, "manifest.json"), manifestBytes);

    const msg = Buffer.concat([Buffer.from(MANIFEST_PREFIX, "utf8"), manifestBytes]);
    const sig = crypto.sign(null, msg, privateKey);
    fs.writeFileSync(path.join(tmpDir, "manifest.json.sig"), sig.toString("hex") + "\n");

    const count = verifyManifest(tmpDir, rawPubHex);
    assert.equal(count, 2);
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("manifest: tampered file fails", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "manifest-test-"));
  try {
    const f1 = path.join(tmpDir, "viewer.js");
    fs.writeFileSync(f1, "original");
    const h1 = crypto.createHash("sha256").update(fs.readFileSync(f1)).digest("hex");

    const { publicKey, privateKey } = crypto.generateKeyPairSync("ed25519");
    const rawPubHex = publicKey.export({ type: "spki", format: "der" }).subarray(-32).toString("hex");

    const manifestBytes = Buffer.from(JSON.stringify({ files: { "viewer.js": h1 } }) + "\n");
    fs.writeFileSync(path.join(tmpDir, "manifest.json"), manifestBytes);

    const msg = Buffer.concat([Buffer.from(MANIFEST_PREFIX, "utf8"), manifestBytes]);
    const sig = crypto.sign(null, msg, privateKey);
    fs.writeFileSync(path.join(tmpDir, "manifest.json.sig"), sig.toString("hex") + "\n");

    // Adultera o arquivo
    fs.writeFileSync(f1, "tampered");
    assert.throws(() => verifyManifest(tmpDir, rawPubHex), /não confere com o hash assinado/);
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("manifest: tampered signature fails", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "manifest-test-"));
  try {
    const f1 = path.join(tmpDir, "index.html");
    fs.writeFileSync(f1, "content");
    const h1 = crypto.createHash("sha256").update(fs.readFileSync(f1)).digest("hex");

    const { publicKey } = crypto.generateKeyPairSync("ed25519");
    const rawPubHex = publicKey.export({ type: "spki", format: "der" }).subarray(-32).toString("hex");

    const manifestBytes = Buffer.from(JSON.stringify({ files: { "index.html": h1 } }) + "\n");
    fs.writeFileSync(path.join(tmpDir, "manifest.json"), manifestBytes);
    fs.writeFileSync(path.join(tmpDir, "manifest.json.sig"), "00".repeat(64) + "\n");

    assert.throws(() => verifyManifest(tmpDir, rawPubHex), /não confere com a assinatura/);
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("manifest: unlisted file on disk fails", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "manifest-test-"));
  try {
    const f1 = path.join(tmpDir, "index.html");
    fs.writeFileSync(f1, "content");
    const h1 = crypto.createHash("sha256").update(fs.readFileSync(f1)).digest("hex");

    const { publicKey, privateKey } = crypto.generateKeyPairSync("ed25519");
    const rawPubHex = publicKey.export({ type: "spki", format: "der" }).subarray(-32).toString("hex");

    const manifestBytes = Buffer.from(JSON.stringify({ files: { "index.html": h1 } }) + "\n");
    fs.writeFileSync(path.join(tmpDir, "manifest.json"), manifestBytes);

    const msg = Buffer.concat([Buffer.from(MANIFEST_PREFIX, "utf8"), manifestBytes]);
    const sig = crypto.sign(null, msg, privateKey);
    fs.writeFileSync(path.join(tmpDir, "manifest.json.sig"), sig.toString("hex") + "\n");

    // Cria arquivo extra não assinado
    fs.writeFileSync(path.join(tmpDir, "malicious.js"), "alert(1)");
    assert.throws(() => verifyManifest(tmpDir, rawPubHex), /não listado no manifesto/);
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});
