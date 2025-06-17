use std::sync::{Arc, RwLock};
use fluent::FluentArgs;
use fluent_resmgr::ResourceManager;
use fluent_langneg::negotiate_languages;
use unic_langid::LanguageIdentifier;
use std::path::Path;

/// Manages localization resources and provides localized strings
pub struct LocalizationManager {
    /// The fluent resource manager
    resmgr: Arc<ResourceManager>,
    /// Current locale
    current_locale: RwLock<LanguageIdentifier>,
    /// Available locales
    available_locales: Vec<LanguageIdentifier>,
    /// Fallback locale
    fallback_locale: LanguageIdentifier,
}

impl LocalizationManager {
    /// Creates a new LocalizationManager with the specified resource directory
    pub fn new(resource_dir: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize the resource manager with a path scheme
        let path_scheme = format!("{}/{{locale}}/{{resname}}", resource_dir.display());
        let resmgr = Arc::new(ResourceManager::new(path_scheme));
        
        // Default to English (US)
        let default_locale: LanguageIdentifier = "en-US".parse().map_err(|e| format!("Locale parse error: {e:?}"))?;
        let fallback_locale = default_locale.clone();
        
        // For now, we'll start with just English
        // In the future, this could be discovered from the locales directory
        let available_locales = vec![default_locale.clone()];
        
        Ok(Self {
            resmgr,
            current_locale: RwLock::new(default_locale),
            available_locales,
            fallback_locale,
        })
    }
    
    /// Gets a localized string by its ID
    pub fn get_string(&self, id: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.get_string_with_args(id, None)
    }
    
    /// Gets a localized string by its ID with optional arguments
    pub fn get_string_with_args(
        &self,
        id: &str,
        args: Option<&FluentArgs>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let locale = self.current_locale.read().map_err(|e| format!("Lock error: {e}"))?;
        
        // Get the bundle for the current locale
        let bundle = self.resmgr.get_bundle(vec![locale.clone()], vec!["main".to_string()]);
        
        // Handle errors from get_bundle
        if let Err(errors) = &bundle {
            tracing::warn!("Failed to get bundle for locale {}: {:?}", locale, errors);
            return Err(format!("Failed to get bundle: {:?}", errors).into());
        }
        
        let bundle = bundle.unwrap();
        
        // Get the message
        let message = bundle
            .get_message(id)
            .ok_or_else(|| format!("Message not found: {}", id))?;
        
        let pattern = message
            .value()
            .ok_or_else(|| format!("Message has no value: {}", id))?;
        
        // Format the message
        let mut errors = Vec::new();
        let result = bundle.format_pattern(pattern, args, &mut errors);
        
        if !errors.is_empty() {
            tracing::warn!("Localization errors for {}: {:?}", id, errors);
        }
        
        Ok(result.into_owned())
    }
    
    /// Sets the current locale
    pub fn set_locale(&self, locale: LanguageIdentifier) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Validate that the locale is available
        if !self.available_locales.contains(&locale) {
            return Err(format!("Locale {} is not available", locale).into());
        }
        
        let mut current = self.current_locale.write().map_err(|e| format!("Lock error: {e}"))?;
        *current = locale;
        Ok(())
    }
    
    /// Gets the current locale
    pub fn get_current_locale(&self) -> Result<LanguageIdentifier, Box<dyn std::error::Error + Send + Sync>> {
        let current = self.current_locale.read().map_err(|e| format!("Lock error: {e}"))?;
        Ok(current.clone())
    }
    
    /// Gets all available locales
    pub fn get_available_locales(&self) -> &[LanguageIdentifier] {
        &self.available_locales
    }
    
    /// Gets the fallback locale
    pub fn get_fallback_locale(&self) -> &LanguageIdentifier {
        &self.fallback_locale
    }
    
    /// Negotiates the best locale from a list of preferred locales
    pub fn negotiate_locale(&self, preferred: &[LanguageIdentifier]) -> LanguageIdentifier {
        let available = self.available_locales.clone();
        let negotiated = negotiate_languages(
            preferred,
            &available,
            Some(&self.fallback_locale),
            fluent_langneg::NegotiationStrategy::Filtering,
        );
        negotiated.first().map_or(self.fallback_locale.clone(), |v| (*v).clone())
    }
}

/// Context for sharing localization across the application
pub struct LocalizationContext {
    /// The localization manager
    manager: Arc<LocalizationManager>,
}

impl LocalizationContext {
    /// Creates a new LocalizationContext
    pub fn new(manager: Arc<LocalizationManager>) -> Self {
        Self { manager }
    }
    
    /// Gets a localized string by its ID
    pub fn get_string(&self, id: &str) -> String {
        self.manager
            .get_string(id)
            .unwrap_or_else(|_| format!("[MISSING: {}]", id))
    }
    
    /// Gets a localized string by its ID with optional arguments
    pub fn get_string_with_args(&self, id: &str, args: Option<&FluentArgs>) -> String {
        self.manager
            .get_string_with_args(id, args)
            .unwrap_or_else(|_| format!("[MISSING: {}]", id))
    }
    
    /// Sets the current locale
    pub fn set_locale(&self, locale: LanguageIdentifier) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.manager.set_locale(locale)
    }
    
    /// Gets the current locale
    pub fn get_current_locale(&self) -> Result<LanguageIdentifier, Box<dyn std::error::Error + Send + Sync>> {
        self.manager.get_current_locale()
    }
    
    /// Gets the underlying manager
    pub fn manager(&self) -> &Arc<LocalizationManager> {
        &self.manager
    }
}

/// Trait for objects that can be localized
pub trait Localizable {
    /// Gets a localized string by its ID
    fn get_localized_string(&self, id: &str) -> String;
    
    /// Gets a localized string by its ID with optional arguments
    fn get_localized_string_with_args(&self, id: &str, args: Option<&FluentArgs>) -> String;
}

impl Localizable for LocalizationContext {
    fn get_localized_string(&self, id: &str) -> String {
        self.get_string(id)
    }
    
    fn get_localized_string_with_args(&self, id: &str, args: Option<&FluentArgs>) -> String {
        self.get_string_with_args(id, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_localization_manager_creation() {
        let temp_dir = std::env::temp_dir().join("notedeck_i18n_test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        let manager = LocalizationManager::new(&temp_dir);
        assert!(manager.is_ok());
        
        // Cleanup
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
    
    #[test]
    fn test_locale_management() {
        let temp_dir = std::env::temp_dir().join("notedeck_i18n_test2");
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        let manager = LocalizationManager::new(&temp_dir).unwrap();
        
        // Test default locale
        let current = manager.get_current_locale().unwrap();
        assert_eq!(current.to_string(), "en-US");
        
        // Test available locales
        let available = manager.get_available_locales();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].to_string(), "en-US");
        
        // Cleanup
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
} 