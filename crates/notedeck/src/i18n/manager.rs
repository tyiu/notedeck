use super::{IntlError, IntlKey, IntlKeyBuf};
use fluent::{FluentArgs, FluentBundle, FluentResource};
use fluent_langneg::negotiate_languages;
use std::borrow::Cow;
use std::collections::HashMap;
use sys_locale;
use unic_langid::{langid, LanguageIdentifier};

const EN_US: LanguageIdentifier = langid!("en-US");
const EN_XA: LanguageIdentifier = langid!("en-XA");
const DE: LanguageIdentifier = langid!("de");
const ES_419: LanguageIdentifier = langid!("es-419");
const ES_ES: LanguageIdentifier = langid!("es-ES");
const FR: LanguageIdentifier = langid!("fr");
const PT_BR: LanguageIdentifier = langid!("pt-BR");
const TH: LanguageIdentifier = langid!("th");
const ZH_CN: LanguageIdentifier = langid!("zh-CN");
const ZH_TW: LanguageIdentifier = langid!("zh-TW");
const NUM_FTLS: usize = 10;

const EN_US_NATIVE_NAME: &str = "English (US)";
const EN_XA_NATIVE_NAME: &str = "Éñglísh (Pséúdólóçàlé)";
const DE_NATIVE_NAME: &str = "Deutsch";
const ES_419_NATIVE_NAME: &str = "Español (Latinoamérica)";
const ES_ES_NATIVE_NAME: &str = "Español (España)";
const FR_NATIVE_NAME: &str = "Français";
const PT_BR_NATIVE_NAME: &str = "Português (Brasil)";
const TH_NATIVE_NAME: &str = "ภาษาไทย";
const ZH_CN_NATIVE_NAME: &str = "简体中文";
const ZH_TW_NATIVE_NAME: &str = "繁體中文";

struct StaticBundle {
    identifier: LanguageIdentifier,
    ftl: &'static str,
}

const FTLS: [StaticBundle; NUM_FTLS] = [
    StaticBundle {
        identifier: EN_US,
        ftl: include_str!("../../../../assets/translations/en-US/main.ftl"),
    },
    StaticBundle {
        identifier: EN_XA,
        ftl: include_str!("../../../../assets/translations/en-XA/main.ftl"),
    },
    StaticBundle {
        identifier: DE,
        ftl: include_str!("../../../../assets/translations/de/main.ftl"),
    },
    StaticBundle {
        identifier: ES_419,
        ftl: include_str!("../../../../assets/translations/es-419/main.ftl"),
    },
    StaticBundle {
        identifier: ES_ES,
        ftl: include_str!("../../../../assets/translations/es-ES/main.ftl"),
    },
    StaticBundle {
        identifier: FR,
        ftl: include_str!("../../../../assets/translations/fr/main.ftl"),
    },
    StaticBundle {
        identifier: PT_BR,
        ftl: include_str!("../../../../assets/translations/pt-BR/main.ftl"),
    },
    StaticBundle {
        identifier: TH,
        ftl: include_str!("../../../../assets/translations/th/main.ftl"),
    },
    StaticBundle {
        identifier: ZH_CN,
        ftl: include_str!("../../../../assets/translations/zh-CN/main.ftl"),
    },
    StaticBundle {
        identifier: ZH_TW,
        ftl: include_str!("../../../../assets/translations/zh-TW/main.ftl"),
    },
];

type Bundle = FluentBundle<FluentResource>;

/// Manages localization resources and provides localized strings
pub struct Localization {
    /// Current locale
    current_locale: LanguageIdentifier,
    /// Available locales
    available_locales: Vec<LanguageIdentifier>,
    /// Fallback locale
    fallback_locale: LanguageIdentifier,
    /// Native names for locales
    locale_native_names: HashMap<LanguageIdentifier, String>,

    /// Cached string results per locale (only for strings without arguments)
    string_cache: HashMap<LanguageIdentifier, HashMap<String, String>>,
    /// Cached normalized keys
    normalized_key_cache: HashMap<String, IntlKeyBuf>,
    /// Bundles
    bundles: HashMap<LanguageIdentifier, Bundle>,

    use_isolating: bool,
}

