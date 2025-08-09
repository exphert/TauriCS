/**
 * @file This file contains globally accessible helper classes, shared state,
 * and security verification logic for all C# native libraries.
 */

using System.ComponentModel;
using System.Diagnostics;
using System.Linq; // Required for LINQ's .Any() method
using System.Runtime.InteropServices;

namespace Globals;

/// <summary>
/// A static class containing shared state and functions that can be accessed
/// from any native library. This is useful for maintaining application-wide state.
/// </summary>
public static class Shared
{
    /// <summary>
    /// A simple static counter to demonstrate shared state.
    /// It increments every time GetGlobalMessage is called.
    /// </summary>
    public static int AccessCounter { get; set; } = 0;

    /// <summary>
    /// A shared function that can be called from any native library.
    /// It increments the global access counter and returns a message.
    /// </summary>
    /// <returns>A formatted string including the current access count.</returns>
    public static string GetGlobalMessage()
    {
        AccessCounter++;
        return $"This is a shared message from Globals. Access count: {AccessCounter}";
    }
}

/// <summary>
/// A helper class for dynamically loading and invoking functions from external
/// native DLLs at runtime using P/Invoke (NativeLibrary).
/// </summary>
public static class NativeLoader
{
    /// <summary>
    /// A cache to store the handles of already loaded libraries.
    /// This prevents the same DLL from being loaded multiple times, improving performance.
    /// The key is the library name (e.g., "ExternalUtility.dll"), and the value is its handle.
    /// </summary>
    private static readonly Dictionary<string, IntPtr> _loadedLibraries = new();

    /// <summary>
    /// A generic delegate for a function that takes two integers and returns an integer.
    /// This can be used for simple calculation functions from external libraries.
    /// You can define more delegate types here for different function signatures.
    /// </summary>
    public delegate int IntCalculationFunc(int a, int b);

    /// <summary>
    /// Loads a native library (if not already cached) and returns a callable delegate
    /// for a specified function within it.
    /// </summary>
    /// <typeparam name="T">The type of the delegate representing the function's signature.</typeparam>
    /// <param name="libraryName">The name of the DLL file (e.g., "ExternalUtility.dll").</param>
    /// <param name="functionName">The name of the function to load from the DLL.</param>
    /// <returns>A callable delegate of type T.</returns>
    public static T LoadFunction<T>(string libraryName, string functionName) where T : Delegate
    {
        // Ensure the library name ends with .dll for consistency.
        if (!libraryName.EndsWith(".dll"))
        {
            libraryName += ".dll";
        }

        IntPtr libraryHandle;
        // Check the cache first to see if the library is already loaded.
        if (!_loadedLibraries.TryGetValue(libraryName, out libraryHandle))
        {
            // If not in the cache, load the library using the modern NativeLibrary API.
            // It will automatically search in the application's base directory (where all our DLLs are).
            libraryHandle = NativeLibrary.Load(libraryName);
            // Add the handle to the cache for future calls.
            _loadedLibraries[libraryName] = libraryHandle;
            Console.WriteLine($"[NativeLoader] Loaded external library: {libraryName}");
        }

        // Get the memory address (pointer) of the exported function.
        var functionPtr = NativeLibrary.GetExport(libraryHandle, functionName);

        // Convert the raw function pointer into a type-safe, callable .NET delegate.
        return Marshal.GetDelegateForFunctionPointer<T>(functionPtr);
    }
}


/// <summary>
/// Provides security-related functions, such as verifying the integrity of the host process.
/// </summary>
public static class Security
{
    /// <summary>
    /// Verifies that this DLL is running inside one of the allowed host processes.
    /// This is more reliable than checking the parent process.
    /// </summary>
    /// <param name="allowedProcessNames">An array of allowed executable filenames (e.g., ["taurics.exe"]).</param>
    /// <returns>A tuple containing the verification result and the detected host process name.</returns>
    public static (bool IsVerified, string DetectedProcessName) VerifyCurrentProcess(string[] allowedProcessNames)
    {
        try
        {
            var currentProcess = Process.GetCurrentProcess();
            // Ensure we always have the .exe for consistent comparison
            var currentProcessName = currentProcess.ProcessName.EndsWith(".exe") ? currentProcess.ProcessName : currentProcess.ProcessName + ".exe";

            // ** DEBUGGING LINE **
            // This will print the detected host process name to the console every time a check is made.
            Console.WriteLine($"[Security] Verifying current process. Detected: '{currentProcessName}'");

            bool isAllowed = allowedProcessNames.Any(allowedName =>
                string.Equals(currentProcessName, allowedName, StringComparison.OrdinalIgnoreCase));

            return (isAllowed, currentProcessName);
        }
        catch (Exception ex)
        {
            Console.WriteLine($"[Security] Process verification failed: {ex.Message}");
            return (false, "Error");
        }
    }
}
