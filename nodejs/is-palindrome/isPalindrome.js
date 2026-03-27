/**
 * Core palindrome check (SPEC §2–§3). Standard library only (no npm deps).
 * @module isPalindrome
 */

/** String API: any Unicode scalar &gt; U+007F (SPEC §3). */
export class PalindromeError extends Error {
  /**
   * @param {string} code Error code (e.g. NON_ASCII_STRING_INPUT)
   */
  constructor(code) {
    super(code);
    this.name = "PalindromeError";
    /** @type {string} */
    this.code = code;
  }
}

/**
 * @param {number} b
 * @returns {boolean}
 */
function isAsciiAlnum(b) {
  return (
    (b >= 0x61 && b <= 0x7a) ||
    (b >= 0x41 && b <= 0x5a) ||
    (b >= 0x30 && b <= 0x39)
  );
}

/**
 * @param {number} b
 * @returns {boolean}
 */
function isAsciiLetter(b) {
  return (b >= 0x61 && b <= 0x7a) || (b >= 0x41 && b <= 0x5a);
}

/**
 * @param {number} a
 * @param {number} b
 * @returns {boolean}
 */
function bytesMatch(a, b) {
  if (isAsciiLetter(a) && isAsciiLetter(b)) {
    return (a | 32) === (b | 32);
  }
  return a === b;
}

/**
 * @param {number} b
 * @param {Set<number> | null | undefined} custom
 * @returns {boolean}
 */
function isValidByte(b, custom) {
  if (!isAsciiAlnum(b)) {
    return false;
  }
  if (custom && custom.size > 0 && custom.has(b)) {
    return false;
  }
  return true;
}

/**
 * Byte mode. `custom` delimiter bytes are skipped when non-empty (SPEC §2).
 * @param {Uint8Array} data
 * @param {Set<number> | null | undefined} [custom]
 * @returns {boolean}
 */
export function fromBytes(data, custom) {
  if (data.length === 0) {
    return true;
  }
  let l = 0;
  let r = data.length - 1;
  for (;;) {
    while (l <= r && !isValidByte(data[l], custom)) {
      l += 1;
    }
    while (l <= r && !isValidByte(data[r], custom)) {
      r -= 1;
    }
    if (l >= r) {
      return true;
    }
    if (!bytesMatch(data[l], data[r])) {
      return false;
    }
    l += 1;
    r -= 1;
  }
}

/**
 * UTF-8 string: rejects any scalar &gt; U+007F (SPEC §3).
 * @param {string} s
 * @param {Set<number> | null | undefined} [custom]
 * @returns {boolean}
 */
export function fromString(s, custom) {
  for (const c of s) {
    if (c.codePointAt(0) > 0x7f) {
      throw new PalindromeError("NON_ASCII_STRING_INPUT");
    }
  }
  if (s.length === 0) {
    return fromBytes(new Uint8Array(0), custom);
  }
  const enc = new TextEncoder();
  return fromBytes(enc.encode(s), custom);
}
