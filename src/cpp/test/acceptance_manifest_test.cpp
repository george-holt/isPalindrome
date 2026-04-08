/**
 * Direct manifest-driven tests for ``palindrome::is_palindrome`` (SPEC §2–§3).
 * Shared cases: ``fixtures/acceptance_manifest.json``. Coverage is from this ``cc_test``.
 */

#include <IsPalindrome.hpp>

#include <catch2/catch_test_macros.hpp>
#include <cctype>
#include <cstdlib>
#include <fstream>
#include <nlohmann/json.hpp>
#include <string>
#include <unordered_set>
#include <vector>

using json = nlohmann::json;
namespace p = palindrome;

static std::string manifest_path() {
    const char* rf = std::getenv("RUNFILES_DIR");
    if (rf) {
        std::string ws = "_main";
        if (const char* w = std::getenv("TEST_WORKSPACE")) {
            ws = w;
        }
        for (const auto& rel : std::vector<std::string>{
                 std::string(rf) + "/" + ws + "/fixtures/acceptance_manifest.json",
                 std::string(rf) + "/fixtures/acceptance_manifest.json",
             }) {
            std::ifstream try_open(rel);
            if (try_open.good()) {
                return rel;
            }
        }
    }
    return "fixtures/acceptance_manifest.json";
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

static std::unordered_set<std::uint8_t>* fill_custom(const json& case_j,
                                                    std::unordered_set<std::uint8_t>& storage) {
    auto oit = case_j.find("options");
    if (oit == case_j.end() || !oit->is_object()) {
        return nullptr;
    }
    const json& opts = *oit;
    if (opts.value("invalid_mode", "") != "custom") {
        return nullptr;
    }
    auto hex_it = opts.find("invalid_bytes_hex");
    if (hex_it == opts.end() || !hex_it->is_array() || hex_it->empty()) {
        return nullptr;
    }
    storage.clear();
    for (const auto& h : *hex_it) {
        storage.insert(static_cast<std::uint8_t>(std::stoul(h.get<std::string>(), nullptr, 16)));
    }
    return &storage;
}

static bool applies_to_cpp(const json& case_j) {
    if (!case_j.contains("expected")) {
        return false;
    }
    auto it = case_j.find("applies_to");
    if (it == case_j.end() || it->is_null()) {
        return true;
    }
    if (!it->is_array()) {
        return false;
    }
    for (const auto& v : *it) {
        if (v.is_string() && v.get<std::string>() == "cpp") {
            return true;
        }
    }
    return false;
}

static bool uses_string_api(const json& case_j) {
    if (case_j.contains("input_unicode_scalar")) {
        return true;
    }
    auto it = case_j.find("category");
    return it != case_j.end() && it->is_string() && it->get<std::string>() == "string_api";
}

static void run_case(const json& case_j, std::unordered_set<std::uint8_t>& custom_storage) {
    const std::string id = case_j.value("id", "?");
    const auto& exp = case_j.at("expected");
    const std::string kind = exp.at("kind").get<std::string>();
    const std::unordered_set<std::uint8_t>* custom = fill_custom(case_j, custom_storage);

    if (uses_string_api(case_j)) {
        std::string utf8;
        if (case_j.contains("input_utf8_hex")) {
            const std::string h = case_j.at("input_utf8_hex").get<std::string>();
            REQUIRE(h.size() % 2 == 0);
            for (std::size_t j = 0; j < h.size(); j += 2) {
                utf8.push_back(static_cast<char>(std::stoul(h.substr(j, 2), nullptr, 16)));
            }
        } else if (case_j.contains("input_unicode_scalar")) {
            utf8 = unicode_scalar_to_utf8(case_j.at("input_unicode_scalar").get<std::string>());
        } else {
            utf8 = case_j.at("input_ascii").get<std::string>();
        }
        try {
            bool got = p::is_palindrome_from_utf8(utf8, custom);
            REQUIRE(kind == "boolean");
            REQUIRE(got == exp.at("value").get<bool>());
        } catch (const p::palindrome_exception& e) {
            REQUIRE(kind == "error");
            REQUIRE(e.error_code() == exp.at("code").get<std::string>());
            if (exp.contains("message")) {
                REQUIRE(std::string(e.what()) == exp.at("message").get<std::string>());
            }
        }
        return;
    }

    std::vector<std::uint8_t> bytes;
    if (case_j.contains("input_ascii")) {
        const std::string s = case_j.at("input_ascii").get<std::string>();
        bytes.assign(s.begin(), s.end());
    } else if (case_j.contains("input_hex")) {
        const std::string h = case_j.at("input_hex").get<std::string>();
        REQUIRE(h.size() % 2 == 0);
        for (std::size_t i = 0; i < h.size(); i += 2) {
            bytes.push_back(static_cast<std::uint8_t>(std::stoul(h.substr(i, 2), nullptr, 16)));
        }
    } else {
        FAIL("case " << id << ": no input field");
    }

    bool got = p::is_palindrome(bytes, custom);
    REQUIRE(kind == "boolean");
    REQUIRE(got == exp.at("value").get<bool>());
}

TEST_CASE("acceptance_manifest_cases", "[pal]") {
    const std::string path = manifest_path();
    std::ifstream in(path);
    INFO("manifest path: " << path);
    REQUIRE(in.good());
    json root;
    in >> root;
    std::unordered_set<std::uint8_t> custom_storage;
    for (const auto& case_j : root.at("cases")) {
        if (!applies_to_cpp(case_j)) {
            continue;
        }
        run_case(case_j, custom_storage);
    }
}
