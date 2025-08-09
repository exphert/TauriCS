/**
 * @file This script handles all frontend logic for the Tauri C# application.
 * It sets up event listeners for the UI, communicates with the Rust backend via Tauri commands,
 * and displays the results.
 */

// Import the necessary functions from the Tauri API.
// 'invoke' is used to call Rust commands.
// 'listen' is used to subscribe to backend events.
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// This event listener ensures that the script runs only after the entire HTML document has been loaded and parsed.
window.addEventListener("DOMContentLoaded", async () => {
    // Get references to the interactive UI elements.
    const regularButton = document.querySelector("#regular-button");
    const externalButton = document.querySelector("#external-button");
    const streamButton = document.querySelector("#stream-button");
    const resultPre = document.querySelector("#result");

    // Variable to store the start time of a streaming task for performance measurement.
    let streamStartTime = 0;

    // --- Global Event Listener for Streaming ---
    // This listener is set up once and will handle all incoming stream data from the backend.
    await listen('csharp-stream', (event) => {
        const message = event.payload;
        console.log("Stream data received:", message);

        // The C# backend sends a special message to signal that the stream has ended.
        if (message === "__STREAM_END__") {
            const duration = (Date.now() - streamStartTime) / 1000;
            resultPre.textContent += `\n--- Streaming Task Finished in ${duration.toFixed(2)}s ---`;
            return;
        }

        // Append the received progress message to the result area.
        resultPre.textContent += message;
    });

    // --- Button 1: Standard Request-Response ---
    regularButton.addEventListener("click", async () => {
        resultPre.textContent = "Sending regular request...";
        try {
            const payload = { requestType: "regular" };

            // Call the 'call_backend' command in Rust, passing the library name and JSON data.
            const responseString = await invoke('call_backend', {
                nativeName: 'sample',
                jsonData: JSON.stringify(payload)
            });

            // Parse the JSON string response and display it nicely formatted.
            const responseObject = JSON.parse(responseString);
            resultPre.textContent = JSON.stringify(responseObject, null, 2);

        } catch (error) {
            resultPre.textContent = `Error: ${error}`;
        }
    });

    // --- Button 2: External Library Call ---
    externalButton.addEventListener("click", async () => {
        resultPre.textContent = "Sending external call request...";
        try {
            // Call the 'call_backend_external' command, which triggers the C# function
            // that loads another native DLL.
            const responseString = await invoke('call_backend_external', {
                nativeName: 'sample',
                jsonData: JSON.stringify({ requestType: "external" })
            });

            const responseObject = JSON.parse(responseString);
            resultPre.textContent = JSON.stringify(responseObject, null, 2);

        } catch (error) {
            resultPre.textContent = `Error: ${error}`;
        }
    });

    // --- Button 3: Streaming Task ---
    streamButton.addEventListener("click", () => {
        resultPre.textContent = ""; // Clear previous results.
        streamStartTime = Date.now(); // Record the start time.

        const payload = { requestType: "streaming" };

        // Call the 'start_streaming_task' command. This command returns immediately
        // and does not need to be awaited. The results will arrive via the 'csharp-stream' event listener.
        invoke('start_streaming_task', {
            nativeName: 'sample',
            jsonData: JSON.stringify(payload)
        }).catch(error => {
            // This catch block handles errors that might occur while *starting* the task.
            resultPre.textContent = `Error starting task: ${error}`;
        });
    });
});
