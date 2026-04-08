/**
 * Direct manifest-driven tests for ``isPalindrome.js``.
 * ``fixtures/acceptance_manifest.json`` is the behavioral source of truth.
 * Coverage for the Node implementation is collected from this ``js_test`` (Bazel + V8 coverage).
 */

import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  isPalindrome,
  isPalindromeFromUtf8,
  PalindromeError,
} from "./isPalindrome.js";

function manifestPath() {
  const rf =
    process.env.JS_BINARY__RUNFILES ||
    process.env.RUNFILES_DIR ||
    process.env.TEST_SRCDIR;
  const ws =
    process.env.JS_BINARY__WORKSPACE ||
    process.env.TEST_WORKSPACE ||
    "_main";
  if (rf) {
    const cands = [
      join(rf, ws, "fixtures", "acceptance_manifest.json"),
      join(rf, "fixtures", "acceptance_manifest.json"),
    ];
    for (const p of cands) {
      if (existsSync(p)) {
        return p;
      }
    }
  }
  const here = fileURLToPath(new URL(".", import.meta.url));
  return join(here, "..", "..", "..", "fixtures", "acceptance_manifest.json");
}

function applies(case_, lang) {
  if (case_.expected === undefined) {
    return false;
  }
  const a = case_.applies_to;
  if (a === undefined || a === null) {
    return true;
  }
  return a.includes(lang);
}

function parseCustom(case_) {
  const opts = case_.options;
  if (!opts || opts.invalid_mode !== "custom") {
    return undefined;
  }
  const hexArr = opts.invalid_bytes_hex;
  if (!Array.isArray(hexArr) || hexArr.length === 0) {
    return undefined;
  }
  return new Set(hexArr.map((h) => parseInt(h, 16)));
}

function unicodeScalarToStr(spec) {
  const p = spec.trim().toUpperCase();
  if (!p.startsWith("U+")) {
    throw new Error(spec);
  }
  return String.fromCodePoint(parseInt(p.slice(2), 16));
}

function usesStringApi(case_) {
  if (case_.input_unicode_scalar !== undefined) {
    return true;
  }
  return case_.category === "string_api";
}

function runCase(case_) {
  const id = case_.id ?? "?";
  const custom = parseCustom(case_);
  const exp = case_.expected;
  const kind = exp.kind;

  if (usesStringApi(case_)) {
    const s =
      case_.input_unicode_scalar !== undefined
        ? unicodeScalarToStr(case_.input_unicode_scalar)
        : case_.input_ascii;
    try {
      const got = isPalindromeFromUtf8(s, custom);
      assert.equal(kind, "boolean", id);
      assert.equal(got, exp.value, id);
    } catch (e) {
      if (e instanceof PalindromeError) {
        assert.equal(kind, "error", id);
        assert.equal(e.code, exp.code, id);
        return;
      }
      throw e;
    }
    return;
  }

  let data;
  if (case_.input_ascii !== undefined) {
    data = new TextEncoder().encode(case_.input_ascii);
  } else if (case_.input_hex !== undefined) {
    const h = case_.input_hex;
    data = new Uint8Array(h.length / 2);
    for (let i = 0; i < h.length; i += 2) {
      data[i / 2] = parseInt(h.slice(i, i + 2), 16);
    }
  } else {
    assert.fail(`${id}: no input field`);
  }

  const got = isPalindrome(data, custom);
  assert.equal(kind, "boolean", id);
  assert.equal(got, exp.value, id);
}

const raw = readFileSync(manifestPath(), "utf8");
const manifest = JSON.parse(raw);
for (const case_ of manifest.cases) {
  if (!applies(case_, "nodejs")) {
    continue;
  }
  runCase(case_);
}
