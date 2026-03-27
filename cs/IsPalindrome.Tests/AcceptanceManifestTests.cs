using System.Globalization;
using System.Text;
using System.Text.Json;
using Xunit;

namespace IsPalindrome.Tests;

/// <summary>Runs shared cases from <c>fixtures/acceptance_manifest.json</c> (SPEC §4).</summary>
public sealed class AcceptanceManifestTests
{
    [Fact]
    public void All_manifest_cases_match_Spec()
    {
        var path = Path.Combine(AppContext.BaseDirectory, "fixtures", "acceptance_manifest.json");
        Assert.True(File.Exists(path), $"Missing manifest at {path}");

        using var doc = JsonDocument.Parse(File.ReadAllText(path));
        var root = doc.RootElement;
        foreach (var caseEl in root.GetProperty("cases").EnumerateArray())
        {
            var id = caseEl.GetProperty("id").GetString()!;
            if (id == "pal-stream-note-001")
                continue;
            if (!AppliesToCSharp(caseEl))
                continue;

            var options = ParseCustomDelimiters(caseEl);
            var expected = caseEl.GetProperty("expected");

            if (expected.GetProperty("kind").GetString() == "boolean")
            {
                var want = expected.GetProperty("value").GetBoolean();
                var input = BuildInput(caseEl, stringApi: caseEl.TryGetProperty("applies_to", out _));
                var got = input.Kind switch
                {
                    InputKind.Bytes => Palindrome.FromBytes(input.Bytes.Span, options),
                    InputKind.String => Palindrome.FromString(input.Text!, options),
                    _ => throw new InvalidOperationException(),
                };
                Assert.Equal(want, got);
            }
            else if (expected.GetProperty("kind").GetString() == "error")
            {
                var code = expected.GetProperty("code").GetString()!;
                var input = BuildInput(caseEl, stringApi: true);
                var ex = Assert.Throws<PalindromeException>(() =>
                {
                    _ = input.Kind switch
                    {
                        InputKind.Bytes => Palindrome.FromBytes(input.Bytes.Span, options),
                        InputKind.String => Palindrome.FromString(input.Text!, options),
                        _ => throw new InvalidOperationException(),
                    };
                });
                Assert.Equal(code, ex.ErrorCode);
            }
        }
    }

    static bool AppliesToCSharp(JsonElement caseEl)
    {
        if (!caseEl.TryGetProperty("applies_to", out var arr))
            return true;
        foreach (var x in arr.EnumerateArray())
        {
            var s = x.GetString();
            if (s is "csharp" or "dotnet" or "cs")
                return true;
        }
        return false;
    }

    static IReadOnlySet<byte>? ParseCustomDelimiters(JsonElement caseEl)
    {
        if (!caseEl.TryGetProperty("options", out var opts))
            return null;
        if (!opts.TryGetProperty("invalid_mode", out var modeEl) || modeEl.GetString() != "custom")
            return null;
        if (!opts.TryGetProperty("invalid_bytes_hex", out var hexArr))
            return null;
        var set = new HashSet<byte>();
        foreach (var el in hexArr.EnumerateArray())
        {
            var hex = el.GetString()!;
            Assert.Equal(2, hex.Length);
            set.Add(Convert.ToByte(hex, 16));
        }
        return set.Count == 0 ? null : set;
    }

    enum InputKind { Bytes, String }

    readonly struct BuiltInput
    {
        public InputKind Kind { get; init; }
        public ReadOnlyMemory<byte> Bytes { get; init; }
        public string? Text { get; init; }
    }

    static BuiltInput BuildInput(JsonElement caseEl, bool stringApi)
    {
        if (caseEl.TryGetProperty("input_ascii", out var ascii))
        {
            var s = ascii.GetString()!;
            if (stringApi)
                return new BuiltInput { Kind = InputKind.String, Text = s };
            return new BuiltInput { Kind = InputKind.Bytes, Bytes = Encoding.Latin1.GetBytes(s) };
        }

        if (caseEl.TryGetProperty("input_hex", out var hex))
            return new BuiltInput
            {
                Kind = InputKind.Bytes,
                Bytes = DecodeHex(hex.GetString()!),
            };

        if (caseEl.TryGetProperty("input_unicode_scalar", out var scalarEl))
        {
            var scalar = scalarEl.GetString()!;
            var text = UnicodeScalarToString(scalar);
            return new BuiltInput { Kind = InputKind.String, Text = text };
        }

        throw new InvalidOperationException("Case has no recognized input field.");
    }

    static ReadOnlyMemory<byte> DecodeHex(string hex)
    {
        if (hex.Length % 2 != 0)
            throw new InvalidOperationException("Odd hex length.");
        var bytes = new byte[hex.Length / 2];
        for (int i = 0; i < bytes.Length; i++)
            bytes[i] = byte.Parse(hex.AsSpan(i * 2, 2), NumberStyles.HexNumber, CultureInfo.InvariantCulture);
        return bytes;
    }

    static string UnicodeScalarToString(string manifestScalar)
    {
        // "U+00E9"
        var prefix = "U+";
        if (!manifestScalar.StartsWith(prefix, StringComparison.OrdinalIgnoreCase))
            throw new InvalidOperationException(manifestScalar);
        var cp = int.Parse(
            manifestScalar.AsSpan(prefix.Length),
            NumberStyles.HexNumber,
            CultureInfo.InvariantCulture);
        return cp <= 0xFFFF
            ? ((char)cp).ToString()
            : char.ConvertFromUtf32(cp);
    }
}
