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