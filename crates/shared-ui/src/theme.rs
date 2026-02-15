use dioxus::prelude::*;

/// Theme families available in the application.
///
/// Each family provides a dark variant, a light variant, or both.
/// Families with only one mode resolve to that mode regardless of `is_dark`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ThemeFamily {
    #[default]
    Cyberpunk,
    Solar,
    Federal,
    /// Dark-only warm judicial theme.
    Chambers,
    /// Light-only document-reading theme.
    Parchment,
}

/// All available theme families in display order.
pub const ALL_FAMILIES: &[ThemeFamily] = &[
    ThemeFamily::Cyberpunk,
    ThemeFamily::Solar,
    ThemeFamily::Federal,
    ThemeFamily::Chambers,
    ThemeFamily::Parchment,
];

impl ThemeFamily {
    /// Internal key used for storage and Select values.
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeFamily::Cyberpunk => "cyberpunk",
            ThemeFamily::Solar => "solar",
            ThemeFamily::Federal => "federal",
            ThemeFamily::Chambers => "chambers",
            ThemeFamily::Parchment => "parchment",
        }
    }

    /// Human-readable name for display in UI.
    pub fn display_name(&self) -> &'static str {
        match self {
            ThemeFamily::Cyberpunk => "Cyberpunk",
            ThemeFamily::Solar => "Solarized",
            ThemeFamily::Federal => "Federal",
            ThemeFamily::Chambers => "Chambers",
            ThemeFamily::Parchment => "Parchment",
        }
    }

    /// Parse a family key string, falling back to Cyberpunk.
    pub fn from_key(s: &str) -> Self {
        match s {
            "solar" => ThemeFamily::Solar,
            "federal" => ThemeFamily::Federal,
            "chambers" => ThemeFamily::Chambers,
            "parchment" => ThemeFamily::Parchment,
            _ => ThemeFamily::Cyberpunk,
        }
    }

    /// Whether this family supports dark mode.
    pub fn has_dark(&self) -> bool {
        !matches!(self, ThemeFamily::Parchment)
    }

    /// Whether this family supports light mode.
    pub fn has_light(&self) -> bool {
        !matches!(self, ThemeFamily::Chambers)
    }

    /// Resolve to the CSS `data-theme` attribute value.
    ///
    /// Single-mode families ignore `is_dark` and always return their mode.
    pub fn resolve(&self, is_dark: bool) -> &'static str {
        match (self, is_dark) {
            (ThemeFamily::Cyberpunk, true) => "cyberpunk",
            (ThemeFamily::Cyberpunk, false) => "light",
            (ThemeFamily::Solar, true) => "solar",
            (ThemeFamily::Solar, false) => "solar-light",
            (ThemeFamily::Federal, true) => "federal",
            (ThemeFamily::Federal, false) => "federal-light",
            // Chambers is dark-only
            (ThemeFamily::Chambers, _) => "chambers",
            // Parchment is light-only
            (ThemeFamily::Parchment, _) => "parchment",
        }
    }
}

/// Shared theme state provided as context.
///
/// Both the sidebar (dark/light toggle) and settings (family picker)
/// read and write these signals. Changes call [`set_theme`] to apply.
#[derive(Clone, Copy)]
pub struct ThemeState {
    pub family: Signal<String>,
    pub is_dark: Signal<bool>,
}

impl ThemeState {
    /// Apply the current family + mode to the document.
    pub fn apply(&self) {
        let family = ThemeFamily::from_key(&self.family.read());
        let theme = family.resolve(*self.is_dark.read());
        set_theme(theme);
    }
}

/// Seed the theme on application startup.
///
/// Reads the persisted theme from a cookie and applies it to the document root.
/// Call this once in your top-level App component.
#[component]
pub fn ThemeSeed() -> Element {
    use_effect(|| {
        // Read theme cookie and apply data-theme attribute to <html>
        document::eval(
            r#"
            (function() {
                var match = document.cookie.match(/(?:^|;\s*)theme=([^;]*)/);
                var theme = match ? match[1] : 'cyberpunk';
                document.documentElement.setAttribute('data-theme', theme);
            })();
            "#,
        );
    });

    rsx! {}
}

/// Set the active theme, persisting to a cookie and updating the document.
///
/// Uses BroadcastChannel to sync across tabs when available.
pub fn set_theme(theme: &str) {
    document::eval(&format!(
        r#"
        (function() {{
            document.cookie = 'theme={theme};path=/;max-age=2592000;SameSite=Lax';
            document.documentElement.setAttribute('data-theme', '{theme}');
            try {{
                var bc = new BroadcastChannel('theme-sync');
                bc.postMessage('{theme}');
                bc.close();
            }} catch(e) {{}}
        }})();
        "#,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_family_default_is_cyberpunk() {
        assert_eq!(ThemeFamily::default(), ThemeFamily::Cyberpunk);
    }

    #[test]
    fn theme_family_as_str_roundtrip() {
        for family in ALL_FAMILIES {
            assert_eq!(ThemeFamily::from_key(family.as_str()), *family);
        }
    }

    #[test]
    fn theme_family_from_key_unknown_falls_back() {
        assert_eq!(ThemeFamily::from_key("unknown"), ThemeFamily::Cyberpunk);
        assert_eq!(ThemeFamily::from_key(""), ThemeFamily::Cyberpunk);
    }

    #[test]
    fn theme_family_resolve_dual_mode() {
        assert_eq!(ThemeFamily::Cyberpunk.resolve(true), "cyberpunk");
        assert_eq!(ThemeFamily::Cyberpunk.resolve(false), "light");
        assert_eq!(ThemeFamily::Solar.resolve(true), "solar");
        assert_eq!(ThemeFamily::Solar.resolve(false), "solar-light");
        assert_eq!(ThemeFamily::Federal.resolve(true), "federal");
        assert_eq!(ThemeFamily::Federal.resolve(false), "federal-light");
    }

    #[test]
    fn theme_family_resolve_single_mode() {
        // Chambers is dark-only — always resolves to "chambers"
        assert_eq!(ThemeFamily::Chambers.resolve(true), "chambers");
        assert_eq!(ThemeFamily::Chambers.resolve(false), "chambers");
        // Parchment is light-only — always resolves to "parchment"
        assert_eq!(ThemeFamily::Parchment.resolve(true), "parchment");
        assert_eq!(ThemeFamily::Parchment.resolve(false), "parchment");
    }

    #[test]
    fn theme_family_mode_support() {
        assert!(ThemeFamily::Cyberpunk.has_dark());
        assert!(ThemeFamily::Cyberpunk.has_light());
        assert!(ThemeFamily::Chambers.has_dark());
        assert!(!ThemeFamily::Chambers.has_light());
        assert!(!ThemeFamily::Parchment.has_dark());
        assert!(ThemeFamily::Parchment.has_light());
    }

    #[test]
    fn all_families_list_is_complete() {
        assert_eq!(ALL_FAMILIES.len(), 5);
    }
}
