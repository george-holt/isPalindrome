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

bool is_palindrome(const uint8_t *data, size_t len,
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

enum is_palindrome_utf8_status is_palindrome_from_utf8(const char *utf8, size_t byte_len,
                                                       const uint8_t *extra_delimiter_bytes,
                                                       size_t extra_delimiter_count,
                                                       bool *result) {
    for (size_t i = 0; i < byte_len; ++i) {
        if ((unsigned char)utf8[i] > 0x7Fu) {
            return IS_PALINDROME_UTF8_ERR_NON_ASCII;
        }
    }
    const uint8_t *bytes = (const uint8_t *)utf8;
    *result = is_palindrome(bytes, byte_len, extra_delimiter_bytes,
                            extra_delimiter_count);
    return IS_PALINDROME_UTF8_OK;
}
