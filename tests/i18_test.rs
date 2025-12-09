// tests/i18_test.rs

rust_i18n::i18n!("./locales");

mod tests {
    use rust_i18n::t;

    #[test]
    fn test_translations_starting_locale_en() {
        assert_eq!(
            t!("main.starting", locale = "en"),
            "Starting Arti Onion Proxy"
        );
    }
    #[test]
    fn test_translations_invalid_nickname_locale_en() {
        let msg = t!("tor.errors.invalid_nickname", locale = "en");
        assert_eq!(msg, "Invalid nickname");
    }

    #[test]
    fn test_translations_starting_locale_ru() {
        assert_eq!(
            t!("main.starting", locale = "ru"),
            "Запуск Arti Onion Proxy"
        );
    }

    #[test]
    fn test_translations_invalid_nickname_locale_ru() {
        let msg = t!("tor.errors.invalid_nickname", locale = "ru");
        assert_eq!(msg, "Некорректный никнейм");
    }

    #[test]
    fn test_translations_locale_en_and_param() {
        // Checking translation with parameter substitution
        // For example, a message that includes an IP address
        let msg = t!("server_status", addr = "127.0.0.1", locale = "en");
        assert_eq!(msg, "Server is running on 127.0.0.1");
    }

    #[test]
    fn test_missing_translation_fallback() {
        // In English locale
        // If the key is missing, it should return the key itself
        let output = t!("missing_key", locale = "ru");
        assert_eq!(output, "missing_key");
    }
}
