// src/utils.rs
use tracing::info;

/// Sets up the locale for internationalization based on environment variables and system settings
/// with fuzzy matching to select the best available locale.
/// Falls back to English if no suitable locale is found.
pub fn setup_locale() {
    let requested_locale = std::env::var("ARTI_LANG")
        .ok()
        .filter(|l| !l.is_empty())
        .or_else(sys_locale::get_locale)
        .unwrap_or_else(|| "en".to_string());

    // Get available locales from rust_i18n
    let available = rust_i18n::available_locales!();

    // Determine the final locale to use with fuzzy matching
    let final_locale = if available.contains(&requested_locale.as_str()) {
        // Exact match found
        requested_locale
    } else {
        // Trying fuzzy match by language code only (e.g., "ru-RU" -> "ru")
        let lang_code = requested_locale.split(['-', '_']).next().unwrap_or("");

        if available.contains(&lang_code) {
            lang_code.to_string()
        } else {
            // Fallback to English if no match found
            "en".to_string()
        }
    };

    rust_i18n::set_locale(&final_locale);

    info!("DEBUG: Current locale: {}", &final_locale);
}
