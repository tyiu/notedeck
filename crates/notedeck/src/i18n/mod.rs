//! Internationalization (i18n) module for Notedeck
//! 
//! This module provides localization support using fluent and fluent-resmgr.
//! It handles loading translation files, managing locales, and providing
//! localized strings throughout the application.

pub mod manager;

pub use manager::LocalizationManager;
pub use manager::LocalizationContext;

/// Re-export commonly used types for convenience
pub use fluent::FluentArgs;
pub use fluent::FluentValue;
pub use unic_langid::LanguageIdentifier;

use once_cell::sync::OnceCell;
use std::sync::Arc;

/// Global localization context for easy access from anywhere
static GLOBAL_I18N: OnceCell<Arc<LocalizationContext>> = OnceCell::new();

/// Initialize the global localization context
pub fn init_global_i18n(context: LocalizationContext) {
    let _ = GLOBAL_I18N.set(Arc::new(context));
}

/// Get the global localization context
pub fn get_global_i18n() -> Option<Arc<LocalizationContext>> {
    GLOBAL_I18N.get().cloned()
}

/// Helper macro for easier string localization
/// 
/// Usage:
/// ```rust
/// // Simple string lookup
/// let text = tr!("action-reply-note");
/// 
/// // String with arguments
/// let text = tr!("welcome-message", &FluentArgs::from_iter([
///     ("name", "John".into())
/// ]));
/// ```
#[macro_export]
macro_rules! tr {
    ($id:expr) => {
        {
            if let Some(i18n) = notedeck::get_global_i18n() {
                i18n.get_localized_string($id)
            } else {
                format!("[MISSING: {}]", $id)
            }
        }
    };
    ($id:expr, $args:expr) => {
        {
            if let Some(i18n) = notedeck::get_global_i18n() {
                i18n.get_localized_string_with_args($id, Some($args))
            } else {
                format!("[MISSING: {}]", $id)
            }
        }
    };
} 