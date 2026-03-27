#include <catch2/catch_test_macros.hpp>

#include <IsPalindrome.hpp>

#include <cctype>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <nlohmann/json.hpp>
#include <string>
#include <unordered_set>
#include <vector>

namespace fs = std::filesystem;
using json = nlohmann::json;

namespace {

fs::path manifest_path() {
    return fs::path(__FILE__).parent_path().parent_path().parent_path() / "fixtures"
        / "acceptance_manifest.json";
}

std::vector<std::uint8_t> decode_hex(std::string_view hex) {
    REQUIRE(hex.size() % 2 == 0);
    std::vector<std::uint8_t> out(hex.size() / 2);
    for (std::size_t i = 0; i < out.size(); ++i) {
        auto pair = hex.substr(i * 2, 2);
        out[i] = static_cast<std::uint8_t>(std::stoul(std::string(pair), nullptr, 16));
    }
    return out;
}

std::string codepoint_to_utf8(char32_t cp) {
    std::string s;
    if (cp <= 0x7F) {
        s.push_back(static_cast<char>(cp));
    } else if (cp <= 0x7FF) {
        s.push_back(static_cast<char>(0xC0 | static_cast<char>(cp >> 6)));
        s.push_back(static_cast<char>(0x80 | static_cast<char>(cp & 0x3F)));
    } else if (cp <= 0xFFFF) {
        s.push_back(static_cast<char>(0xE0 | static_cast<char>(cp >> 12)));
        s.push_back(static_cast<char>(0x80 | static_cast<char>((cp >> 6) & 0x3F)));
        s.push_back(static_cast<char>(0x80 | static_cast<char>(cp & 0x3F)));
    } else {
        s.push_back(static_cast<char>(0xF0 | static_cast<char>(cp >> 18)));
        s.push_back(static_cast<char>(0x80 | static_cast<char>((cp >> 12) & 0x3F)));
        s.push_back(static_cast<char>(0x80 | static_cast<char>((cp >> 6) & 0x3F)));
        s.push_back(static_cast<char>(0x80 | static_cast<char>(cp & 0x3F)));
    }
    return s;
}

std::string unicode_scalar_to_utf8(const std::string& manifest_scalar) {
    constexpr std::string_view prefix = "U+";
    REQUIRE(manifest_scalar.size() > prefix.size());
    REQUIRE(manifest_scalar.compare(0, prefix.size(), prefix) == 0);
    auto hex = manifest_scalar.substr(prefix.size());
    auto cp = static_cast<char32_t>(std::stoul(hex, nullptr, 16));
    return codepoint_to_utf8(cp);
}

bool applies_to_cpp(const json& case_j) {
    if (!case_j.contains("applies_to"))
        return true;
    for (const auto& x : case_j.at("applies_to")) {
        if (x.get<std::string>() == "cpp")
            return true;
    }
    return false;
}

const std::unordered_set<std::uint8_t>* parse_custom_delimiters(
    const json& case_j,
    std::unordered_set<std::uint8_t>& out) {
    out.clear();
    if (!case_j.contains("options"))
        return nullptr;
    const auto& o = case_j.at("options");
    if (!o.contains("invalid_mode") || o.at("invalid_mode") != "custom")
        return nullptr;
    if (!o.contains("invalid_bytes_hex"))
        return nullptr;
    for (const auto& h : o.at("invalid_bytes_hex")) {
        const auto& hex = h.get_ref<const std::string&>();
        REQUIRE(hex.size() == 2);
        out.insert(static_cast<std::uint8_t>(std::stoul(hex, nullptr, 16)));
    }
    return out.empty() ? nullptr : &out;
}

enum class input_kind { bytes, string };

struct built_input {
    input_kind kind = input_kind::bytes;
    std::vector<std::uint8_t> bytes;
    std::string text;
};

built_input build_input(const json& case_j, bool string_api) {
    if (case_j.contains("input_ascii")) {
        const auto& s = case_j.at("input_ascii").get_ref<const std::string&>();
        if (string_api) {
            return {input_kind::string, {}, s};
        }
        return {input_kind::bytes, std::vector<std::uint8_t>(s.begin(), s.end()), {}};
    }
    if (case_j.contains("input_hex")) {
        const auto& hex = case_j.at("input_hex").get_ref<const std::string&>();
        return {input_kind::bytes, decode_hex(hex), {}};
    }
    if (case_j.contains("input_unicode_scalar")) {
        const auto& sc = case_j.at("input_unicode_scalar").get_ref<const std::string&>();
        return {input_kind::string, {}, unicode_scalar_to_utf8(sc)};
    }
    throw std::runtime_error("case has no recognized input field");
}

} // namespace

TEST_CASE("fixtures/acceptance_manifest.json (cpp profile)", "[acceptance]") {
    const auto path = manifest_path();
    INFO(path.string());
    REQUIRE(fs::exists(path));
    std::ifstream in(path);
    REQUIRE(in);
    json root = json::parse(in);
    std::unordered_set<std::uint8_t> custom_buf;

    for (const auto& case_j : root.at("cases")) {
        const auto id = case_j.at("id").get<std::string>();
        if (id == "pal-stream-note-001")
            continue;
        if (!applies_to_cpp(case_j))
            continue;

        std::cerr << "case " << id << '\n';

        const auto* custom = parse_custom_delimiters(case_j, custom_buf);
        const auto& expected = case_j.at("expected");
        const auto kind = expected.at("kind").get<std::string>();

        const bool string_api = case_j.contains("applies_to");

        if (kind == "boolean") {
            const bool want = expected.at("value").get<bool>();
            auto input = build_input(case_j, string_api);
            bool got = false;
            if (input.kind == input_kind::bytes) {
                got = palindrome::from_bytes(input.bytes, custom);
            } else {
                got = palindrome::from_string(input.text, custom);
            }
            REQUIRE(got == want);
        } else if (kind == "error") {
            const auto code = expected.at("code").get<std::string>();
            auto input = build_input(case_j, true);
            REQUIRE(input.kind == input_kind::string);
            try {
                (void)palindrome::from_string(input.text, custom);
                FAIL("expected palindrome_exception");
            } catch (const palindrome::palindrome_exception& ex) {
                REQUIRE(ex.error_code() == code);
            }
        }
    }
}
