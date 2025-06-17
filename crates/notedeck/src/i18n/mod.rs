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

/// Helper macro for easier string localization
/// 
/// Usage:
/// ```rust
/// // Simple string lookup
/// let text = tr!(ctx, "action-reply-note");
/// 
/// // String with arguments
/// let text = tr!(ctx, "welcome-message", &FluentArgs::from_iter([
///     ("name", "John".into())
/// ]));
/// ```
#[macro_export]
macro_rules! tr {
    ($ctx:expr, $id:expr) => {
        $ctx.get_localized_string($id)
    };
    ($ctx:expr, $id:expr, $args:expr) => {
        $ctx.get_localized_string_with_args($id, Some($args))
    };
} 