/**
 * Loads fixtures/acceptance_manifest.json (SPEC §4). Requires cJSON (test-only dependency).
 */

#include "is_palindrome.h"

#include <cJSON.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifndef PAL_MANIFEST_PATH
#error PAL_MANIFEST_PATH must be defined by CMake
#endif

enum input_kind { INPUT_BYTES, INPUT_STRING };

static int read_file(const char *path, char **out, size_t *out_len) {
    FILE *f = fopen(path, "rb");
    if (!f) {
        return -1;
    }
    if (fseek(f, 0, SEEK_END) != 0) {
        fclose(f);
        return -1;
    }
    long sz = ftell(f);
    if (sz < 0) {
        fclose(f);
        return -1;
    }
    if (fseek(f, 0, SEEK_SET) != 0) {
        fclose(f);
        return -1;
    }
    char *buf = (char *)malloc((size_t)sz + 1u);
    if (!buf) {
        fclose(f);
        return -1;
    }
    size_t n = fread(buf, 1u, (size_t)sz, f);
    fclose(f);
    buf[n] = '\0';
    *out = buf;
    *out_len = n;
    return 0;
}

static int decode_hex(const char *hex, uint8_t *out, size_t out_cap, size_t *out_len) {
    size_t n = strlen(hex);
    if (n % 2u != 0u) {
        return -1;
    }
    if (n / 2u > out_cap) {
        return -1;
    }
    for (size_t i = 0; i < n / 2u; ++i) {
        char pair[3] = {hex[i * 2], hex[i * 2 + 1], '\0'};
        out[i] = (uint8_t)strtoul(pair, NULL, 16);
    }
    *out_len = n / 2u;
    return 0;
}

/** Encode Unicode scalar to UTF-8 (manifest uses BMP / supplementary rarely). */
static int codepoint_to_utf8(uint32_t cp, char *out, size_t cap, size_t *out_len) {
    if (cp <= 0x7Fu) {
        if (cap < 1u) {
            return -1;
        }
        out[0] = (char)cp;
        *out_len = 1u;
        return 0;
    }
    if (cp <= 0x7FFu) {
        if (cap < 2u) {
            return -1;
        }
        out[0] = (char)(0xC0u | (char)((cp >> 6) & 0x1Fu));
        out[1] = (char)(0x80u | (char)(cp & 0x3Fu));
        *out_len = 2u;
        return 0;
    }
    if (cp <= 0xFFFFu) {
        if (cap < 3u) {
            return -1;
        }
        out[0] = (char)(0xE0u | (char)((cp >> 12) & 0x0Fu));
        out[1] = (char)(0x80u | (char)((cp >> 6) & 0x3Fu));
        out[2] = (char)(0x80u | (char)(cp & 0x3Fu));
        *out_len = 3u;
        return 0;
    }
    if (cp <= 0x10FFFFu) {
        if (cap < 4u) {
            return -1;
        }
        out[0] = (char)(0xF0u | (char)((cp >> 18) & 0x07u));
        out[1] = (char)(0x80u | (char)((cp >> 12) & 0x3Fu));
        out[2] = (char)(0x80u | (char)((cp >> 6) & 0x3Fu));
        out[3] = (char)(0x80u | (char)(cp & 0x3Fu));
        *out_len = 4u;
        return 0;
    }
    return -1;
}

static int unicode_scalar_to_utf8(const char *manifest_scalar, char *out, size_t cap, size_t *out_len) {
    const char *p = manifest_scalar;
    if (strncmp(p, "U+", 2u) != 0 && strncmp(p, "u+", 2u) != 0) {
        return -1;
    }
    p += 2;
    uint32_t cp = (uint32_t)strtoul(p, NULL, 16);
    return codepoint_to_utf8(cp, out, cap, out_len);
}

static int applies_to_c(const cJSON *case_obj) {
    const cJSON *at = cJSON_GetObjectItem((cJSON *)case_obj, "applies_to");
    if (at == NULL || cJSON_IsNull(at)) {
        return 1;
    }
    if (!cJSON_IsArray(at)) {
        return 0;
    }
    const cJSON *x = NULL;
    cJSON_ArrayForEach(x, (cJSON *)at) {
        if (cJSON_IsString(x) && strcmp(x->valuestring, "c") == 0) {
            return 1;
        }
    }
    return 0;
}

