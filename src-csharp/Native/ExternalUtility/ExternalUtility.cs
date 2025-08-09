/**
 * @file This file defines a simple, standalone utility library.
 * It is compiled as a Native AOT DLL and is intended to be loaded and called
 * by other C# native libraries within the application, not directly by Rust.
 */

using System.Runtime.InteropServices;

namespace ExternalUtility;

/// <summary>
/// A static class containing simple utility functions that can be exported.
/// </summary>
public static class ExternalUtility
{
    /// <summary>
    /// A simple function that we will call from another plugin.
    /// It takes two integers, adds them, and returns the result.
    /// </summary>
    /// <param name="a">The first integer.</param>
    /// <param name="b">The second integer.</param>
    /// <returns>The sum of a and b.</returns>
    [UnmanagedCallersOnly(EntryPoint = "perform_calculation")]
    public static int PerformCalculation(int a, int b)
    {
        Console.WriteLine($"[ExternalUtility] Performing calculation: {a} + {b}");
        return a + b;
    }

    // Note: We don't need other functions like get_name or execute here,
    // because this library is not intended to be a main plugin called directly by Rust.
    // It's a utility library meant to be called by other plugins.
}
