#include <IsPalindrome.hpp>

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

} // namespace

palindrome_exception::palindrome_exception(std::string code, std::string message)
    : code_(std::move(code)), message_(std::move(message)) {}

const char* palindrome_exception::what() const noexcept {
    return message_.c_str();
}

const std::string& palindrome_exception::error_code() const noexcept {
    return code_;
}

bool is_palindrome(
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

bool is_palindrome_from_utf8(
    std::string_view utf8,
    const std::unordered_set<std::uint8_t>* custom_delimiter_bytes) {
    for (unsigned char b : utf8) {
        if (b > 0x7F) {
            throw palindrome_exception(
                "NON_ASCII_STRING_INPUT",
                "Input contains a scalar value > U+007F.");
        }
    }
    std::vector<std::uint8_t> bytes(utf8.begin(), utf8.end());
    return is_palindrome(bytes, custom_delimiter_bytes);
}

} // namespace palindrome
