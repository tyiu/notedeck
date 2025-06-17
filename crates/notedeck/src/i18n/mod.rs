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
use tracing::info;

/// Global localization context for easy access from anywhere
static GLOBAL_I18N: OnceCell<Arc<LocalizationContext>> = OnceCell::new();

/// Initialize the global localization context
pub fn init_global_i18n(context: LocalizationContext) {
    info!("Initializing global i18n context");
    let _ = GLOBAL_I18N.set(Arc::new(context));
    info!("Global i18n context initialized successfully");
}

/// Get the global localization context
pub fn get_global_i18n() -> Option<Arc<LocalizationContext>> {
    GLOBAL_I18N.get().cloned()
}

/// Macro for getting localized strings
/// Uses the English string as the key and falls back to the English text if no translation is found
/// Optional comment parameter provides context for translators
#[macro_export]
macro_rules! tr {
    ($key:expr) => {
        {
            if let Some(i18n) = $crate::i18n::get_global_i18n() {
                i18n.get_string($key).unwrap_or_else(|| $key.to_string())
            } else {
                $key.to_string() // Fallback to English text if i18n not initialized
            }
        }
    };
    ($key:expr, $comment:expr) => {
        {
            if let Some(i18n) = $crate::i18n::get_global_i18n() {
                i18n.get_string($key).unwrap_or_else(|| $key.to_string())
            } else {
                $key.to_string() // Fallback to English text if i18n not initialized
            }
        }
    };
}

/// Macro for getting localized strings with context
/// The context is used to generate a more specific key for better translation
/// Optional comment parameter provides additional context for translators
#[macro_export]
macro_rules! tr_with_context {
    ($key:expr, $($param:expr => $value:expr),*) => {
        {
            if let Some(i18n) = $crate::i18n::get_global_i18n() {
                let mut args = $crate::i18n::FluentArgs::new();
                $(
                    args.set($param, $value);
                )*
                i18n.get_string_with_args($key, Some(&args))
            } else {
                // Fallback: replace placeholders with values
                let mut result = $key.to_string();
                $(
                    result = result.replace(&format!("{{{}}}", $param), &$value.to_string());
                )*
                result
            }
        }
    };
    ($key:expr, $context:expr) => {
        {
            let context_key = format!("{}#{}", $key, $context);
            if let Some(i18n) = $crate::i18n::get_global_i18n() {
                i18n.get_string(&context_key).unwrap_or_else(|| $key.to_string())
            } else {
                $key.to_string() // Fallback to English text if i18n not initialized
            }
        }
    };
    ($key:expr, $context:expr, $comment:expr) => {
        {
            let context_key = format!("{}#{}", $key, $context);
            if let Some(i18n) = $crate::i18n::get_global_i18n() {
                i18n.get_string(&context_key).unwrap_or_else(|| $key.to_string())
            } else {
                $key.to_string() // Fallback to English text if i18n not initialized
            }
        }
    };
}

/// Macro for getting localized pluralized strings with count
/// Uses the English string as the key and falls back to the English text if no translation is found
/// Optional comment parameter provides context for translators
#[macro_export]
macro_rules! tr_plural {
    ($key:expr, $count:expr) => {
        {
            if let Some(i18n) = $crate::i18n::get_global_i18n() {
                let mut args = $crate::i18n::FluentArgs::new();
                args.set("count", $count);
                i18n.get_string_with_args($key, Some(&args)).unwrap_or_else(|| {
                    $key.replace("$count", &$count.to_string())
                })
            } else {
                $key.replace("$count", &$count.to_string())
            }
        }
    };
    ($key:expr, $count:expr, $comment:expr) => {
        $crate::tr_plural!($key, $count)
    };
} 