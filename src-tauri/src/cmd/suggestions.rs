//! Tauri commands for managing break suggestions.
//!
//! This module provides commands to read and write break suggestion configurations
//! for different languages.

use tauri::{AppHandle, State, command};

use crate::core::suggestions::{
    SharedSuggestions, SuggestionsConfig, save_suggestions_internal,
};

/// Retrieves the suggestions configuration.
///
/// # Errors
///
/// This function does not return errors in normal operation.
#[command]
pub async fn get_suggestions(
    state: State<'_, SharedSuggestions>,
) -> Result<SuggestionsConfig, String> {
    let suggestions = state.read().await;
    Ok(suggestions.clone())
}

/// Saves the suggestions configuration.
///
/// # Errors
///
/// Returns an error if saving the configuration file fails.
#[command]
pub async fn save_suggestions(
    app: AppHandle,
    state: State<'_, SharedSuggestions>,
    config: SuggestionsConfig,
) -> Result<(), String> {
    let saved_config = save_suggestions_internal(&app, &config)
        .await
        .map_err(|e| e.to_string())?;

    {
        let mut suggestions = state.write().await;
        *suggestions = saved_config;
    }

    tracing::info!("Suggestions updated successfully");
    Ok(())
}
