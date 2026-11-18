//! Persistent GUI settings (theme and font size).

use std::fs;
use std::path::{Path, PathBuf};

use eframe::egui;
use serde::{Deserialize, Serialize};

/// Default font size in points. egui's body text is 13pt; the GUI bumps that
/// by 2pt at startup, so 15pt corresponds to zoom factor 1.0 (the default
/// look).
pub const DEFAULT_FONT_SIZE: f32 = 15.0;

/// Smallest selectable font size.
pub const MIN_FONT_SIZE: f32 = 10.0;

/// Largest selectable font size.
pub const MAX_FONT_SIZE: f32 = 24.0;

/// Color theme selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeSetting {
    Light,
    Dark,
    #[default]
    System,
}

impl From<ThemeSetting> for egui::ThemePreference {
    fn from(theme: ThemeSetting) -> Self {
        match theme {
            ThemeSetting::Light => Self::Light,
            ThemeSetting::Dark => Self::Dark,
            ThemeSetting::System => Self::System,
        }
    }
}

/// Settings persisted between runs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub theme: ThemeSetting,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
}

fn default_font_size() -> f32 {
    DEFAULT_FONT_SIZE
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemeSetting::default(),
            font_size: DEFAULT_FONT_SIZE,
        }
    }
}

impl Settings {
    /// egui zoom factor for the configured font size.
    pub fn zoom_factor(&self) -> f32 {
        zoom_for_font_size(self.font_size)
    }
}

/// Convert a font size in points to an egui zoom factor.
///
/// The whole UI is scaled (not just the glyphs), so 15pt keeps the default
/// look and e.g. 22.5pt renders everything 1.5x larger.
pub fn zoom_for_font_size(font_size: f32) -> f32 {
    font_size / DEFAULT_FONT_SIZE
}

/// Default settings location: `$XDG_CONFIG_HOME/nomforge/settings.json`,
/// falling back to `$HOME/.config/nomforge/settings.json`.
pub fn settings_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .filter(|p| !p.is_empty())
                .map(|p| PathBuf::from(p).join(".config"))
        })
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("nomforge").join("settings.json")
}

/// Load settings from the default location.
pub fn load() -> Settings {
    load_from(&settings_path())
}

/// Load settings from `path`.
///
/// A missing or unreadable file (including invalid JSON) falls back to the
/// defaults so a broken settings file never blocks startup.
pub fn load_from(path: &Path) -> Settings {
    fs::read_to_string(path)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default()
}

/// Save settings to the default location.
pub fn save(settings: &Settings) -> std::io::Result<()> {
    save_to(&settings_path(), settings)
}

/// Save settings to `path`, creating parent directories as needed.
pub fn save_to(path: &Path, settings: &Settings) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(settings).map_err(std::io::Error::other)?;
    fs::write(path, data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn defaults_are_system_and_default_size() {
        let settings = Settings::default();
        assert_eq!(settings.theme, ThemeSetting::System);
        assert_eq!(settings.font_size, DEFAULT_FONT_SIZE);
    }

    #[test]
    fn serde_roundtrip() {
        let settings = Settings {
            theme: ThemeSetting::Dark,
            font_size: 18.0,
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"dark\""), "json: {json}");
        let loaded: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, settings);
    }

    #[test]
    fn missing_fields_use_defaults() {
        let loaded: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(loaded, Settings::default());
    }

    #[test]
    fn missing_file_returns_defaults() {
        let tmp = TempDir::new().unwrap();
        let loaded = load_from(&tmp.path().join("absent.json"));
        assert_eq!(loaded, Settings::default());
    }

    #[test]
    fn corrupt_file_returns_defaults() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");
        fs::write(&path, "not json").unwrap();
        assert_eq!(load_from(&path), Settings::default());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nested").join("settings.json");
        let settings = Settings {
            theme: ThemeSetting::Light,
            font_size: 20.0,
        };
        save_to(&path, &settings).unwrap();
        assert_eq!(load_from(&path), settings);
    }

    #[test]
    fn settings_path_points_to_nomforge() {
        let path = settings_path();
        let path = path.to_string_lossy();
        assert!(path.contains("nomforge"), "path: {path}");
        assert!(path.ends_with("settings.json"), "path: {path}");
    }

    #[test]
    fn zoom_maps_font_size() {
        assert_eq!(zoom_for_font_size(DEFAULT_FONT_SIZE), 1.0);
        assert_eq!(zoom_for_font_size(30.0), 2.0);
    }

    #[test]
    fn theme_converts_to_egui_preference() {
        assert_eq!(
            egui::ThemePreference::from(ThemeSetting::Light),
            egui::ThemePreference::Light
        );
        assert_eq!(
            egui::ThemePreference::from(ThemeSetting::Dark),
            egui::ThemePreference::Dark
        );
        assert_eq!(
            egui::ThemePreference::from(ThemeSetting::System),
            egui::ThemePreference::System
        );
    }
}
