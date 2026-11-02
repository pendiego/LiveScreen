#!/usr/bin/env node
// web/scripts/verify-manifest.mjs
// Valida offline a assinatura Ed25519 e os hashes SHA-256 do manifesto web (manifest.json).
import { readFileSync, existsSync, readdirSync, statSync } from "node:fs";
import { resolve, dirname, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { createHash, createPublicKey, verify } from "node:crypto";

const __dirname = dirname(fileURLToPath(import.meta.url));

export const DEFAULT_RELEASE_PUBLIC_KEY =
  "c46025005551cda3e63968e0b6a71bccd47a6411276b85aff966f93d81290930";
export const MANIFEST_PREFIX = "livescreen/web-manifest/v1\n";
export const MAX_MANIFEST_BYTES = 16 * 1024;
export const MAX_FILE_BYTES = 2 * 1024 * 1024;
export const MAX_FILES = 64;

export function parseArgs(args) {
  let keyHex = DEFAULT_RELEASE_PUBLIC_KEY;
  let dir = resolve(__dirname, "../public/v");
  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--key" && i + 1 < args.length) {
      keyHex = args[++i];
    } else if (args[i] === "--dir" && i + 1 < args.length) {
      dir = resolve(process.cwd(), args[++i]);
    }
  }
  return { keyHex, dir };
}

export function validatePath(path) {
  if (!path || typeof path !== "string") {
    throw new Error("caminho vazio no manifesto");
  }
  if (path.includes("..")) {
    throw new Error(`caminho contém '..' proibido: ${path}`);
  }
  if (path.startsWith("/") || path.startsWith("\\")) {
    throw new Error(`caminho absoluto proibido: ${path}`);
  }
  if (!/^[A-Za-z0-9._/-]+$/.test(path)) {
    throw new Error(`caractere inválido no caminho: ${path}`);
  }
}

export function getAllFiles(dir, baseDir = dir) {
  const results = [];
  if (!existsSync(dir)) return results;
  const entries = readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = resolve(dir, entry.name);
    if (entry.isDirectory()) {
      results.push(...getAllFiles(fullPath, baseDir));
    } else if (entry.isFile()) {
      const rel = relative(baseDir, fullPath).replace(/\\/g, "/");
      if (rel !== "manifest.json" && rel !== "manifest.json.sig") {
        results.push(rel);
      }
    }
  }
  return results;
}

export function verifyManifest(dir, keyHex = DEFAULT_RELEASE_PUBLIC_KEY) {
  const manifestPath = resolve(dir, "manifest.json");
  const sigPath = resolve(dir, "manifest.json.sig");

  if (!existsSync(manifestPath)) {
    throw new Error(`manifesto não encontrado: ${manifestPath}`);
  }
  if (!existsSync(sigPath)) {
    throw new Error(`assinatura do manifesto não encontrada: ${sigPath}`);
  }

  const manifestBytes = readFileSync(manifestPath);
  if (manifestBytes.length > MAX_MANIFEST_BYTES) {
    throw new Error(`manifesto excede o limite de tamanho (16 KiB): ${manifestBytes.length} bytes`);
  }

  const sigText = readFileSync(sigPath, "utf8").trim();
  if (!/^[0-9a-fA-F]{128}$/.test(sigText)) {
    throw new Error("formato de assinatura Ed25519 inválido (deve ser 128 caracteres hex)");
  }

  // Verifica assinatura Ed25519
  const rawPub = Buffer.from(keyHex, "hex");
  if (rawPub.length !== 32) {
    throw new Error("chave pública de release inválida (tamanho != 32 bytes)");
  }
  const spkiPrefix = Buffer.from("302a300506032b6570032100", "hex");
  const publicKey = createPublicKey({
    key: Buffer.concat([spkiPrefix, rawPub]),
    format: "der",
    type: "spki",
  });

  const msg = Buffer.concat([Buffer.from(MANIFEST_PREFIX, "utf8"), manifestBytes]);
  const sigBytes = Buffer.from(sigText, "hex");

  const sigOk = verify(null, msg, publicKey, sigBytes);
  if (!sigOk) {
    throw new Error("a página do espectador não confere com a assinatura do LiveScreen");
  }

  const manifest = JSON.parse(manifestBytes.toString("utf8"));
  if (!manifest || typeof manifest !== "object" || !manifest.files || typeof manifest.files !== "object") {
    throw new Error("formato de manifesto JSON inválido (campo 'files' obrigatório)");
  }

  const filesMap = manifest.files;
  const fileKeys = Object.keys(filesMap);

  if (fileKeys.length === 0) {
    throw new Error("manifesto não contém arquivos");
  }
  if (fileKeys.length > MAX_FILES) {
    throw new Error(`manifesto excede o limite de 64 arquivos (${fileKeys.length})`);
  }

  for (const relPath of fileKeys) {
    validatePath(relPath);
    const expectedHash = String(filesMap[relPath]).trim().toLowerCase();
    const filePath = resolve(dir, relPath);

    if (!existsSync(filePath)) {
      throw new Error(`arquivo não encontrado em disco: ${relPath}`);
    }

    const stat = statSync(filePath);
    if (stat.size > MAX_FILE_BYTES) {
      throw new Error(`arquivo ${relPath} excede o limite de tamanho (2 MiB): ${stat.size} bytes`);
    }

    const content = readFileSync(filePath);
    const gotHash = createHash("sha256").update(content).digest("hex");
    if (gotHash !== expectedHash) {
      throw new Error(`conteúdo do arquivo ${relPath} não confere com o hash assinado`);
    }
  }

  // Confere se há algum arquivo em disco não coberto pelo manifesto
  const diskFiles = getAllFiles(dir);
  for (const diskRel of diskFiles) {
    if (!Object.prototype.hasOwnProperty.call(filesMap, diskRel)) {
      throw new Error(`arquivo em disco não listado no manifesto assinado: ${diskRel}`);
    }
  }

  return fileKeys.length;
}

if (process.argv[1] && resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url))) {
  try {
    const { keyHex, dir } = parseArgs(process.argv.slice(2));
    const count = verifyManifest(dir, keyHex);
    console.log(`Manifesto verificado com sucesso (${count} arquivos conferem com a assinatura do LiveScreen).`);
  } catch (err) {
    console.error(`erro: ${err.message}`);
    process.exit(1);
  }
}
