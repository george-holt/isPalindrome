/**
 * Direct manifest-driven tests for the C palindrome library.
 * Cases: ``fixtures/acceptance_manifest.json``. Coverage from this ``cc_test``, not adapters.
 */

extern "C" {
#include "is_palindrome.h"
}

#include <cJSON.h>

#include <catch2/catch_test_macros.hpp>

#include <cctype>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <string>
#include <vector>

static std::string manifest_path() {
    const char* rf = std::getenv("RUNFILES_DIR");
    if (rf) {
        std::string ws = "_main";
        if (const char* w = std::getenv("TEST_WORKSPACE")) {
            ws = w;
        }
        std::vector<std::string> cands = {
            std::string(rf) + "/" + ws + "/fixtures/acceptance_manifest.json",
            std::string(rf) + "/fixtures/acceptance_manifest.json",
        };
        for (const auto& rel : cands) {
            std::ifstream try_open(rel);
            if (try_open.good()) {
                return rel;
            }
        }
    }
    return "fixtures/acceptance_manifest.json";
}

static std::string read_file(const std::string& path) {
    std::ifstream in(path, std::ios::binary);
    REQUIRE(in.good());
    return std::string(std::istreambuf_iterator<char>(in), {});
}

static std::string codepoint_to_utf8(std::uint32_t cp) {
    std::string out;
    if (cp <= 0x7FU) {
        out += static_cast<char>(cp);
    } else if (cp <= 0x7FFU) {
        out += static_cast<char>(0xC0U | ((cp >> 6) & 0x1FU));
        out += static_cast<char>(0x80U | (cp & 0x3FU));
    } else if (cp <= 0xFFFFU) {
        out += static_cast<char>(0xE0U | ((cp >> 12) & 0x0FU));
        out += static_cast<char>(0x80U | ((cp >> 6) & 0x3FU));
        out += static_cast<char>(0x80U | (cp & 0x3FU));
    } else {
        out += static_cast<char>(0xF0U | ((cp >> 18) & 0x07U));
        out += static_cast<char>(0x80U | ((cp >> 12) & 0x3FU));
        out += static_cast<char>(0x80U | ((cp >> 6) & 0x3FU));
        out += static_cast<char>(0x80U | (cp & 0x3FU));
    }
    return out;
}

static std::string unicode_scalar_to_utf8(const std::string& spec) {
    std::string p = spec;
    while (!p.empty() && p.front() == ' ') {
        p.erase(p.begin());
    }
    while (!p.empty() && p.back() == ' ') {
        p.pop_back();
    }
    for (auto& c : p) {
        c = static_cast<char>(std::toupper(static_cast<unsigned char>(c)));
    }
    REQUIRE(p.rfind("U+", 0) == 0);
    std::uint32_t cp = std::stoul(p.substr(2), nullptr, 16);
    return codepoint_to_utf8(cp);
}

static int applies_to_c(const cJSON* case_j) {
    if (!cJSON_GetObjectItem(case_j, "expected")) {
        return 0;
    }
    const cJSON* at = cJSON_GetObjectItem(case_j, "applies_to");
    if (!at || at->type == cJSON_NULL) {
        return 1;
    }
    if (at->type != cJSON_Array) {
        return 0;
    }
    const cJSON* x = nullptr;
    cJSON_ArrayForEach(x, at) {
        if (cJSON_IsString(x) && std::string(x->valuestring) == "c") {
            return 1;
        }
    }
    return 0;
}

static void get_custom_bytes(const cJSON* case_j, std::vector<uint8_t>* out_extra) {
    out_extra->clear();
    const cJSON* opts = cJSON_GetObjectItem(case_j, "options");
    if (!cJSON_IsObject(opts)) {
        return;
    }
    const cJSON* mode = cJSON_GetObjectItem(opts, "invalid_mode");
    if (!cJSON_IsString(mode) || std::string(mode->valuestring) != "custom") {
        return;
    }
    const cJSON* hex_arr = cJSON_GetObjectItem(opts, "invalid_bytes_hex");
    if (!cJSON_IsArray(hex_arr) || cJSON_GetArraySize(hex_arr) == 0) {
        return;
    }
    const cJSON* h = nullptr;
    cJSON_ArrayForEach(h, hex_arr) {
        REQUIRE(cJSON_IsString(h));
        unsigned v = std::stoul(h->valuestring, nullptr, 16);
        out_extra->push_back(static_cast<uint8_t>(v));
    }
}

