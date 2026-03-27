/**
 * Thin stdin-JSON adapter for `fixtures.cli check --impl nodejs`.
 * One JSON object on stdin: { "mode": "hex"|"string", "hex"?, "text"?, "custom": number[] }
 */

import { readFileSync } from "node:fs";
import { fromBytes, fromString, PalindromeError } from "./isPalindrome.js";

const buf = readFileSync(0, "utf8");
const req = JSON.parse(buf.trim());
const custom = req.custom?.length ? new Set(req.custom) : undefined;

if (req.mode === "hex") {
  const hex = req.hex;
  if (!hex) {
    console.error("missing hex");
    process.exit(2);
  }
  const out = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    out[i / 2] = Number.parseInt(hex.slice(i, i + 2), 16);
  }
  const r = fromBytes(out, custom);
  process.stdout.write(r ? "true\n" : "false\n");
  process.exit(r ? 0 : 1);
}

if (req.mode === "string") {
  const text = req.text;
  if (text === undefined) {
    console.error("missing text");
    process.exit(2);
  }
  try {
    const r = fromString(text, custom);
    process.stdout.write(r ? "true\n" : "false\n");
    process.exit(r ? 0 : 1);
  } catch (e) {
    if (e instanceof PalindromeError) {
      console.error(e.code);
      console.error(String(e.message ?? e));
      process.exit(2);
    }
    throw e;
  }
}

console.error("unknown mode");
process.exit(2);
