using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Text;
using System.Text.Json;
using IsPalindrome;

var json = Console.In.ReadToEnd();
using var doc = JsonDocument.Parse(json);
var root = doc.RootElement;
var mode = root.GetProperty("mode").GetString()!;
var custom = ParseCustom(root);

if (mode == "hex")
{
    var hex = root.GetProperty("hex").GetString()!;
    var data = DecodeHex(hex);
    var r = Palindrome.IsPalindrome(data, custom);
    Console.Out.WriteLine(r ? "true" : "false");
    Environment.Exit(r ? 0 : 1);
}
else if (mode == "string")
{
    var text = root.GetProperty("text").GetString()!;
    try
    {
        var r = Palindrome.IsPalindromeFromUtf8(text, custom);
        Console.Out.WriteLine(r ? "true" : "false");
        Environment.Exit(r ? 0 : 1);
    }
    catch (PalindromeException ex)
    {
        Console.Error.WriteLine(ex.ErrorCode);
        Console.Error.WriteLine(ex.Message);
        Environment.Exit(2);
    }
}
else
{
    Console.Error.WriteLine("unknown mode");
    Environment.Exit(2);
}

static IReadOnlySet<byte>? ParseCustom(JsonElement root)
{
    if (!root.TryGetProperty("custom", out var arr) || arr.ValueKind != JsonValueKind.Array)
        return null;
    var set = new HashSet<byte>();
    foreach (var x in arr.EnumerateArray())
        set.Add((byte)x.GetInt32());
    return set.Count == 0 ? null : set;
}

static byte[] DecodeHex(string hex)
{
    var outb = new byte[hex.Length / 2];
    for (var i = 0; i < outb.Length; i++)
        outb[i] = byte.Parse(hex.AsSpan(i * 2, 2), NumberStyles.HexNumber);
    return outb;
}
