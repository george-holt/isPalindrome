#include <IsPalindrome.hpp>

#include <stdexcept>
#include <vector>

namespace palindrome {
namespace {

bool is_ascii_alnum(std::uint8_t b) noexcept {
    return (b >= 'a' && b <= 'z') || (b >= 'A' && b <= 'Z') || (b >= '0' && b <= '9');
}

bool is_ascii_letter(std::uint8_t b) noexcept {
    return (b >= 'a' && b <= 'z') || (b >= 'A' && b <= 'Z');
}

bool bytes_match(std::uint8_t a, std::uint8_t b) noexcept {
    if (is_ascii_letter(a) && is_ascii_letter(b))
        return (a | 32) == (b | 32);
    return a == b;
}

bool is_valid_byte(
    std::uint8_t b,
    const std::unordered_set<std::uint8_t>* custom_delimiter_bytes) noexcept {
    if (!is_ascii_alnum(b))
        return false;
    if (custom_delimiter_bytes && !custom_delimiter_bytes->empty()
        && custom_delimiter_bytes->count(b) != 0)
        return false;
    return true;
}

/// Decodes one UTF-8 codepoint; advances `i` past the sequence. Returns false on invalid UTF-8.
bool utf8_next(std::string_view s, std::size_t& i, char32_t& cp) noexcept {
    if (i >= s.size())
        return false;
    const auto c0 = static_cast<unsigned char>(s[i]);
    if (c0 <= 0x7F) {
        cp = c0;
        ++i;
        return true;
    }
    if ((c0 & 0xE0) == 0xC0) {
        if (i + 1 >= s.size())
            return false;
        const auto c1 = static_cast<unsigned char>(s[i + 1]);
        if ((c1 & 0xC0) != 0x80)
            return false;
        cp = ((c0 & 0x1F) << 6) | (c1 & 0x3F);
        if (cp < 0x80)
            return false; // overlong
        i += 2;
        return true;
    }
    if ((c0 & 0xF0) == 0xE0) {
        if (i + 2 >= s.size())
            return false;
        const auto c1 = static_cast<unsigned char>(s[i + 1]);
        const auto c2 = static_cast<unsigned char>(s[i + 2]);
        if ((c1 & 0xC0) != 0x80 || (c2 & 0xC0) != 0x80)
            return false;
        cp = ((c0 & 0x0F) << 12) | ((c1 & 0x3F) << 6) | (c2 & 0x3F);
        if (cp < 0x800)
            return false;
        i += 3;
        return true;
    }
    if ((c0 & 0xF8) == 0xF0) {
        if (i + 3 >= s.size())
            return false;
        const auto c1 = static_cast<unsigned char>(s[i + 1]);
        const auto c2 = static_cast<unsigned char>(s[i + 2]);
        const auto c3 = static_cast<unsigned char>(s[i + 3]);
        if ((c1 & 0xC0) != 0x80 || (c2 & 0xC0) != 0x80 || (c3 & 0xC0) != 0x80)
            return false;
        cp = ((c0 & 0x07) << 18) | ((c1 & 0x3F) << 12) | ((c2 & 0x3F) << 6) | (c3 & 0x3F);
        if (cp < 0x10000)
            return false;
        i += 4;
        return true;
    }
    return false;
}

} // namespace

palindrome_exception::palindrome_exception(std::string code, std::string message)
    : code_(std::move(code)), message_(std::move(message)) {}

const char* palindrome_exception::what() const noexcept {
    return message_.c_str();
}

const std::string& palindrome_exception::error_code() const noexcept {
    return code_;
}

bool from_bytes(
    std::span<const std::uint8_t> data,
    const std::unordered_set<std::uint8_t>* custom_delimiter_bytes) {
    std::size_t l = 0;
    std::size_t r = data.size();
    if (r == 0)
        return true;
    --r;
    while (true) {
        while (l <= r && !is_valid_byte(data[l], custom_delimiter_bytes))
            ++l;
        while (l <= r && !is_valid_byte(data[r], custom_delimiter_bytes))
            --r;
        if (l >= r)
            return true;
        if (!bytes_match(data[l], data[r]))
            return false;
        ++l;
        --r;
    }
}

bool from_string(
    std::string_view utf8,
    const std::unordered_set<std::uint8_t>* custom_delimiter_bytes) {
    std::size_t i = 0;
    while (i < utf8.size()) {
        char32_t cp = 0;
        if (!utf8_next(utf8, i, cp))
            throw std::runtime_error("invalid UTF-8 in from_string");
        if (cp > 0x7F) {
            throw palindrome_exception(
                "NON_ASCII_STRING_INPUT",
                "Input contains a scalar value > U+007F.");
        }
    }
    std::vector<std::uint8_t> bytes(utf8.begin(), utf8.end());
    return from_bytes(bytes, custom_delimiter_bytes);
}

} // namespace palindrome
