using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Text;
using System.Text.Json;
#if BAZEL_TEST
using Bazel;
#endif
using IsPalindrome;

#if DOTNET_TEST
using NUnit.Framework;
#endif

/// <summary>Shared manifest driver: Bazel <c>csharp_test</c> (BAZEL_TEST) or <c>dotnet test</c> (DOTNET_TEST + coverlet).</summary>
internal static class AcceptanceManifestDriver
{
    internal static int RunAll()
    {
        try
        {
            var path = ResolveManifestPath();
            var json = File.ReadAllText(path);
            using var doc = JsonDocument.Parse(json);
            var root = doc.RootElement;
            foreach (var caseEl in root.GetProperty("cases").EnumerateArray())
            {
                if (!caseEl.TryGetProperty("expected", out _))
                    continue;
                if (!AppliesToCs(caseEl))
                    continue;
                RunCase(caseEl);
            }

            return 0;
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine(ex);
            return 1;
        }
    }

    static bool AppliesToCs(JsonElement caseEl)
    {
        if (!caseEl.TryGetProperty("applies_to", out var arr) || arr.ValueKind != JsonValueKind.Array)
            return true;
        foreach (var x in arr.EnumerateArray())
        {
            if (x.GetString() == "cs")
                return true;
        }

        return false;
    }

    static void RunCase(JsonElement caseEl)
    {
        var id = caseEl.GetProperty("id").GetString()!;
        var custom = ParseCustomBytes(caseEl);
        var exp = caseEl.GetProperty("expected");
        var kind = exp.GetProperty("kind").GetString()!;

        try
        {
            if (UsesStringApi(caseEl))
            {
                string s;
                if (caseEl.TryGetProperty("input_unicode_scalar", out var usc))
                    s = UnicodeScalarToCharString(usc.GetString()!);
                else
                    s = caseEl.GetProperty("input_ascii").GetString()!;
                try
                {
                    var got = Palindrome.IsPalindromeFromUtf8(s, custom);
                    if (kind == "error")
                        throw new InvalidOperationException(
                            $"case {id}: expected error {exp.GetProperty("code").GetString()}, got bool {got}");
                    AssertBool(id, exp, got);
                }
                catch (PalindromeException ex)
                {
                    if (kind != "error")
                        throw new InvalidOperationException($"case {id}: unexpected PalindromeException", ex);
                    var want = exp.GetProperty("code").GetString();
                    if (ex.ErrorCode != want)
                        throw new InvalidOperationException(
                            $"case {id}: expected error code {want}, got {ex.ErrorCode}");
                }

                return;
            }

            ReadOnlySpan<byte> data;
            if (caseEl.TryGetProperty("input_ascii", out var ia))
                data = Encoding.Latin1.GetBytes(ia.GetString()!);
            else if (caseEl.TryGetProperty("input_hex", out var ih))
                data = DecodeHex(ih.GetString()!);
            else
                throw new InvalidOperationException($"case {id}: no input_ascii, input_hex, or string-api fields");

            var gotByte = Palindrome.IsPalindrome(data, custom);
            if (kind != "boolean")
                throw new InvalidOperationException($"case {id}: expected boolean result, got kind {kind}");
            AssertBool(id, exp, gotByte);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"case {id} failed", ex);
        }
    }

    static void AssertBool(string id, JsonElement exp, bool got)
    {
        var want = exp.GetProperty("value").GetBoolean();
        if (got != want)
            throw new InvalidOperationException($"case {id}: expected {want}, got {got}");
    }

    static bool UsesStringApi(JsonElement caseEl)
    {
        if (caseEl.TryGetProperty("input_unicode_scalar", out _))
            return true;
        return caseEl.TryGetProperty("category", out var c) && c.GetString() == "string_api";
    }

    static IReadOnlySet<byte>? ParseCustomBytes(JsonElement caseEl)
    {
        if (!caseEl.TryGetProperty("options", out var opts))
            return null;
        if (!opts.TryGetProperty("invalid_mode", out var im) || im.GetString() != "custom")
            return null;
        if (!opts.TryGetProperty("invalid_bytes_hex", out var arr) || arr.ValueKind != JsonValueKind.Array)
            return null;
        var set = new HashSet<byte>();
        foreach (var x in arr.EnumerateArray())
            set.Add((byte)Convert.ToInt32(x.GetString()!, 16));
        return set.Count == 0 ? null : set;
    }

    static string UnicodeScalarToCharString(string spec)
    {
        var p = spec.Trim().ToUpperInvariant();
        if (!p.StartsWith("U+", StringComparison.Ordinal))
            throw new FormatException(spec);
        var cp = int.Parse(p[2..], NumberStyles.HexNumber);
        return char.ConvertFromUtf32(cp);
    }

    static byte[] DecodeHex(string hex)
    {
        var outb = new byte[hex.Length / 2];
        for (var i = 0; i < outb.Length; i++)
            outb[i] = byte.Parse(hex.AsSpan(i * 2, 2), NumberStyles.HexNumber);
        return outb;
    }

    static string ResolveManifestPath()
    {
#if BAZEL_TEST
        var runfilesDir = Environment.GetEnvironmentVariable("RUNFILES_DIR")
                          ?? Environment.GetEnvironmentVariable("TEST_SRCDIR");
        var ws = Environment.GetEnvironmentVariable("TEST_WORKSPACE") ?? "_main";
        if (!string.IsNullOrEmpty(runfilesDir))
        {
            foreach (var candidate in new[]
                     {
                         Path.Combine(runfilesDir, ws, "fixtures", "acceptance_manifest.json"),
                         Path.Combine(runfilesDir, "fixtures", "acceptance_manifest.json"),
                     })
            {
                if (File.Exists(candidate))
                    return candidate;
            }
        }

        var rf = Runfiles.Create();
        foreach (var logical in new[]
                 {
                     $"{ws}/fixtures/acceptance_manifest.json",
                     "_main/fixtures/acceptance_manifest.json",
                     "fixtures/acceptance_manifest.json",
                 })
        {
            var p = rf.Rlocation(logical);
            if (File.Exists(p))
                return p;
        }

        throw new FileNotFoundException("acceptance_manifest.json (Bazel runfiles)");
#else
        var baseDir = AppContext.BaseDirectory;
        var copied = Path.Combine(baseDir, "fixtures", "acceptance_manifest.json");
        if (File.Exists(copied))
            return copied;
        throw new FileNotFoundException("acceptance_manifest.json (dotnet test output)");
#endif
    }
}

#if BAZEL_TEST
internal static class Program
{
    private static int Main() => AcceptanceManifestDriver.RunAll();
}
#endif

#if DOTNET_TEST
[TestFixture]
internal sealed class AcceptanceManifestNUnit
{
    [Test]
    public void AllCasesFromManifest()
    {
        Assert.That(AcceptanceManifestDriver.RunAll(), Is.EqualTo(0));
    }
}
#endif
