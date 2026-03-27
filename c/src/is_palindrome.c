/**
 * Core palindrome check (SPEC §2–§3). Standard library only.
 */

#include "is_palindrome.h"

#include <string.h>

static int is_ascii_alnum(uint8_t b) {
    return (b >= 'a' && b <= 'z') || (b >= 'A' && b <= 'Z') || (b >= '0' && b <= '9');
}

static int is_ascii_letter(uint8_t b) {
    return (b >= 'a' && b <= 'z') || (b >= 'A' && b <= 'Z');
}

static int bytes_match(uint8_t a, uint8_t b) {
    if (is_ascii_letter(a) && is_ascii_letter(b)) {
        return (uint8_t)(a | 32u) == (uint8_t)(b | 32u);
    }
    return a == b;
}

static int is_valid_byte(uint8_t b, const unsigned char *extra_mark, int has_extra) {
    if (!is_ascii_alnum(b)) {
        return 0;
    }
    if (has_extra && extra_mark[b]) {
        return 0;
    }
    return 1;
}

bool is_palindrome_from_bytes(const uint8_t *data, size_t len,
                              const uint8_t *extra_delimiter_bytes,
                              size_t extra_delimiter_count) {
    unsigned char extra_mark[256];
    int has_extra = extra_delimiter_bytes != NULL && extra_delimiter_count > 0;
    if (has_extra) {
        memset(extra_mark, 0, sizeof extra_mark);
        for (size_t i = 0; i < extra_delimiter_count; ++i) {
            extra_mark[extra_delimiter_bytes[i]] = 1;
        }
    }

    if (len == 0) {
        return true;
    }
    size_t l = 0;
    size_t r = len - 1;
    for (;;) {
        while (l <= r && !is_valid_byte(data[l], has_extra ? extra_mark : NULL, has_extra)) {
            ++l;
        }
        while (l <= r && !is_valid_byte(data[r], has_extra ? extra_mark : NULL, has_extra)) {
            --r;
        }
        if (l >= r) {
            return true;
        }
        if (!bytes_match(data[l], data[r])) {
            return false;
        }
        ++l;
        --r;
    }
}

/**
 * Decode one UTF-8 codepoint from utf8[*idx]. Advances *idx. Returns 0 on success, -1 on invalid.
 */
static int utf8_next(const char *utf8, size_t len, size_t *idx, uint32_t *out_cp) {
    if (*idx >= len) {
        return -1;
    }
    unsigned char c0 = (unsigned char)utf8[*idx];
    if (c0 <= 0x7Fu) {
        *out_cp = c0;
        ++*idx;
        return 0;
    }
    if ((c0 & 0xE0u) == 0xC0u) {
        if (*idx + 1 >= len) {
            return -1;
        }
        unsigned char c1 = (unsigned char)utf8[*idx + 1];
        if ((c1 & 0xC0u) != 0x80u) {
            return -1;
        }
        uint32_t cp = (uint32_t)((c0 & 0x1Fu) << 6) | (uint32_t)(c1 & 0x3Fu);
        if (cp < 0x80u) {
            return -1;
        }
        *out_cp = cp;
        *idx += 2;
        return 0;
    }
    if ((c0 & 0xF0u) == 0xE0u) {
        if (*idx + 2 >= len) {
            return -1;
        }
        unsigned char c1 = (unsigned char)utf8[*idx + 1];
        unsigned char c2 = (unsigned char)utf8[*idx + 2];
        if ((c1 & 0xC0u) != 0x80u || (c2 & 0xC0u) != 0x80u) {
            return -1;
        }
        uint32_t cp =
            (uint32_t)((c0 & 0x0Fu) << 12) | (uint32_t)((c1 & 0x3Fu) << 6) | (uint32_t)(c2 & 0x3Fu);
        if (cp < 0x800u) {
            return -1;
        }
        *out_cp = cp;
        *idx += 3;
        return 0;
    }
    if ((c0 & 0xF8u) == 0xF0u) {
        if (*idx + 3 >= len) {
            return -1;
        }
        unsigned char c1 = (unsigned char)utf8[*idx + 1];
        unsigned char c2 = (unsigned char)utf8[*idx + 2];
        unsigned char c3 = (unsigned char)utf8[*idx + 3];
        if ((c1 & 0xC0u) != 0x80u || (c2 & 0xC0u) != 0x80u || (c3 & 0xC0u) != 0x80u) {
            return -1;
        }
        uint32_t cp = (uint32_t)((c0 & 0x07u) << 18) | (uint32_t)((c1 & 0x3Fu) << 12)
            | (uint32_t)((c2 & 0x3Fu) << 6) | (uint32_t)(c3 & 0x3Fu);
        if (cp < 0x10000u) {
            return -1;
        }
        *out_cp = cp;
        *idx += 4;
        return 0;
    }
    return -1;
}

enum is_palindrome_utf8_status is_palindrome_from_utf8(const char *utf8, size_t byte_len,
                                                       const uint8_t *extra_delimiter_bytes,
                                                       size_t extra_delimiter_count,
                                                       bool *result) {
    size_t i = 0;
    while (i < byte_len) {
        uint32_t cp = 0;
        if (utf8_next(utf8, byte_len, &i, &cp) != 0) {
            return IS_PALINDROME_UTF8_ERR_INVALID_UTF8;
        }
        if (cp > 0x7Fu) {
            return IS_PALINDROME_UTF8_ERR_NON_ASCII;
        }
    }
    const uint8_t *bytes = (const uint8_t *)utf8;
    *result = is_palindrome_from_bytes(bytes, byte_len, extra_delimiter_bytes,
                                       extra_delimiter_count);
    return IS_PALINDROME_UTF8_OK;
}
