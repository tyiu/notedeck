use std::sync::{Arc, RwLock, Mutex};
use fluent::FluentArgs;
use fluent_resmgr::ResourceManager;
use fluent_langneg::negotiate_languages;
use unic_langid::LanguageIdentifier;
use std::path::Path;
use fluent::{FluentBundle, FluentResource};

/// Manages localization resources and provides localized strings
pub struct LocalizationManager {
    /// The fluent resource manager (wrapped in Mutex for thread safety)
    resmgr: Arc<Mutex<ResourceManager>>,
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
        // The resname should include the .ftl extension
        let path_scheme = format!("{}/{{locale}}/{{resname}}.ftl", resource_dir.display());
        tracing::info!("Creating ResourceManager with path scheme: {}", path_scheme);
        let resmgr = Arc::new(Mutex::new(ResourceManager::new(path_scheme)));
        
        // Default to English (US)
        let default_locale: LanguageIdentifier = "en-US".parse().map_err(|e| format!("Locale parse error: {e:?}"))?;
        let fallback_locale = default_locale.clone();
        
        // Check if pseudolocale is enabled via environment variable
        let enable_pseudolocale = std::env::var("NOTEDECK_PSEUDOLOCALE").is_ok();
        
        // Build available locales list
        let mut available_locales = vec![default_locale.clone()];
        
        // Add en-XA if pseudolocale is enabled
        if enable_pseudolocale {
            let pseudolocale: LanguageIdentifier = "en-XA".parse().map_err(|e| format!("Pseudolocale parse error: {e:?}"))?;
            available_locales.push(pseudolocale);
            tracing::info!("Pseudolocale (en-XA) enabled via NOTEDECK_PSEUDOLOCALE environment variable");
        }
        
        Ok(Self {
            resmgr,
            current_locale: RwLock::new(default_locale),
            available_locales,
            fallback_locale,
        })
    }
    
    /// Gets a localized string by its ID
    pub fn get_string(&self, id: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        tracing::debug!("Getting string '{}' for locale '{}'", id, self.get_current_locale()?);
        let result = self.get_string_with_args(id, None);
        if let Err(ref e) = result {
            tracing::error!("Failed to get string '{}': {}", id, e);
        }
        result
    }
    
    /// Gets a localized string by its ID with optional arguments
    pub fn get_string_with_args(
        &self,
        id: &str,
        args: Option<&FluentArgs>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let locale = self.current_locale.read().map_err(|e| format!("Lock error: {e}"))?;
        
        tracing::debug!("Getting string '{}' for locale '{}'", id, locale);
        
        // Reconstruct the expected path for the FTL file
        let expected_path = format!(
            "/Users/tyiu/Library/Application Support/notedeck/i18n/{}/main.ftl",
            locale
        );
        tracing::debug!("Expected path for bundle: {}", expected_path);
        
        // Try to open the file directly
        match std::fs::File::open(&expected_path) {
            Ok(_) => tracing::info!("Direct file open succeeded: {}", expected_path),
            Err(e) => {
                tracing::error!("Direct file open failed: {} ({})", expected_path, e);
                return Err(format!("Failed to open FTL file: {}", e).into());
            }
        }
        
        // Load the FTL file directly instead of using ResourceManager
        let ftl_string = std::fs::read_to_string(&expected_path)
            .map_err(|e| format!("Failed to read FTL file: {}", e))?;
        
        tracing::debug!("Successfully read FTL file, content length: {}", ftl_string.len());
        
        // Create a bundle directly from the FTL content
        let mut bundle = FluentBundle::new(vec![locale.clone()]);
        bundle.add_resource(
            FluentResource::try_new(ftl_string)
                .map_err(|e| format!("Failed to parse FTL content: {:?}", e))?
        ).map_err(|e| format!("Failed to add resource to bundle: {:?}", e))?;
        
        tracing::debug!("Successfully created bundle and added resource");
        
        // Get the message
        let message = bundle
            .get_message(id)
            .ok_or_else(|| format!("Message not found: {}", id))?;
        
        tracing::debug!("Successfully found message for id: {}", id);
        
        let pattern = message
            .value()
            .ok_or_else(|| format!("Message has no value: {}", id))?;
        
        tracing::debug!("Successfully got pattern for id: {}", id);
        
        // Format the message
        let mut errors = Vec::new();
        let result = bundle.format_pattern(pattern, args, &mut errors);
        
        if !errors.is_empty() {
            tracing::warn!("Localization errors for {}: {:?}", id, errors);
        }
        
        tracing::debug!("Successfully got string '{}' = '{}'", id, result);
        Ok(result.into_owned())
    }
    
    /// Sets the current locale
    pub fn set_locale(&self, locale: LanguageIdentifier) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Attempting to set locale to: {}", locale);
        tracing::info!("Available locales: {:?}", self.available_locales);
        
        // Validate that the locale is available
        if !self.available_locales.contains(&locale) {
            tracing::error!("Locale {} is not available. Available locales: {:?}", locale, self.available_locales);
            return Err(format!("Locale {} is not available", locale).into());
        }
        
        let mut current = self.current_locale.write().map_err(|e| format!("Lock error: {e}"))?;
        tracing::info!("Switching locale from {} to {}", *current, locale);
        *current = locale.clone();
        tracing::info!("Successfully set locale to: {}", locale);
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
#[derive(Clone)]
pub struct LocalizationContext {
    /// The localization manager
    manager: Arc<LocalizationManager>,
}

impl LocalizationContext {
    /// Creates a new LocalizationContext
    pub fn new(manager: Arc<LocalizationManager>) -> Self {
        let context = Self { manager };
        
        // Auto-switch to pseudolocale if environment variable is set
        if std::env::var("NOTEDECK_PSEUDOLOCALE").is_ok() {
            tracing::info!("NOTEDECK_PSEUDOLOCALE environment variable detected");
            if let Ok(pseudolocale) = "en-XA".parse::<LanguageIdentifier>() {
                tracing::info!("Attempting to switch to pseudolocale: {}", pseudolocale);
                if let Err(e) = context.set_locale(pseudolocale) {
                    tracing::warn!("Failed to switch to pseudolocale: {}", e);
                } else {
                    tracing::info!("Automatically switched to pseudolocale (en-XA)");
                }
            } else {
                tracing::error!("Failed to parse en-XA as LanguageIdentifier");
            }
        } else {
            tracing::info!("NOTEDECK_PSEUDOLOCALE environment variable not set");
        }
        
        context
    }
    
    /// Gets a localized string by its ID
    pub fn get_string(&self, id: &str) -> Option<String> {
        self.manager
            .get_string(id)
            .ok()
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
        self.get_string(id).unwrap_or_else(|| format!("[MISSING: {}]", id))
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
