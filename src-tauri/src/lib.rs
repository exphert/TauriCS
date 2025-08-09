// Crate attributes
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

/**
 * @file This file is the main entry point for the Rust core of the Tauri application.
 * It is responsible for:
 * 1. Defining the interface to the C# native libraries (FFI).
 * 2. Dynamically loading all C# libraries at startup.
 * 3. Managing the loaded libraries in a shared state.
 * 4. Exposing Tauri commands that the frontend can call to interact with the C# backend.
 */

// --- Crate Imports ---
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, Manager, State};

// --- Foreign Function Interface (FFI) Definitions ---
// These `type` aliases define the exact function signatures that Rust expects
// to find in the C# native libraries. They must match the C# delegates and function exports.

// A function that takes no arguments and returns a pointer to a C-style string.
type GetNativeNameFunc = unsafe extern "C" fn() -> *mut c_char;
// A function that takes a pointer to a JSON string and returns a pointer to a result string.
type ExecuteFunc = unsafe extern "C" fn(*const c_char) -> *mut c_char;
// A function for streaming, taking a JSON string and a pointer to a Rust callback function.
type ExecuteStreamingFunc = unsafe extern "C" fn(*const c_char, unsafe extern "C" fn(*mut c_char));
// A function for calling external DLLs.
type ExecuteExternalFunc = unsafe extern "C" fn(*const c_char) -> *mut c_char;
// A function that takes a string pointer and frees the memory allocated by C#.
type FreeStringFunc = unsafe extern "C" fn(*mut c_char);

// --- State Management Structs ---

/// <summary>
/// Holds the loaded symbols (function pointers) for a single dynamically loaded native library.
/// The 'static lifetime ensures these symbols are valid for the entire duration of the application.
/// </summary>
struct LoadedNative {
    execute: Symbol<'static, ExecuteFunc>,
    execute_streaming: Symbol<'static, ExecuteStreamingFunc>,
    execute_external: Symbol<'static, ExecuteExternalFunc>,
    free_string: Symbol<'static, FreeStringFunc>,
}

/// <summary>
/// The main state manager for all native libraries.
/// It uses a Mutex-protected HashMap to safely store and access loaded libraries from multiple threads.
/// The key is the lowercase name of the library (e.g., "sample"), and the value is a thread-safe smart pointer (Arc) to the LoadedNative struct.
/// </summary>
struct NativeManager {
    natives: Mutex<HashMap<String, Arc<LoadedNative>>>,
}

// --- Tauri Command Implementations ---

/// <summary>
/// Command for standard request-response calls.
/// </summary>
#[tauri::command]
fn call_backend(
    native_name: String,
    json_data: String,
    state: State<NativeManager>,
) -> Result<String, String> {
    // Lock the mutex to safely access the shared HashMap.
    let natives = state.natives.lock().unwrap();
    // Find the requested library by its name.
    if let Some(native) = natives.get(&native_name.to_lowercase()) {
        // Convert the Rust String to a C-compatible, null-terminated string.
        let json_data_c = CString::new(json_data).map_err(|e| e.to_string())?;

        // Call the C# 'execute' function via its function pointer. This is an unsafe operation.
        let result_ptr = unsafe { (native.execute)(json_data_c.as_ptr()) };
        if result_ptr.is_null() {
            return Err("Native function returned a null pointer.".to_string());
        }

        // Convert the C-string pointer back into a Rust String.
        let result_str = unsafe { CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };
        // Call the C# 'free_string' function to prevent memory leaks.
        unsafe {
            (native.free_string)(result_ptr);
        }
        Ok(result_str)
    } else {
        Err(format!("Native library '{}' not found.", native_name))
    }
}

/// <summary>
/// Command for calls that involve an external native library.
/// </summary>
#[tauri::command]
fn call_backend_external(
    native_name: String,
    json_data: String,
    state: State<NativeManager>,
) -> Result<String, String> {
    let natives = state.natives.lock().unwrap();
    if let Some(native) = natives.get(&native_name.to_lowercase()) {
        let json_data_c = CString::new(json_data).map_err(|e| e.to_string())?;
        let result_ptr = unsafe { (native.execute_external)(json_data_c.as_ptr()) };
        if result_ptr.is_null() {
            return Err("External call function returned a null pointer.".to_string());
        }
        let result_str = unsafe { CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };
        unsafe {
            (native.free_string)(result_ptr);
        }
        Ok(result_str)
    } else {
        Err(format!("Native library '{}' not found.", native_name))
    }
}

// --- Streaming Logic ---

