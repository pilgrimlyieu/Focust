use tauri::{AppHandle, State, command};

use crate::core::suggestions::SuggestionsConfig;

/// Global shared suggestions state
pub type SharedSuggestions = tokio::sync::RwLock<SuggestionsConfig>;

/// Get suggestions configuration
#[command]
pub async fn get_suggestions(
    state: State<'_, SharedSuggestions>,
) -> Result<SuggestionsConfig, String> {
    let suggestions = state.read().await;
    Ok(suggestions.clone())
}

/// Get suggestions for a specific language
#[command]
pub async fn get_suggestions_for_language(
    language: String,
    state: State<'_, SharedSuggestions>,
) -> Result<Vec<String>, String> {
    let suggestions = state.read().await;
    Ok(crate::core::suggestions::get_suggestions_for_language(
        &suggestions,
        &language,
    ))
}

/// Save suggestions configuration
#[command]
pub async fn save_suggestions(
    app: AppHandle,
    state: State<'_, SharedSuggestions>,
    config: SuggestionsConfig,
) -> Result<(), String> {
    // Save to file
    crate::core::suggestions::save_suggestions(&app, &config)
        .await
        .map_err(|e| e.to_string())?;

    // Update state
    let mut suggestions = state.write().await;
    *suggestions = config;

    tracing::info!("Suggestions updated successfully");
    Ok(())
}
