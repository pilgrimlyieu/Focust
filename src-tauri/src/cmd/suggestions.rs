//! Tauri commands for managing break suggestions.
//!
//! This module provides commands to read and write break suggestion configurations
//! for different languages.

use tauri::{AppHandle, State, command};

use crate::core::suggestions::{
    SharedSuggestions, SuggestionsConfig, get_suggestions_for_language_internal,
    save_suggestions_internal,
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

/// Retrieves suggestions for a specific language.
///
/// # Errors
///
/// This function does not return errors in normal operation.
#[command]
pub async fn get_suggestions_for_language(
    language: String,
    state: State<'_, SharedSuggestions>,
) -> Result<Vec<String>, String> {
    let suggestions = state.read().await;
    Ok(get_suggestions_for_language_internal(
        &suggestions,
        &language,
    ))
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
    // Save to file
    save_suggestions_internal(&app, &config)
        .await
        .map_err(|e| e.to_string())?;

    // Update state
    {
        let mut suggestions = state.write().await;
        *suggestions = config;
    }

    tracing::info!("Suggestions updated successfully");
    Ok(())
}