// Global mutable static variables to allow the C# callback to communicate with the Tauri app.
// This is inherently unsafe and requires careful handling.
static mut APP_HANDLE: Option<AppHandle> = None;
static mut FREE_STRING_FN: Option<FreeStringFunc> = None;

/// <summary>
/// The callback function that Rust passes to C#. C# calls this function to send progress updates.
/// It must be marked `#[no_mangle]` and `unsafe extern "C"` to be callable from C#.
/// </summary>
#[no_mangle]
unsafe extern "C" fn progress_callback(message_ptr: *mut c_char) {
    // Safely access the global APP_HANDLE.
    if let Some(handle) = (*(&raw const APP_HANDLE)).as_ref() {
        let message = CStr::from_ptr(message_ptr).to_string_lossy().to_string();
        // Use the handle to emit a Tauri event to the frontend.
        handle.emit("csharp-stream", message).unwrap();
    }
    // Safely access and call the global free_string function to prevent memory leaks.
    if let Some(free_fn) = *(&raw const FREE_STRING_FN) {
        free_fn(message_ptr);
    }
}

/// <summary>
/// Command to start a long-running, streaming task.
/// </summary>
#[tauri::command]
fn start_streaming_task(
    native_name: String,
    json_data: String,
    state: State<NativeManager>,
    app_handle: AppHandle,
) {
    let natives = state.natives.lock().unwrap();
    if let Some(native) = natives.get(&native_name.to_lowercase()) {
        // Clone the thread-safe pointer to the library to move it into the new thread.
        let native = Arc::clone(native);
        let json_data_c = CString::new(json_data).unwrap();

        // Store the app handle and the free_string function globally so the callback can access them.
        unsafe {
            APP_HANDLE = Some(app_handle.clone());
            FREE_STRING_FN = Some(*native.free_string);
        }

        // Spawn a new thread to run the C# task, preventing the UI from freezing.
        thread::spawn(move || unsafe {
            (native.execute_streaming)(json_data_c.as_ptr(), progress_callback);
        });
    }
}

// --- Application Entry Point ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Initialize and manage the NativeManager state.
        .manage(NativeManager { natives: Mutex::new(HashMap::new()) })
        // The setup hook runs once when the application starts.
        .setup(|app| {
            // Determine the correct path to the 'natives' directory.
            let natives_dir = app.path().resource_dir().unwrap().join("natives");
            let state: tauri::State<NativeManager> = app.state();
            let mut natives = state.natives.lock().unwrap();

            // Scan the 'natives' directory for all .dll files.
            if let Ok(entries) = std::fs::read_dir(natives_dir) {
                for entry in entries.filter_map(Result::ok) {
                    if entry.path().extension().map_or(false, |e| e == "dll") {
                        // Use a closure to handle potential errors gracefully without crashing the app.
                        let result: Result<(), Box<dyn std::error::Error>> = (|| unsafe {
                            let lib = Library::new(entry.path())?;
                            // "Leak" the library to give it a 'static lifetime. This is safe because
                            // the libraries are meant to live for the entire application's duration.
                            let lib = Box::leak(Box::new(lib));

                            // Load all required functions. The '?' operator will cause an early return
                            // if any function is not found, which is caught by the error handler below.
                            let get_name: Symbol<GetNativeNameFunc> = lib.get(b"get_native_name\0")?;
                            let free_string_for_name: Symbol<FreeStringFunc> = lib.get(b"free_string\0")?;

                            let name_ptr = get_name();
                            let name = CStr::from_ptr(name_ptr).to_string_lossy().into_owned();
                            free_string_for_name(name_ptr);

                            let loaded_native = LoadedNative {
                                execute: lib.get(b"execute\0")?,
                                execute_streaming: lib.get(b"execute_streaming\0")?,
                                execute_external: lib.get(b"execute_external\0")?,
                                free_string: lib.get(b"free_string\0")?,
                            };

                            println!("Dynamically loaded tri-mode library: {}", name);
                            // Insert the fully loaded library into the state manager.
                            natives.insert(name, Arc::new(loaded_native));
                            Ok(())
                        })();

                        // If the closure returned an error, it means the DLL was not a valid main library.
                        if let Err(e) = result {
                            println!("Note: Could not load {:?} as a main library (might be a utility DLL): {}", entry.path(), e);
                        }
                    }
                }
            }
            Ok(())
        })
        // Register all the Tauri commands so the frontend can call them.
        .invoke_handler(tauri::generate_handler![call_backend, start_streaming_task, call_backend_external])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
