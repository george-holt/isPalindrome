enum InvalidMode
{
    Default,
    Custom,
}

sealed class PalindromeOptions
{
    public InvalidMode InvalidMode { get; init; } = InvalidMode.Default;

    public HashSet<byte>? CustomInvalidBytes { get; init; }

    public static PalindromeOptions Default { get; } = new();

    public void Validate()
    {
        if (InvalidMode != InvalidMode.Custom)
            return;
        if (CustomInvalidBytes is null || CustomInvalidBytes.Count == 0)
            throw new EmptyCustomInvalidSetException();
    }
}

sealed class EmptyCustomInvalidSetException : Exception
{
}

static class PalindromeCharacter
{
    public static bool IsValidCharacter(byte b, PalindromeOptions options)
    {
        if (options.InvalidMode == InvalidMode.Default)
            return IsAsciiAlphanumeric(b);
        return !options.CustomInvalidBytes!.Contains(b);
    }

    public static bool IsAsciiAlphanumeric(byte b) =>
        b is >= (byte)'a' and <= (byte)'z'
            or >= (byte)'A' and <= (byte)'Z'
            or >= (byte)'0' and <= (byte)'9';

    public static bool IsAsciiLetter(byte b) =>
        b is >= (byte)'a' and <= (byte)'z' or >= (byte)'A' and <= (byte)'Z';

    public static bool BytesMatch(byte a, byte b)
    {
        if (IsAsciiLetter(a) && IsAsciiLetter(b))
            return (a | 32) == (b | 32);
        return a == b;
    }
}

static class Foo
{
    public static bool Compare(byte[] data, PalindromeOptions options)
    {
        options.Validate();
        int L = 0;
        int R = data.Length - 1;
        while (true)
        {
            if (!TryAdvanceToNextValid(data, ref L, R, forward: true, options, out byte left))
                return true;
            if (!TryAdvanceToNextValid(data, ref R, L, forward: false, options, out byte right))
                return true;
            if (L >= R)
                return true;
            if (!PalindromeCharacter.BytesMatch(left, right))
                return false;
            L++;
            R--;
        }
    }

    // Moves index toward a valid byte without crossing boundaryInclusive (max index when forward, min when backward).
    static bool TryAdvanceToNextValid(
        byte[] data,
        ref int index,
        int boundaryInclusive,
        bool forward,
        PalindromeOptions options,
        out byte value)
    {
        if (forward)
        {
            while (index <= boundaryInclusive && !PalindromeCharacter.IsValidCharacter(data[index], options))
                index++;
            if (index > boundaryInclusive)
            {
                value = default;
                return false;
            }
        }
        else
        {
            while (index >= boundaryInclusive && !PalindromeCharacter.IsValidCharacter(data[index], options))
                index--;
            if (index < boundaryInclusive)
            {
                value = default;
                return false;
            }
        }

        value = data[index];
        return true;
    }
}
