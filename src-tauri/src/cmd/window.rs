use std::sync::Arc;
use tauri::{AppHandle, Listener, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::Mutex;

/// Open settings window (create if not exists, show if already exists)
#[tauri::command]
pub async fn open_settings_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    // Window exists, just show and focus it
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        tracing::debug!("Settings window already exists, showing and focusing");
        return Ok(());
    }

    tracing::info!("Creating new settings window");

    // Register event listener BEFORE creating the window
    let (tx, rx) = tokio::sync::oneshot::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));

    let tx_clone = tx.clone();
    let app_clone = app.clone();
    let unlisten = app.listen("settings-ready", move |_event| {
        tracing::info!("Received settings-ready event from frontend");
        let tx_clone = tx_clone.clone();
        tauri::async_runtime::spawn(async move {
            if let Some(sender) = tx_clone.lock().await.take() {
                let _ = sender.send(());
            }
        });
    });

    // Create window
    let _window =
        WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("settings.html".into()))
            .title("Focust - Settings")
            .inner_size(1400.0, 900.0)
            .center()
            .visible(false) // Start hidden to avoid showing blank window
            .build()
            .map_err(|e| e.to_string())?;

    tracing::info!("Settings window created, waiting for ready event...");

    // Wait for ready event with timeout
    let app_clone2 = app.clone();
    tauri::async_runtime::spawn(async move {
        let ready = tokio::time::timeout(tokio::time::Duration::from_millis(3000), rx).await;
        app_clone.unlisten(unlisten);

        match ready {
            Ok(Ok(_)) => {
                tracing::info!("Settings window content ready, showing window");
            }
            Ok(Err(e)) => {
                tracing::warn!("Error waiting for ready event: {e}, showing window anyway");
            }
            Err(_) => {
                tracing::warn!("Timeout waiting for ready event, showing window anyway");
            }
        }

        // Show the window now
        if let Some(win) = app_clone2.get_webview_window("settings") {
            let _ = win.show();
            let _ = win.set_focus();
        }
    });

    Ok(())
}