static const uint8_t *parse_custom_delimiters(const cJSON *case_obj, uint8_t *buf, size_t *count_out) {
    const cJSON *opts = cJSON_GetObjectItem((cJSON *)case_obj, "options");
    if (opts == NULL) {
        return NULL;
    }
    const cJSON *mode = cJSON_GetObjectItem((cJSON *)opts, "invalid_mode");
    if (mode == NULL || !cJSON_IsString(mode) || strcmp(mode->valuestring, "custom") != 0) {
        return NULL;
    }
    const cJSON *arr = cJSON_GetObjectItem((cJSON *)opts, "invalid_bytes_hex");
    if (arr == NULL || !cJSON_IsArray(arr)) {
        return NULL;
    }
    size_t n = 0;
    const cJSON *h = NULL;
    cJSON_ArrayForEach(h, (cJSON *)arr) {
        if (!cJSON_IsString(h)) {
            continue;
        }
        const char *hex = h->valuestring;
        if (strlen(hex) != 2u) {
            continue;
        }
        char pair[3] = {hex[0], hex[1], '\0'};
        buf[n++] = (uint8_t)strtoul(pair, NULL, 16);
    }
    if (n == 0u) {
        return NULL;
    }
    *count_out = n;
    return buf;
}

static int build_input(const cJSON *case_obj, int string_api, uint8_t *bytes_buf, size_t bytes_cap,
                       size_t *bytes_len, char *str_buf, size_t str_cap, enum input_kind *kind) {
    const cJSON *ascii = cJSON_GetObjectItem((cJSON *)case_obj, "input_ascii");
    if (ascii != NULL) {
        if (!cJSON_IsString(ascii)) {
            return -1;
        }
        const char *s = ascii->valuestring;
        if (string_api) {
            strncpy(str_buf, s, str_cap - 1u);
            str_buf[str_cap - 1u] = '\0';
            *kind = INPUT_STRING;
            return 0;
        }
        size_t sl = strlen(s);
        if (sl > bytes_cap) {
            return -1;
        }
        memcpy(bytes_buf, s, sl);
        *bytes_len = sl;
        *kind = INPUT_BYTES;
        return 0;
    }
    const cJSON *hex = cJSON_GetObjectItem((cJSON *)case_obj, "input_hex");
    if (hex != NULL) {
        if (!cJSON_IsString(hex)) {
            return -1;
        }
        if (decode_hex(hex->valuestring, bytes_buf, bytes_cap, bytes_len) != 0) {
            return -1;
        }
        *kind = INPUT_BYTES;
        return 0;
    }
    const cJSON *usc = cJSON_GetObjectItem((cJSON *)case_obj, "input_unicode_scalar");
    if (usc != NULL) {
        if (!cJSON_IsString(usc)) {
            return -1;
        }
        size_t ulen = 0;
        if (unicode_scalar_to_utf8(usc->valuestring, str_buf, str_cap, &ulen) != 0) {
            return -1;
        }
        str_buf[ulen] = '\0';
        *kind = INPUT_STRING;
        return 0;
    }
    return -1;
}