static int uses_string_api(const cJSON* case_j) {
    if (cJSON_GetObjectItem(case_j, "input_unicode_scalar")) {
        return 1;
    }
    const cJSON* cat = cJSON_GetObjectItem(case_j, "category");
    return cJSON_IsString(cat) && std::string(cat->valuestring) == "string_api";
}

static void run_case(const cJSON* case_j, std::vector<uint8_t>* extra_buf) {
    const cJSON* id_j = cJSON_GetObjectItem(case_j, "id");
    const char* id = cJSON_IsString(id_j) ? id_j->valuestring : "?";
    const cJSON* exp = cJSON_GetObjectItem(case_j, "expected");
    REQUIRE(exp);
    const cJSON* kind_j = cJSON_GetObjectItem(exp, "kind");
    REQUIRE(cJSON_IsString(kind_j));
    std::string kind = kind_j->valuestring;

    get_custom_bytes(case_j, extra_buf);
    const uint8_t* extra_ptr = extra_buf->empty() ? nullptr : extra_buf->data();
    size_t extra_n = extra_buf->size();

    if (uses_string_api(case_j)) {
        std::string utf8;
        const cJSON* uh = cJSON_GetObjectItem(case_j, "input_utf8_hex");
        if (cJSON_IsString(uh)) {
            std::string h = uh->valuestring;
            REQUIRE(h.size() % 2 == 0);
            for (size_t j = 0; j < h.size(); j += 2) {
                utf8.push_back(static_cast<char>(std::stoul(h.substr(j, 2), nullptr, 16)));
            }
        } else {
            const cJSON* us = cJSON_GetObjectItem(case_j, "input_unicode_scalar");
            if (cJSON_IsString(us)) {
                utf8 = unicode_scalar_to_utf8(us->valuestring);
            } else {
                const cJSON* ia = cJSON_GetObjectItem(case_j, "input_ascii");
                REQUIRE(cJSON_IsString(ia));
                utf8 = ia->valuestring;
            }
        }
        bool result = false;
        enum is_palindrome_utf8_status st = is_palindrome_from_utf8(
            utf8.data(), utf8.size(), extra_ptr, extra_n, &result);
        if (kind == "boolean") {
            REQUIRE(st == IS_PALINDROME_UTF8_OK);
            const cJSON* v = cJSON_GetObjectItem(exp, "value");
            REQUIRE(cJSON_IsBool(v));
            int want = cJSON_IsTrue(v);
            REQUIRE((result ? 1 : 0) == want);
        } else {
            REQUIRE(kind == "error");
            REQUIRE(st == IS_PALINDROME_UTF8_ERR_NON_ASCII);
        }
        return;
    }

    std::vector<uint8_t> bytes;
    const cJSON* ia = cJSON_GetObjectItem(case_j, "input_ascii");
    const cJSON* ih = cJSON_GetObjectItem(case_j, "input_hex");
    if (cJSON_IsString(ia)) {
        const char* s = ia->valuestring;
        bytes.assign(s, s + std::strlen(s));
    } else if (cJSON_IsString(ih)) {
        std::string h = ih->valuestring;
        REQUIRE(h.size() % 2 == 0);
        for (size_t i = 0; i < h.size(); i += 2) {
            bytes.push_back(static_cast<uint8_t>(std::stoul(h.substr(i, 2), nullptr, 16)));
        }
    } else {
        FAIL("case " << id << ": no input field");
    }

    bool got = is_palindrome(bytes.data(), bytes.size(), extra_ptr, extra_n);
    REQUIRE(kind == "boolean");
    const cJSON* v = cJSON_GetObjectItem(exp, "value");
    REQUIRE(cJSON_IsBool(v));
    REQUIRE((got ? 1 : 0) == (cJSON_IsTrue(v) ? 1 : 0));
}

TEST_CASE("acceptance_manifest_cases", "[pal]") {
    std::string path = manifest_path();
    std::string raw = read_file(path);
    cJSON* root = cJSON_Parse(raw.c_str());
    REQUIRE(root);
    const cJSON* cases = cJSON_GetObjectItem(root, "cases");
    REQUIRE(cJSON_IsArray(cases));
    std::vector<uint8_t> extra_buf;

    const cJSON* case_j = nullptr;
    cJSON_ArrayForEach(case_j, cases) {
        if (!applies_to_c(case_j)) {
            continue;
        }
        run_case(case_j, &extra_buf);
    }
    cJSON_Delete(root);
}
