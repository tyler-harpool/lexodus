use serde::{Deserialize, Serialize};

/// Feature flags controlling which optional integrations are active.
///
/// Loaded from `config.toml` at server startup and exposed to clients
/// via a server function. Every field defaults to `false` so that a
/// missing or incomplete config file disables all optional features.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct FeatureFlags {
    #[serde(default)]
    pub oauth: bool,
    #[serde(default)]
    pub stripe: bool,
    #[serde(default)]
    pub mailgun: bool,
    #[serde(default)]
    pub twilio: bool,
    #[serde(default)]
    pub s3: bool,
    #[serde(default)]
    pub telemetry: bool,
}

/// Top-level config file structure matching `config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub features: FeatureFlags,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_flags_all_false() {
        let flags = FeatureFlags::default();
        assert!(!flags.oauth);
        assert!(!flags.stripe);
        assert!(!flags.mailgun);
        assert!(!flags.twilio);
        assert!(!flags.s3);
        assert!(!flags.telemetry);
    }

    #[test]
    fn deserialize_empty_toml_defaults_all_false() {
        let config: AppConfig = toml::from_str("").unwrap();
        assert_eq!(config.features, FeatureFlags::default());
    }

    #[test]
    fn deserialize_partial_toml_defaults_missing_fields() {
        let config: AppConfig = toml::from_str(
            r#"
            [features]
            stripe = true
            "#,
        )
        .unwrap();
        assert!(config.features.stripe);
        assert!(!config.features.oauth);
        assert!(!config.features.mailgun);
        assert!(!config.features.twilio);
        assert!(!config.features.s3);
        assert!(!config.features.telemetry);
    }

    #[test]
    fn deserialize_full_toml() {
        let config: AppConfig = toml::from_str(
            r#"
            [features]
            oauth = true
            stripe = true
            mailgun = true
            twilio = true
            s3 = true
            telemetry = true
            "#,
        )
        .unwrap();
        assert!(config.features.oauth);
        assert!(config.features.stripe);
        assert!(config.features.mailgun);
        assert!(config.features.twilio);
        assert!(config.features.s3);
        assert!(config.features.telemetry);
    }

    #[test]
    fn serialize_roundtrip() {
        let flags = FeatureFlags {
            oauth: true,
            stripe: false,
            mailgun: true,
            twilio: false,
            s3: true,
            telemetry: false,
        };
        let json = serde_json::to_string(&flags).unwrap();
        let deserialized: FeatureFlags = serde_json::from_str(&json).unwrap();
        assert_eq!(flags, deserialized);
    }

    #[test]
    fn deserialize_json_with_missing_fields_defaults() {
        let flags: FeatureFlags = serde_json::from_str("{}").unwrap();
        assert_eq!(flags, FeatureFlags::default());
    }
}
