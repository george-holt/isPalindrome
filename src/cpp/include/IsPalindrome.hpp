#pragma once

#include <cstdint>
#include <exception>
#include <span>
#include <string>
#include <string_view>
#include <unordered_set>

namespace palindrome {

/// Thrown when the string API sees a Unicode scalar &gt; U+007F (SPEC §3).
class palindrome_exception : public std::exception {
public:
    palindrome_exception(std::string code, std::string message);

    [[nodiscard]] const char* what() const noexcept override;
    [[nodiscard]] const std::string& error_code() const noexcept;

private:
    std::string code_;
    std::string message_;
};

/// Optional extra delimiter bytes (non-null, non-empty set). Null or empty ⇒ default ASCII alnum only (SPEC §2).
[[nodiscard]] bool is_palindrome(
    std::span<const std::uint8_t> data,
    const std::unordered_set<std::uint8_t>* custom_delimiter_bytes = nullptr);

/// UTF-8 byte view: rejects any byte &gt; U+007F (ASCII UTF-8 only) with \ref palindrome_exception `NON_ASCII_STRING_INPUT` (same code for malformed multi-byte prefixes).
[[nodiscard]] bool is_palindrome_from_utf8(
    std::string_view utf8,
    const std::unordered_set<std::uint8_t>* custom_delimiter_bytes = nullptr);

} // namespace palindrome