impl Default for Localization {
    fn default() -> Self {
        // Build available locales list
        let available_locales = vec![
            EN_US.clone(),
            EN_XA.clone(),
            DE.clone(),
            ES_419.clone(),
            ES_ES.clone(),
            FR.clone(),
            PT_BR.clone(),
            TH.clone(),
            ZH_CN.clone(),
            ZH_TW.clone(),
        ];

        let locale_native_names = HashMap::from([
            (EN_US, EN_US_NATIVE_NAME.to_owned()),
            (EN_XA, EN_XA_NATIVE_NAME.to_owned()),
            (DE, DE_NATIVE_NAME.to_owned()),
            (ES_419, ES_419_NATIVE_NAME.to_owned()),
            (ES_ES, ES_ES_NATIVE_NAME.to_owned()),
            (FR, FR_NATIVE_NAME.to_owned()),
            (PT_BR, PT_BR_NATIVE_NAME.to_owned()),
            (TH, TH_NATIVE_NAME.to_owned()),
            (ZH_CN, ZH_CN_NATIVE_NAME.to_owned()),
            (ZH_TW, ZH_TW_NATIVE_NAME.to_owned()),
        ]);

        // Detect system locale and find best match
        let current_locale = Self::negotiate_system_locale_with_preferences(&available_locales);

        // Fallback locale is always EN_US
        let fallback_locale = EN_US.clone();

        tracing::info!(
            "Localization initialized - Selected locale: {}, Fallback: {}",
            current_locale,
            fallback_locale
        );

        Self {
            current_locale,
            available_locales,
            fallback_locale,
            locale_native_names,
            use_isolating: true,
            normalized_key_cache: HashMap::new(),
            string_cache: HashMap::new(),
            bundles: HashMap::new(),
        }
    }
}

impl Localization {
    /// Creates a new Localization with the specified resource directory
    pub fn new() -> Self {
        Localization::default()
    }

    /// Disable bidirectional isolation markers. mostly useful for tests
    pub fn no_bidi() -> Self {
        Localization {
            use_isolating: false,
            ..Localization::default()
        }
    }

    /// Extract just the language and region from locale string (e.g., "fr-FR-u-mu-celsius" -> "fr-FR")
    fn extract_language_region(locale_str: &str) -> String {
        // Split by '-' and analyze the parts
        let parts: Vec<&str> = locale_str.split('-').collect();

        if parts.len() >= 2 {
            // Check if the second part looks like a region
            let second_part = parts[1];
            if (second_part.len() >= 2) {
                format!("{}-{}", parts[0], parts[1])
            } else {
                // Second part is not a region, probably an extension (e.g., "u", "t", "x")
                // Just return the language part
                parts[0].to_string()
            }
        } else {
            // Only one part, return as is
            locale_str.to_string()
        }
    }

