// web/scripts/build-pq.mjs
// Bundles @noble/post-quantum and @noble/curves into an IIFE for browser v3 signaling.
import * as esbuild from "esbuild";
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";

const __dirname = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(__dirname, "..");

const pqPkg = JSON.parse(readFileSync(resolve(webRoot, "node_modules/@noble/post-quantum/package.json"), "utf8"));
const curvesPkg = JSON.parse(readFileSync(resolve(webRoot, "node_modules/@noble/curves/package.json"), "utf8"));

const entryCode = `import { ml_kem768 } from '@noble/post-quantum/ml-kem.js';
import { x25519 } from '@noble/curves/ed25519.js';
globalThis.LiveScreenPQ = { ml_kem768, x25519 };
`;

const inputHash = createHash("sha256").update(entryCode, "utf8").digest("hex");

const banner = `// LiveScreen PQ bundle: @noble/post-quantum@${pqPkg.version}, @noble/curves@${curvesPkg.version}
// Bundle input SHA-256: ${inputHash}
`;

const res = await esbuild.build({
  stdin: {
    contents: entryCode,
    resolveDir: webRoot,
    sourcefile: "entry-pq.js",
    loader: "js",
  },
  bundle: true,
  format: "iife",
  platform: "browser",
  target: "es2022",
  write: false,
});

const outPath = resolve(webRoot, "public/v/pq.js");
mkdirSync(dirname(outPath), { recursive: true });
const finalContent = banner + res.outputFiles[0].text;
writeFileSync(outPath, finalContent, "utf8");

console.log(`Wrote ${outPath} (${Buffer.byteLength(finalContent)} bytes)`);