int main(void) {
    char *json_raw = NULL;
    size_t json_len = 0;
    if (read_file(PAL_MANIFEST_PATH, &json_raw, &json_len) != 0) {
        fprintf(stderr, "cannot read manifest: %s\n", PAL_MANIFEST_PATH);
        return 1;
    }
    cJSON *root = cJSON_Parse(json_raw);
    free(json_raw);
    if (root == NULL) {
        fprintf(stderr, "JSON parse error\n");
        return 1;
    }
    cJSON *cases = cJSON_GetObjectItem(root, "cases");
    if (cases == NULL || !cJSON_IsArray(cases)) {
        fprintf(stderr, "no cases array\n");
        cJSON_Delete(root);
        return 1;
    }

    uint8_t bytes_buf[4096];
    char str_buf[4096];
    uint8_t custom_buf[256];

    cJSON *case_obj = NULL;
    cJSON_ArrayForEach(case_obj, cases) {
        const cJSON *id = cJSON_GetObjectItem((cJSON *)case_obj, "id");
        if (id == NULL || !cJSON_IsString(id)) {
            fprintf(stderr, "case missing id\n");
            cJSON_Delete(root);
            return 1;
        }
        if (strcmp(id->valuestring, "pal-stream-note-001") == 0) {
            continue;
        }
        if (!applies_to_c(case_obj)) {
            continue;
        }

        fprintf(stderr, "case %s\n", id->valuestring);

        size_t custom_count = 0;
        const uint8_t *custom = parse_custom_delimiters(case_obj, custom_buf, &custom_count);

        const cJSON *expected = cJSON_GetObjectItem((cJSON *)case_obj, "expected");
        if (expected == NULL) {
            fprintf(stderr, "%s: missing expected\n", id->valuestring);
            cJSON_Delete(root);
            return 1;
        }
        const cJSON *ek = cJSON_GetObjectItem((cJSON *)expected, "kind");
        if (ek == NULL || !cJSON_IsString(ek)) {
            fprintf(stderr, "%s: expected.kind\n", id->valuestring);
            cJSON_Delete(root);
            return 1;
        }

        int string_api = cJSON_GetObjectItem((cJSON *)case_obj, "applies_to") != NULL;

        if (strcmp(ek->valuestring, "boolean") == 0) {
            const cJSON *val = cJSON_GetObjectItem((cJSON *)expected, "value");
            if (val == NULL || !cJSON_IsBool(val)) {
                fprintf(stderr, "%s: expected.value\n", id->valuestring);
                cJSON_Delete(root);
                return 1;
            }
            int want = cJSON_IsTrue(val);
            size_t blen = 0;
            enum input_kind ik;
            if (build_input(case_obj, string_api, bytes_buf, sizeof bytes_buf, &blen, str_buf,
                            sizeof str_buf, &ik)
                != 0) {
                fprintf(stderr, "%s: build_input\n", id->valuestring);
                cJSON_Delete(root);
                return 1;
            }
            if (ik == INPUT_BYTES) {
                int got = is_palindrome_from_bytes(bytes_buf, blen, custom_buf, custom_count);
                if ((int)got != want) {
                    fprintf(stderr, "%s: from_bytes want %d got %d\n", id->valuestring, want, (int)got);
                    cJSON_Delete(root);
                    return 1;
                }
            } else {
                size_t slen = strlen(str_buf);
                bool got = false;
                enum is_palindrome_utf8_status st =
                    is_palindrome_from_utf8(str_buf, slen, custom_buf, custom_count, &got);
                if (st != IS_PALINDROME_UTF8_OK) {
                    fprintf(stderr, "%s: utf8 unexpected status %d\n", id->valuestring, (int)st);
                    cJSON_Delete(root);
                    return 1;
                }
                if ((int)got != want) {
                    fprintf(stderr, "%s: from_utf8 want %d got %d\n", id->valuestring, want, (int)got);
                    cJSON_Delete(root);
                    return 1;
                }
            }
        } else if (strcmp(ek->valuestring, "error") == 0) {
            const cJSON *code = cJSON_GetObjectItem((cJSON *)expected, "code");
            if (code == NULL || !cJSON_IsString(code)) {
                fprintf(stderr, "%s: expected.code\n", id->valuestring);
                cJSON_Delete(root);
                return 1;
            }
            size_t blen = 0;
            enum input_kind ik;
            if (build_input(case_obj, 1, bytes_buf, sizeof bytes_buf, &blen, str_buf, sizeof str_buf,
                            &ik)
                != 0) {
                fprintf(stderr, "%s: build_input error case\n", id->valuestring);
                cJSON_Delete(root);
                return 1;
            }
            if (ik != INPUT_STRING) {
                fprintf(stderr, "%s: expected string input for error case\n", id->valuestring);
                cJSON_Delete(root);
                return 1;
            }
            size_t slen = strlen(str_buf);
            bool dummy = false;
            enum is_palindrome_utf8_status st =
                is_palindrome_from_utf8(str_buf, slen, custom_buf, custom_count, &dummy);
            if (st != IS_PALINDROME_UTF8_ERR_NON_ASCII) {
                fprintf(stderr, "%s: want NON_ASCII got %d\n", id->valuestring, (int)st);
                cJSON_Delete(root);
                return 1;
            }
            if (strcmp(code->valuestring, "NON_ASCII_STRING_INPUT") != 0) {
                fprintf(stderr, "%s: unexpected code %s\n", id->valuestring, code->valuestring);
                cJSON_Delete(root);
                return 1;
            }
        } else {
            fprintf(stderr, "%s: unknown expected.kind\n", id->valuestring);
            cJSON_Delete(root);
            return 1;
        }
    }

    cJSON_Delete(root);
    return 0;
}