    /// Negotiate the best locale from all system preferences against available locales
    fn negotiate_system_locale_with_preferences(
        available_locales: &[LanguageIdentifier],
    ) -> LanguageIdentifier {
        // Get all system preferred locales in descending order
        let mut system_locales: Vec<String> = sys_locale::get_locales().collect();
        if system_locales.is_empty() {
            tracing::info!("No system locales detected, using fallback: en-US");
            return EN_US.clone();
        }

        tracing::info!("System preferred locales: {:?}", system_locales);

        // If we only got one locale, it might be that the system only returns the primary locale
        // In this case, we can try to add common fallbacks based on the detected locale
        if system_locales.len() == 1 {
            let primary = &system_locales[0];

            // Try to parse the primary locale, handling extensions
            let primary_lang = if let Ok(locale) = primary.parse::<LanguageIdentifier>() {
                locale.language.as_str().to_string()
            } else {
                // If parsing fails, try extracting language-region
                // let stripped = Self::extract_language_region(primary);
                // if let Ok(locale) = stripped.parse::<LanguageIdentifier>() {
                //     locale.language.as_str().to_string()
                // } else {
                    tracing::info!("Could not parse primary locale: {}", primary);
                    "unknown".to_string()
                // }
            };

            tracing::info!(
                "Only one system locale detected: {} (language: {})",
                primary,
                primary_lang
            );

            // Add common fallbacks for the detected language
            match primary_lang.as_str() {
                "uk" => {
                    // For Ukrainian, add common fallbacks
                    system_locales.push("es-ES".to_string());
                    system_locales.push("en-US".to_string());
                    tracing::info!("Added fallbacks for Ukrainian: {:?}", system_locales);
                }
                "es" => {
                    // For Spanish, add English fallback
                    system_locales.push("en-US".to_string());
                    tracing::info!("Added fallback for Spanish: {:?}", system_locales);
                }
                _ => {
                    // For other languages, add English fallback
                    system_locales.push("en-US".to_string());
                    tracing::info!("Added fallback for {}: {:?}", primary_lang, system_locales);
                }
            }
        }

        // Convert system locale strings to LanguageIdentifiers, handling extensions
        let mut parsed_system_locales = Vec::new();
        for locale_str in system_locales {
            // Try to parse the locale string directly first
            if let Ok(locale) = locale_str.parse::<LanguageIdentifier>() {
                parsed_system_locales.push(locale);
                continue;
            }

            // If parsing fails, try extracting just language-region
            // let stripped_locale = Self::extract_language_region(&locale_str);
            // if let Ok(locale) = stripped_locale.parse::<LanguageIdentifier>() {
            //     parsed_system_locales.push(locale);
            //     continue;
            // }

            tracing::info!("Failed to parse locale string: {}", locale_str);
        }

        if parsed_system_locales.is_empty() {
            tracing::info!("No valid system locales parsed, using fallback: en-US");
            return EN_US.clone();
        }

        // First try exact matches with fluent_langneg
        let fallback = &EN_US;
        let negotiated = negotiate_languages(
            &parsed_system_locales,
            available_locales,
            Some(fallback),
            fluent_langneg::NegotiationStrategy::Filtering,
        );

        if let Some(result) = negotiated.first() {
            tracing::info!(
                "Exact match found: {} from preferences: {:?}",
                result,
                parsed_system_locales
            );
            return (*result).clone();
        }

        // If no exact match, try language-only fallbacks
        tracing::info!("No exact matches found, trying language-only fallbacks");
        for system_locale in &parsed_system_locales {
            let system_lang = system_locale.language.as_str();

            // Look for any available locale with the same language
            for available_locale in available_locales {
                if available_locale.language.as_str() == system_lang {
                    tracing::debug!(
                        "Language match found: {} (system: {})",
                        available_locale,
                        system_locale
                    );
                    return available_locale.clone();
                }
            }
        }

        tracing::info!("No language matches found, using fallback: en-US");
        EN_US.clone()
    }

    /// Gets a localized string by its ID
    pub fn get_string(&mut self, id: IntlKey<'_>) -> Result<String, IntlError> {
        self.get_cached_string(id, None)
    }

    /// Load a fluent bundle given a language identifier. Only looks in the static
    /// ftl files baked into the binary
    fn load_bundle(lang: &LanguageIdentifier) -> Result<Bundle, IntlError> {
        for ftl in &FTLS {
            if &ftl.identifier == lang {
                let mut bundle = FluentBundle::new(vec![lang.to_owned()]);
                let resource = FluentResource::try_new(ftl.ftl.to_string());
                match resource {
                    Err((resource, errors)) => {
                        for error in errors {
                            tracing::error!("load_bundle ({lang}): {error}");
                        }

                        tracing::warn!("load_bundle ({}: loading bundle with errors", lang);
                        if let Err(errs) = bundle.add_resource(resource) {
                            for err in errs {
                                tracing::error!("adding resource: {err}");
                            }
                        }
                    }

                    Ok(resource) => {
                        tracing::info!("loaded {} bundle OK!", lang);
                        if let Err(errs) = bundle.add_resource(resource) {
                            for err in errs {
                                tracing::error!("adding resource 2: {err}");
                            }
                        }
                    }
                }

                return Ok(bundle);
            }
        }

        // no static ftl for this LanguageIdentifier
        Err(IntlError::NoFtl(lang.to_owned()))
    }

