/**
 * Thin stdin-JSON adapter for `fixtures.cli check --impl c`.
 */

#include "is_palindrome.h"

#include <cJSON.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static char *read_stdin(void) {
    size_t cap = 4096;
    size_t len = 0;
    char *buf = (char *)malloc(cap);
    if (!buf) {
        return NULL;
    }
    int c;
    while ((c = getchar()) != EOF) {
        if (len + 1 >= cap) {
            cap *= 2;
            char *nb = (char *)realloc(buf, cap);
            if (!nb) {
                free(buf);
                return NULL;
            }
            buf = nb;
        }
        buf[len++] = (char)c;
    }
    buf[len] = '\0';
    return buf;
}

static int decode_hex(const char *hex, uint8_t **out, size_t *out_len) {
    size_t n = strlen(hex);
    if (n % 2u != 0u) {
        return -1;
    }
    *out = (uint8_t *)malloc(n / 2u);
    if (!*out) {
        return -1;
    }
    for (size_t i = 0; i < n / 2u; ++i) {
        char pair[3] = {hex[i * 2], hex[i * 2 + 1], '\0'};
        (*out)[i] = (uint8_t)strtoul(pair, NULL, 16);
    }
    *out_len = n / 2u;
    return 0;
}

int main(void) {
    char *raw = read_stdin();
    if (!raw) {
        return 2;
    }
    cJSON *root = cJSON_Parse(raw);
    free(raw);
    if (!root) {
        fprintf(stderr, "invalid json\n");
        return 2;
    }

    cJSON *mode_js = cJSON_GetObjectItem(root, "mode");
    if (!cJSON_IsString(mode_js)) {
        cJSON_Delete(root);
        return 2;
    }
    const char *mode = mode_js->valuestring;

    uint8_t custom_buf[256];
    size_t custom_n = 0;
    cJSON *cust = cJSON_GetObjectItem(root, "custom");
    if (cJSON_IsArray(cust)) {
        cJSON *x = NULL;
        cJSON_ArrayForEach(x, cust) {
            if (cJSON_IsNumber(x)) {
                custom_buf[custom_n++] = (uint8_t)x->valueint;
            }
        }
    }

    int rc = 2;
    if (strcmp(mode, "hex") == 0) {
        cJSON *hex_js = cJSON_GetObjectItem(root, "hex");
        if (!cJSON_IsString(hex_js)) {
            cJSON_Delete(root);
            return 2;
        }
        uint8_t *data = NULL;
        size_t dlen = 0;
        if (decode_hex(hex_js->valuestring, &data, &dlen) != 0) {
            cJSON_Delete(root);
            return 2;
        }
        bool r = is_palindrome_from_bytes(data, dlen, custom_buf, custom_n);
        free(data);
        printf("%s\n", r ? "true" : "false");
        rc = r ? 0 : 1;
    } else if (strcmp(mode, "string") == 0) {
        cJSON *text_js = cJSON_GetObjectItem(root, "text");
        if (!cJSON_IsString(text_js)) {
            cJSON_Delete(root);
            return 2;
        }
        const char *text = text_js->valuestring;
        bool result = false;
        enum is_palindrome_utf8_status st =
            is_palindrome_from_utf8(text, strlen(text), custom_buf, custom_n, &result);
        if (st == IS_PALINDROME_UTF8_ERR_NON_ASCII) {
            fprintf(stderr, "NON_ASCII_STRING_INPUT\n");
            fprintf(stderr, "Input contains a scalar value > U+007F.\n");
            rc = 2;
        } else if (st == IS_PALINDROME_UTF8_OK) {
            printf("%s\n", result ? "true" : "false");
            rc = result ? 0 : 1;
        } else if (st == IS_PALINDROME_UTF8_ERR_INVALID_UTF8) {
            fprintf(stderr, "invalid utf-8\n");
            rc = 2;
        } else {
            fprintf(stderr, "unknown status\n");
            rc = 2;
        }
    } else {
        fprintf(stderr, "unknown mode\n");
        rc = 2;
    }

    cJSON_Delete(root);
    return rc;
}
