/**
 * Core palindrome check (SPEC §2–§3). ISO C + libc only.
 */
#ifndef IS_PALINDROME_H
#define IS_PALINDROME_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/**
 * Byte mode. When extra_delimiter_count is 0, extra_delimiter_bytes is ignored and only
 * default ASCII alnum validity applies (SPEC §2).
 */
bool is_palindrome(const uint8_t *data, size_t len,
                   const uint8_t *extra_delimiter_bytes,
                   size_t extra_delimiter_count);

enum is_palindrome_utf8_status {
    IS_PALINDROME_UTF8_OK = 0,
    /** String / byte-string API: reject non-ASCII (any byte > U+007F); same code as manifest `NON_ASCII_STRING_INPUT`. */
    IS_PALINDROME_UTF8_ERR_NON_ASCII = 1,
};

/**
 * UTF-8 string API. On success, writes palindrome result to *result and returns
 * IS_PALINDROME_UTF8_OK. If any input byte has the high bit set (not UTF-8 ASCII), returns
 * IS_PALINDROME_UTF8_ERR_NON_ASCII (includes malformed multi-byte prefixes; no separate invalid-UTF-8 code).
 */
enum is_palindrome_utf8_status is_palindrome_from_utf8(const char *utf8, size_t byte_len,
                                                       const uint8_t *extra_delimiter_bytes,
                                                       size_t extra_delimiter_count,
                                                       bool *result);

#endif
