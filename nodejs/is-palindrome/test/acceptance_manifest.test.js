/**
 * Loads `fixtures/acceptance_manifest.json` (SPEC §4).
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { fromBytes, fromString, PalindromeError } from "../isPalindrome.js";

const __dirname = dirname(fileURLToPath(import.meta.url));

function manifestPath() {
  return join(__dirname, "../../../fixtures/acceptance_manifest.json");
}

/**
 * @param {string} hex
 * @returns {Uint8Array}
 */
function decodeHex(hex) {
  assert.equal(hex.length % 2, 0, "odd hex length");
  const out = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    out[i / 2] = Number.parseInt(hex.slice(i, i + 2), 16);
  }
  return out;
}

/**
 * @param {string} manifestScalar e.g. U+00E9
 * @returns {string}
 */
function unicodeScalarToString(manifestScalar) {
  const prefix = "U+";
  assert.ok(manifestScalar.length > prefix.length);
  assert.ok(manifestScalar.toUpperCase().startsWith(prefix));
  const hex = manifestScalar.slice(prefix.length);
  const cp = Number.parseInt(hex, 16);
  return String.fromCodePoint(cp);
}

/**
 * @param {object} caseObj
 * @returns {boolean}
 */
function appliesToNodejs(caseObj) {
  const at = caseObj.applies_to;
  if (at === undefined || at === null) {
    return true;
  }
  if (!Array.isArray(at)) {
    return false;
  }
  return at.some((s) => s === "nodejs" || s === "javascript");
}

/**
 * @param {object} caseObj
 * @param {Set<number>} buf mutated; cleared then filled
 * @returns {Set<number> | null}
 */
function parseCustomDelimiters(caseObj, buf) {
  buf.clear();
  const opts = caseObj.options;
  if (!opts || opts.invalid_mode !== "custom") {
    return null;
  }
  const arr = opts.invalid_bytes_hex;
  if (!Array.isArray(arr)) {
    return null;
  }
  for (const h of arr) {
    assert.equal(typeof h, "string");
    assert.equal(h.length, 2);
    buf.add(Number.parseInt(h, 16));
  }
  if (buf.size === 0) {
    return null;
  }
  return buf;
}

/**
 * @param {object} caseObj
 * @param {boolean} stringApi
 * @returns {{ kind: "bytes", bytes: Uint8Array } | { kind: "string", text: string }}
 */
function buildInput(caseObj, stringApi) {
  if (Object.hasOwn(caseObj, "input_ascii")) {
    const ascii = caseObj.input_ascii;
    assert.equal(typeof ascii, "string");
    if (stringApi) {
      return { kind: "string", text: ascii };
    }
    return { kind: "bytes", bytes: new TextEncoder().encode(ascii) };
  }
  if (Object.hasOwn(caseObj, "input_hex")) {
    const hex = caseObj.input_hex;
    assert.equal(typeof hex, "string");
    return { kind: "bytes", bytes: decodeHex(hex) };
  }
  if (Object.hasOwn(caseObj, "input_unicode_scalar")) {
    const sc = caseObj.input_unicode_scalar;
    assert.equal(typeof sc, "string");
    return { kind: "string", text: unicodeScalarToString(sc) };
  }
  throw new Error("case has no recognized input field");
}

test("acceptance_manifest_matches_spec", () => {
  const path = manifestPath();
  const json = readFileSync(path, "utf8");
  const root = JSON.parse(json);
  const cases = root.cases;
  assert.ok(Array.isArray(cases));

  const customBuf = new Set();

  for (const caseObj of cases) {
    const id = caseObj.id;
    assert.equal(typeof id, "string");

    if (id === "pal-stream-note-001") {
      continue;
    }
    if (!appliesToNodejs(caseObj)) {
      continue;
    }

    const custom = parseCustomDelimiters(caseObj, customBuf);
    const expected = caseObj.expected;
    assert.ok(expected);
    const kind = expected.kind;
    assert.equal(typeof kind, "string");

    const stringApi = caseObj.applies_to !== undefined && caseObj.applies_to !== null;

    if (kind === "boolean") {
      const want = expected.value;
      assert.equal(typeof want, "boolean");
      const input = buildInput(caseObj, stringApi);
      if (input.kind === "bytes") {
        assert.equal(fromBytes(input.bytes, custom), want, id);
      } else {
        assert.equal(fromString(input.text, custom), want, id);
      }
    } else if (kind === "error") {
      const code = expected.code;
      assert.equal(typeof code, "string");
      const input = buildInput(caseObj, true);
      assert.equal(input.kind, "string", `${id}: expected string input for error case`);
      assert.throws(
        () => {
          fromString(input.text, custom);
        },
        (err) => err instanceof PalindromeError && err.code === code,
        id,
      );
    } else {
      throw new Error(`unknown expected.kind in ${id}`);
    }
  }
});
