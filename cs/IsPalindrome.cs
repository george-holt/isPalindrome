using System.Text;

namespace IsPalindrome;

/// <summary>Thrown when the string API receives a scalar &gt; U+007F (SPEC §3).</summary>
public sealed class PalindromeException : Exception
{
    public string ErrorCode { get; }

    public PalindromeException(string errorCode, string message)
        : base(message) =>
        ErrorCode = errorCode;
}

/// <summary>Byte and string palindrome checks (dual cursors, ASCII alnum validity, optional extra delimiter bytes).</summary>
public static class Palindrome
{
    /// <param name="customDelimiterBytes">When non-null and non-empty, these alnum bytes are treated as delimiters (skipped). Null or empty ⇒ default rule only (SPEC §2).</param>
    public static bool FromBytes(ReadOnlySpan<byte> data, IReadOnlySet<byte>? customDelimiterBytes = null)
    {
        int l = 0;
        int r = data.Length - 1;
        while (true)
        {
            while (l <= r && !IsValidByte(data[l], customDelimiterBytes))
                l++;
            while (l <= r && !IsValidByte(data[r], customDelimiterBytes))
                r--;
            if (l >= r)
                return true;
            if (!BytesMatch(data[l], data[r]))
                return false;
            l++;
            r--;
        }
    }

    /// <param name="customDelimiterBytes">Same semantics as <see cref="FromBytes"/>.</param>
    public static bool FromString(string text, IReadOnlySet<byte>? customDelimiterBytes = null)
    {
        for (int i = 0; i < text.Length; i++)
        {
            if (text[i] > '\x7F')
            {
                throw new PalindromeException(
                    "NON_ASCII_STRING_INPUT",
                    "Input contains a scalar value > U+007F.");
            }
        }

        if (text.Length == 0)
            return FromBytes(ReadOnlySpan<byte>.Empty, customDelimiterBytes);

        var bytes = Encoding.Latin1.GetBytes(text);
        return FromBytes(bytes, customDelimiterBytes);
    }

    static bool IsValidByte(byte b, IReadOnlySet<byte>? customDelimiterBytes)
    {
        if (!IsAsciiAlphanumeric(b))
            return false;
        if (customDelimiterBytes is { Count: > 0 } && customDelimiterBytes.Contains(b))
            return false;
        return true;
    }

    static bool IsAsciiAlphanumeric(byte b) =>
        b is >= (byte)'a' and <= (byte)'z'
            or >= (byte)'A' and <= (byte)'Z'
            or >= (byte)'0' and <= (byte)'9';

    static bool BytesMatch(byte a, byte b)
    {
        if (IsAsciiLetter(a) && IsAsciiLetter(b))
            return (a | 32) == (b | 32);
        return a == b;
    }

    static bool IsAsciiLetter(byte b) =>
        b is >= (byte)'a' and <= (byte)'z' or >= (byte)'A' and <= (byte)'Z';
}
