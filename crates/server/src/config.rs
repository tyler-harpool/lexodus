use shared_types::{AppConfig, FeatureFlags};
use std::sync::OnceLock;

static FLAGS: OnceLock<FeatureFlags> = OnceLock::new();

/// Path to the config file, relative to the project root.
const CONFIG_PATH: &str = "config.toml";

/// Read `config.toml`, parse feature flags, and store them in the global
/// `OnceLock`. Safe to call multiple times — only the first call has effect.
///
/// If the file is missing or unparseable, all flags default to `false`.
pub fn load_feature_flags() {
    FLAGS.get_or_init(|| match std::fs::read_to_string(CONFIG_PATH) {
        Ok(contents) => {
            let config: AppConfig = toml::from_str(&contents).unwrap_or_else(|e| {
                eprintln!("[config] Failed to parse {CONFIG_PATH}: {e} — defaulting all flags off");
                AppConfig::default()
            });
            eprintln!("[config] Feature flags: {:?}", config.features);
            config.features
        }
        Err(e) => {
            eprintln!("[config] {CONFIG_PATH} not found ({e}) — defaulting all flags off");
            FeatureFlags::default()
        }
    });
}

/// Get the loaded feature flags. Returns all-false defaults if
/// `load_feature_flags()` hasn't been called yet (safe fallback).
pub fn feature_flags() -> &'static FeatureFlags {
    static DEFAULT: FeatureFlags = FeatureFlags {
        oauth: false,
        stripe: false,
        mailgun: false,
        twilio: false,
        s3: false,
        telemetry: false,
    };
    FLAGS.get().unwrap_or(&DEFAULT)
}
