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
bool is_palindrome_from_bytes(const uint8_t *data, size_t len,
                                const uint8_t *extra_delimiter_bytes,
                                size_t extra_delimiter_count);

enum is_palindrome_utf8_status {
    IS_PALINDROME_UTF8_OK = 0,
    /** String API: scalar > U+007F (SPEC §3). */
    IS_PALINDROME_UTF8_ERR_NON_ASCII = 1,
    /** Input is not well-formed UTF-8. */
    IS_PALINDROME_UTF8_ERR_INVALID_UTF8 = 2,
};

/**
 * UTF-8 string API. On success, writes palindrome result to *result and returns
 * IS_PALINDROME_UTF8_OK. On NON_ASCII scalar, returns IS_PALINDROME_UTF8_ERR_NON_ASCII.
 */
enum is_palindrome_utf8_status is_palindrome_from_utf8(const char *utf8, size_t byte_len,
                                                       const uint8_t *extra_delimiter_bytes,
                                                       size_t extra_delimiter_count,
                                                       bool *result);

#endif