    fn get_bundle<'a>(&'a self, lang: &LanguageIdentifier) -> &'a Bundle {
        self.bundles
            .get(lang)
            .expect("make sure to call ensure_bundle!")
    }

    fn has_bundle(&self, lang: &LanguageIdentifier) -> bool {
        self.bundles.contains_key(lang)
    }

    fn try_load_bundle(&mut self, lang: &LanguageIdentifier) -> Result<(), IntlError> {
        let mut bundle = Self::load_bundle(lang)?;
        if !self.use_isolating {
            bundle.set_use_isolating(false);
        }
        self.bundles.insert(lang.to_owned(), bundle);
        Ok(())
    }

    pub fn normalized_ftl_key(&mut self, key: &str, comment: &str) -> IntlKeyBuf {
        match self.get_ftl_key(key) {
            Some(intl_key) => intl_key,
            None => {
                self.insert_ftl_key(key, comment);
                self.get_ftl_key(key).unwrap()
            }
        }
    }

    fn get_ftl_key(&self, cache_key: &str) -> Option<IntlKeyBuf> {
        self.normalized_key_cache.get(cache_key).cloned()
    }

    fn insert_ftl_key(&mut self, cache_key: &str, comment: &str) {
        let mut result = fixup_key(cache_key);

        // Ensure the key starts with a letter (Fluent requirement)
        if result.is_empty() || !result.chars().next().unwrap().is_ascii_alphabetic() {
            result = format!("k_{result}");
        }

        // If we have a comment, append a hash of it to reduce collisions
        let hash_str = format!("_{}", simple_hash(comment));
        result.push_str(&hash_str);

        tracing::debug!(
            "normalize_ftl_key: original='{}', final='{}'",
            cache_key,
            result
        );

        self.normalized_key_cache
            .insert(cache_key.to_owned(), IntlKeyBuf::new(result));
    }

    fn get_cached_string_no_args<'key>(
        &'key self,
        lang: &LanguageIdentifier,
        id: IntlKey<'key>,
    ) -> Result<Cow<'key, str>, IntlError> {
        // Try to get from string cache first
        if let Some(locale_cache) = self.string_cache.get(lang) {
            if let Some(cached_string) = locale_cache.get(id.as_str()) {
                /*
                tracing::trace!(
                    "Using cached string result for '{}' in locale: {}",
                    id,
                    &lang
                );
                */

                return Ok(Cow::Borrowed(cached_string));
            }
        }

        Err(IntlError::NotFound(id.to_owned()))
    }

    fn ensure_bundle(&mut self) -> Result<(), IntlError> {
        let locale = self.current_locale.clone();
        if !self.has_bundle(&locale) {
            match self.try_load_bundle(&locale) {
                Err(err) => {
                    tracing::warn!(
                        "tried to load bundle {} but failed with '{err}'. using fallback {}",
                        &locale,
                        &self.fallback_locale
                    );
                    self.try_load_bundle(&locale)
                        .expect("failed to load fallback bundle!?");

                    Ok(())
                }

                Ok(()) => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    fn get_current_bundle(&self) -> &Bundle {
        if self.has_bundle(&self.current_locale) {
            return self.get_bundle(&self.current_locale);
        }

        self.get_bundle(&self.fallback_locale)
    }

    /// Gets cached string result, or formats it and caches the result
    pub fn get_cached_string(
        &mut self,
        id: IntlKey<'_>,
        args: Option<&FluentArgs>,
    ) -> Result<String, IntlError> {
        self.ensure_bundle()?;

        if args.is_none() {
            if let Ok(result) = self.get_cached_string_no_args(&self.current_locale, id) {
                return Ok(result.to_string());
            }
        }

        let result = {
            let bundle = self.get_current_bundle();

            let message = bundle
                .get_message(id.as_str())
                .ok_or_else(|| IntlError::NotFound(id.to_owned()))?;

            let pattern = message
                .value()
                .ok_or_else(|| IntlError::NoValue(id.to_owned()))?;

            let mut errors = Vec::with_capacity(0);
            let result = bundle.format_pattern(pattern, args, &mut errors);

            if !errors.is_empty() {
                tracing::warn!("Localization errors for {}: {:?}", id, &errors);
            }

            result.to_string()
        };

        // Only cache simple strings without arguments
        // This prevents caching issues when the same message ID is used with different arguments
        if args.is_none() {
            self.cache_string(self.current_locale.clone(), id, result.as_str());
            tracing::debug!(
                "Cached string result for '{}' in locale: {}",
                id,
                &self.current_locale
            );
        } else {
            tracing::trace!("Not caching string '{}' due to arguments", id);
        }

        Ok(result)
    }

    pub fn cache_string<'a>(&mut self, locale: LanguageIdentifier, id: IntlKey<'a>, result: &str) {
        tracing::debug!("Cached string result for '{}' in locale: {}", id, &locale);
        let locale_cache = self.string_cache.entry(locale).or_default();
        locale_cache.insert(id.to_owned().to_string(), result.to_owned());
    }

    /// Sets the current locale
    pub fn set_locale(&mut self, locale: LanguageIdentifier) -> Result<(), IntlError> {
        tracing::info!("Attempting to set locale to: {}", locale);
        tracing::info!("Available locales: {:?}", self.available_locales);

        // Validate that the locale is available
        if !self.available_locales.contains(&locale) {
            tracing::error!(
                "Locale {} is not available. Available locales: {:?}",
                locale,
                self.available_locales
            );
            return Err(IntlError::LocaleNotAvailable(locale));
        }

        tracing::info!(
            "Switching locale from {} to {}",
            &self.current_locale,
            &locale
        );
        self.current_locale = locale;

        // Clear caches when locale changes since they are locale-specific
        self.string_cache.clear();
        tracing::debug!("String cache cleared due to locale change");

        Ok(())
    }

    /// Clears the parsed FluentResource cache (useful for development when FTL files change)
    pub fn clear_cache(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.bundles.clear();
        tracing::debug!("Parsed FluentResource cache cleared");

        self.string_cache.clear();
        tracing::debug!("String result cache cleared");

        Ok(())
    }

    /// Gets the current locale
    pub fn get_current_locale(&self) -> &LanguageIdentifier {
        &self.current_locale
    }

    /// Gets all available locales
    pub fn get_available_locales(&self) -> &[LanguageIdentifier] {
        &self.available_locales
    }

    /// Gets the fallback locale
    pub fn get_fallback_locale(&self) -> &LanguageIdentifier {
        &self.fallback_locale
    }

    pub fn get_locale_native_name(&self, locale: &LanguageIdentifier) -> Option<&str> {
        self.locale_native_names.get(locale).map(|s| s.as_str())
    }

    /// Gets cache statistics for monitoring performance
    pub fn get_cache_stats(&self) -> Result<CacheStats, Box<dyn std::error::Error + Send + Sync>> {
        let mut total_strings = 0;
        for locale_cache in self.string_cache.values() {
            total_strings += locale_cache.len();
        }

        Ok(CacheStats {
            resource_cache_size: self.bundles.len(),
            string_cache_size: total_strings,
            cached_locales: self.bundles.keys().cloned().collect(),
        })
    }

    /// Limits the string cache size to prevent memory growth
    pub fn limit_string_cache_size(
        &mut self,
        max_strings_per_locale: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for locale_cache in self.string_cache.values_mut() {
            if locale_cache.len() > max_strings_per_locale {
                // Remove oldest entries (simple approach: just clear and let it rebuild)
                // In a more sophisticated implementation, you might use an LRU cache
                locale_cache.clear();
                tracing::debug!("Cleared string cache for locale due to size limit");
            }
        }

        Ok(())
    }
}

/// Statistics about cache usage
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub resource_cache_size: usize,
    pub string_cache_size: usize,
    pub cached_locales: Vec<LanguageIdentifier>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_language_region() {
        // Test that we extract just language and region from various locale formats

        // Test locales with extensions
        let unicode_locale = "fr-FR-u-mu-celsius";
        let extracted = Localization::extract_language_region(unicode_locale);
        assert_eq!(extracted, "fr-FR");

        let transformed_locale = "en-US-t-0-abc123";
        let extracted = Localization::extract_language_region(transformed_locale);
        assert_eq!(extracted, "en-US");

        let private_locale = "de-DE-x-phonebk";
        let extracted = Localization::extract_language_region(private_locale);
        assert_eq!(extracted, "de-DE");

        // Test simple locale (no extensions)
        let simple_locale = "en-US";
        let extracted = Localization::extract_language_region(simple_locale);
        assert_eq!(extracted, "en-US");

        // Test language-only locale
        let lang_only = "en";
        let extracted = Localization::extract_language_region(lang_only);
        assert_eq!(extracted, "en");

        // Test language with extensions (no region)
        let lang_with_extensions = "fr-u-mu-celsius";
        let extracted = Localization::extract_language_region(lang_with_extensions);
        assert_eq!(extracted, "fr");

        // Test language with other extension types (no region)
        let lang_with_t_ext = "en-t-0-abc123";
        let extracted = Localization::extract_language_region(lang_with_t_ext);
        assert_eq!(extracted, "en");

        let lang_with_x_ext = "de-x-phonebk";
        let extracted = Localization::extract_language_region(lang_with_x_ext);
        assert_eq!(extracted, "de");

        // Test locale with numeric region code
        let numeric_region = "es-419-u-mu-celsius";
        let extracted = Localization::extract_language_region(numeric_region);
        assert_eq!(extracted, "es-419");

        // Test locale with 3-letter region code
        let three_letter_region = "en-USA-t-0-abc123";
        let extracted = Localization::extract_language_region(three_letter_region);
        assert_eq!(extracted, "en-USA");

        // Test locale with 2-letter region code
        let two_letter_region = "fr-FR-u-mu-celsius";
        let extracted = Localization::extract_language_region(two_letter_region);
        assert_eq!(extracted, "fr-FR");

        // Test complex locale with multiple parts
        let complex_locale = "zh-CN-u-ca-chinese-x-private";
        let extracted = Localization::extract_language_region(complex_locale);
        assert_eq!(extracted, "zh-CN");

        // Verify that extracted locales can be parsed
        let test_cases = ["fr-FR", "en-US", "de-DE", "en", "zh-CN"];
        for extracted in test_cases {
            if let Ok(locale) = extracted.parse::<LanguageIdentifier>() {
                tracing::info!("Successfully parsed extracted locale: {}", locale);
            } else {
                tracing::error!("Failed to parse extracted locale: {}", extracted);
                panic!("Should parse locale after extraction");
            }
        }
    }

    //
    // TODO(jb55): write tests that work, i broke all these during the refacto
    //

    /*
    use super::*;
    #[test]
    fn test_locale_management() {
        let i18n = Localization::default();

        // Test default locale
        let current = i18n.get_current_locale();
        assert_eq!(current.to_string(), "en-US");

        // Test available locales
        let available = i18n.get_available_locales();
        assert_eq!(available.len(), 2);
        assert_eq!(available[0].to_string(), "en-US");
        assert_eq!(available[1].to_string(), "en-XA");
    }

    #[test]
    fn test_cache_clearing() {
        let mut i18n = Localization::default();

        // Load and cache the FTL content
        let result1 = i18n.get_string(IntlKeyBuf::new("test_key").borrow());
        assert!(result1.is_ok());

        // Clear the cache
        let clear_result = i18n.clear_cache();
        assert!(clear_result.is_ok());

        // Should still work after clearing cache (will reload)
        let result2 = i18n.get_string(IntlKeyBuf::new("test_key").borrow());
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), "Test Value");
    }

    #[test]
    fn test_context_caching() {
        let mut i18n = Localization::default();

        // Debug: check what the normalized key should be
        let normalized_key = i18n.normalized_ftl_key("test_key", "comment");
        println!("Normalized key: '{}'", normalized_key);

        // First call should load and cache the FTL content
        let result1 = i18n.get_string(normalized_key.borrow());
        println!("First result: {:?}", result1);
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), "Test Value");

        // Second call should use cached FTL content
        let result2 = i18n.get_string(normalized_key.borrow());
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), "Test Value");

        // Test cache clearing through context
        let clear_result = i18n.clear_cache();
        assert!(clear_result.is_ok());

        // Should still work after clearing cache
        let result3 = i18n.get_string(normalized_key.borrow());
        assert!(result3.is_ok());
        assert_eq!(result3.unwrap(), "Test Value");
    }


    #[test]
    fn test_ftl_caching() {
        let mut i18n = Localization::default();

        // First call should load and cache the FTL content
        let result1 = i18n.get_string(IntlKeyBuf::new("test_key").borrow());
        assert!(result1.is_ok());
        assert_eq!(result1.as_ref().unwrap(), "Test Value");

        // Second call should use cached FTL content
        let result2 = i18n.get_string(IntlKeyBuf::new("test_key").borrow());
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), "Test Value");

        // Test another key from the same FTL content
        let result3 = i18n.get_string(IntlKeyBuf::new("another_key").borrow());
        assert!(result3.is_ok());
        assert_eq!(result3.unwrap(), "Another Value");
    }
    #[test]
    fn test_bundle_caching() {
        let mut i18n = Localization::default();

        // First call should create bundle and cache the resource
        let result1 = i18n.get_string(IntlKeyBuf::new("test_key").borrow());
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), "Test Value");

        // Second call should use cached resource but create new bundle
        let result2 = i18n.get_string(IntlKeyBuf::new("another_key").borrow());
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), "Another Value");

        // Check cache stats
        let stats = i18n.get_cache_stats().unwrap();
        assert_eq!(stats.resource_cache_size, 1);
        assert_eq!(stats.string_cache_size, 2); // Both strings should be cached
    }

    #[test]
    fn test_string_caching() {
        let mut i18n = Localization::default();
        let key = i18n.normalized_ftl_key("test_key", "comment");

        // First call should format and cache the string
        let result1 = i18n.get_string(key.borrow());
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), "Test Value");

        // Second call should use cached string
        let result2 = i18n.get_string(key.borrow());
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), "Test Value");

        // Check cache stats
        let stats = i18n.get_cache_stats().unwrap();
        assert_eq!(stats.string_cache_size, 1);
    }
    #[test]
    fn test_string_caching_with_arguments() {
        let mut manager = Localization::default();

        // First call with arguments should not be cached
        let mut args = fluent::FluentArgs::new();
        args.set("name", "Alice");
        let key = IntlKeyBuf::new("welcome_message");
        let result1 = manager
            .get_cached_string(key.borrow(), Some(&args))
            .unwrap();
        assert!(result1.contains("Alice"));

        // Check that it's not in the string cache
        let stats1 = manager.get_cache_stats().unwrap();
        assert_eq!(stats1.string_cache_size, 0);

        // Second call with different arguments should work correctly
        let mut args2 = fluent::FluentArgs::new();
        args2.set("name", "Bob");
        let result2 = manager.get_cached_string(key.borrow(), Some(&args2));
        assert!(result2.is_ok());
        let result2_str = result2.unwrap();
        assert!(result2_str.contains("Bob"));

        // Check that it's still not in the string cache
        let stats2 = manager.get_cache_stats().unwrap();
        assert_eq!(stats2.string_cache_size, 0);

        // Clear cache to start fresh
        manager.clear_cache().unwrap();

        let result3 = manager.get_string(key.borrow());
        assert!(result3.is_ok());
        assert_eq!(result3.unwrap(), "Hello World");

        // Check that simple string is cached
        let stats3 = manager.get_cache_stats().unwrap();
        assert_eq!(stats3.string_cache_size, 1);
    }

    #[test]
    fn test_cache_clearing_on_locale_change() {
        let mut i18n = Localization::default();

        // Check that caches are populated
        let stats1 = i18n.get_cache_stats().unwrap();
        assert!(stats1.resource_cache_size > 0);
        assert!(stats1.string_cache_size > 0);

        // Switch to en-XA
        let en_xa: LanguageIdentifier = langid!("en-XA");
        i18n.set_locale(en_xa).unwrap();

        // Check that string cache is cleared (resource cache remains for both locales)
        let stats2 = i18n.get_cache_stats().unwrap();
        assert_eq!(stats2.string_cache_size, 0);
    }
    */
}

/// Replace each invalid character with exactly one underscore
/// This matches the behavior of the Python extraction script
pub fn fixup_key(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' => out.push(ch),
            _ => out.push('_'), // always push
        }
    }
    let trimmed = out.trim_matches('_');
    trimmed.to_owned()
}

fn simple_hash(s: &str) -> String {
    let digest = md5::compute(s.as_bytes());
    // Take the first 2 bytes and convert to 4 hex characters
    format!("{:02x}{:02x}", digest[0], digest[1])
}
