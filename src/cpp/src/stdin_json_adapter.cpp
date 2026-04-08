// Stdin JSON adapter for is_palindrome_cli --impl cpp (JSON on stdin).

#include <IsPalindrome.hpp>

#include <cstdlib>
#include <iostream>
#include <vector>
#include <nlohmann/json.hpp>
#include <string>
#include <unordered_set>

int main() {
    std::string s;
    {
        std::istreambuf_iterator<char> it(std::cin), end;
        s.assign(it, end);
    }
    if (s.empty()) {
        std::cerr << "empty stdin\n";
        return 2;
    }
    nlohmann::json j = nlohmann::json::parse(s, nullptr, false);
    if (j.is_discarded()) {
        std::cerr << "invalid json\n";
        return 2;
    }
    std::unordered_set<std::uint8_t> custom_set;
    if (j.contains("custom") && j["custom"].is_array()) {
        for (const auto& x : j["custom"]) {
            custom_set.insert(static_cast<std::uint8_t>(x.get<int>()));
        }
    }
    const std::unordered_set<std::uint8_t>* custom_ptr =
        custom_set.empty() ? nullptr : &custom_set;

    const std::string mode = j.at("mode").get<std::string>();
    if (mode == "hex") {
        const std::string hex = j.at("hex").get<std::string>();
        std::vector<std::uint8_t> data(hex.size() / 2);
        for (std::size_t i = 0; i < data.size(); ++i) {
            data[i] = static_cast<std::uint8_t>(
                std::stoul(hex.substr(i * 2, 2), nullptr, 16));
        }
        const bool r = palindrome::is_palindrome(data, custom_ptr);
        std::cout << (r ? "true\n" : "false\n");
        return r ? 0 : 1;
    }
    if (mode == "string") {
        const std::string text = j.at("text").get<std::string>();
        try {
            const bool r = palindrome::is_palindrome_from_utf8(text, custom_ptr);
            std::cout << (r ? "true\n" : "false\n");
            return r ? 0 : 1;
        } catch (const palindrome::palindrome_exception& ex) {
            std::cerr << ex.error_code() << "\n";
            std::cerr << ex.what() << "\n";
            return 2;
        }
    }
    std::cerr << "unknown mode\n";
    return 2;
}
