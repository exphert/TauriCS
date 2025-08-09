# Tauri C# Native Library Template

This project is a powerful template for building cross-platform desktop applications using [Tauri](https://v2.tauri.app/) (v2) for the frontend and a high-performance C# backend. It showcases a robust architecture where the C# backend is compiled using **.NET Native AOT** and dynamically loaded by the Rust core as native libraries.

This provides the safety and security of Rust, the UI flexibility of web technologies, and the full power of the .NET ecosystem for your business logic, all with native performance.

.            |  .
:-------------------------:|:-------------------------:
![](/.img/1.png)  |  ![](/.img/2.png) 
![](/.img/3.png)  |  ![](/.img/4.png) 

## Architecture Overview

The core of this template is a decoupled architecture that allows for a clean separation of concerns and a highly extensible backend.

`JS Frontend → Tauri Rust Core → Dynamic C# Native Libraries`

-   **Frontend:** A standard web frontend (HTML, CSS, JS) running inside a Tauri webview.
-   **Rust Core (`src-tauri`):** Acts as a secure and lightweight bridge. It does not contain business logic. Instead, it dynamically loads all compiled C# libraries from a `natives` folder at startup and exposes them to the frontend through Tauri commands.
-   **C# Backend (`src-csharp`):**
    -   **Native Libraries:** Each piece of business logic is encapsulated in its own C# project, compiled using .NET Native AOT into a standalone `.dll`. These are true native libraries, just like ones made with C++ or Rust.
    -   **Globals Library:** A shared C# project (`Globals`) can be used to store common variables, functions, security helpers, and data models accessible by all native libraries.

## Key Features

-   **Dynamic Native Library System:** Simply drop a compiled C# Native AOT `.dll` into the `natives` folder, and the application will load it at startup. No Rust code changes needed.
-   **Tri-Mode Communication:** Each library supports three distinct communication patterns out of the box:
    1.  **Request-Response:** A standard synchronous call for quick tasks.
    2.  **Streaming:** For long-running tasks, the C# backend can stream progress updates back to the frontend in real-time using Tauri events.
    3.  **External Native Calls:** A C# library can dynamically load and call functions from *other* native DLLs (including non-C# ones), acting as a bridge to existing native code.
-   **Built-in Security:** Native libraries are automatically protected by a current process check. This ensures they can only be loaded and executed by your main application (e.g., `taurics.exe`), preventing unauthorized use or hijacking of your DLLs.
-   **Automated Build Process:** A custom Rust build script (`build.rs`) automatically:
    -   Builds the shared `Globals` library.
    -   Finds all C# library projects in `src-csharp/Native/`.
    -   Publishes each one using .NET Native AOT.
    -   Copies the final `.dll` files to the correct directory for bundling.
-   **Library Scaffolding:** A simple Node.js script allows you to create a new, fully configured, and secure C# native library with a single command.
-   **Clean Bundling:** The final installed application has a clean directory structure with a single `natives` folder containing all backend libraries.

## Project Structure

```
.
├── natives/              # Staging area for custom/pre-compiled native DLLs.
├── src/                  # Frontend code (HTML, CSS, JS).
├── src-csharp/
│   ├── Globals/          # Shared C# code, functions, and security helpers.
│   └── Native/
│       ├── ExternalUtility/  # Example C# native External library (can be any compiled DLL).
│       ├── Sample/       # Example C# native library project.
│       └── ...           # Other native library projects.
├── src-tauri/
│   ├── build.rs          # The Rust build script that automates C# compilation.
│   ├── natives/          # (Auto-generated) Final location for DLLs to be bundled.
│   └── src/
│       └── lib.rs       # Rust core: loads libraries and handles commands.
├── .scripts/
│   └── make-native.mjs   # Scaffolding script for new C# libraries.
├── package.json
└── tauri.conf.json       # Tauri configuration.
```

## Getting Started

### Prerequisites

-   **Node.js and npm/yarn/pnpm:** [Install Node.js](https://nodejs.org/)
-   **Rust Toolchain:** [Install Rust](https://www.rust-lang.org/tools/install)
-   **.NET SDK:** (Recommended: .NET 8 or newer for Native AOT) [Install .NET SDK](https://dotnet.microsoft.com/download)
-   **Tauri CLI:** Run `cargo install tauri-cli`

### Installation & Running

1.  **Clone the repository:**
    ```bash
    git clone <your-repo-url>
    cd <your-repo-name>
    ```

2.  **Install frontend dependencies:**
    ```bash
    npm install
    ```

3.  **Run in development mode:**
    ```bash
    npm run tauri dev
    ```
    The first time you run this, `build.rs` will compile all C# projects, which may take a moment. Subsequent builds will be much faster.

4.  **Build for production:**
    ```bash
    npm run tauri build
    ```
    This will create a standalone installer for your application in `src-tauri/target/release/bundle/`.

## Creating a New Native Library

To add new functionality, you can easily create a new C# native library.

1.  Run the scaffolding script from the root of the project:
    ```bash
    npm run cs:make MyNewLibrary
    ```
    (Replace `MyNewLibrary` with your desired name in PascalCase).

2.  This will automatically create a new project at `src-csharp/Native/MyNewLibrary` with all the necessary files and configurations.

3.  Open the new `NativeEntry.cs` file. Here you can:
    -   Add your custom logic to the `Execute`, `ExecuteStreaming`, and `ExecuteExternal` methods.
    -   Configure the list of allowed current processes in the `ALLOWED_PROCESSES` array at the top of the file.

4.  Run `npm run tauri dev`. The build script will automatically find, compile, and include your new library in the application.
