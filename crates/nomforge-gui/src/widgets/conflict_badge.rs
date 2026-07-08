use eframe::egui;

/// A colored badge indicating conflict status.
#[allow(dead_code)]
pub enum BadgeKind {
    /// No conflict (green).
    Ok,
    /// Warning (yellow/amber) - reserved for future use.
    Warning,
    /// Error/conflict (red).
    Error,
    /// Informational (blue) - reserved for future use.
    Info,
}

impl BadgeKind {
    fn color(&self) -> egui::Color32 {
        match self {
            Self::Ok => egui::Color32::from_rgb(34, 139, 34), // forest green
            Self::Warning => egui::Color32::from_rgb(218, 165, 32), // goldenrod
            Self::Error => egui::Color32::from_rgb(220, 20, 60), // crimson
            Self::Info => egui::Color32::from_rgb(70, 130, 180), // steel blue
        }
    }
}

/// Show a conflict badge with the given text and kind.
pub fn show(ui: &mut egui::Ui, text: &str, kind: BadgeKind) {
    let color = kind.color();
    ui.label(
        egui::RichText::new(format!(" {text} "))
            .color(egui::Color32::WHITE)
            .background_color(color),
    );
}

/// Show a conflict badge for a rename result.
#[expect(dead_code, reason = "reserved for future result status display")]
pub fn show_result_status(ui: &mut egui::Ui, success: bool) {
    if success {
        show(ui, "ok", BadgeKind::Ok);
    } else {
        show(ui, "FAILED", BadgeKind::Error);
    }
}

/// Show a conflict badge for a conflict count.
pub fn show_conflict_count(ui: &mut egui::Ui, count: usize) {
    if count == 0 {
        show(ui, "no conflicts", BadgeKind::Ok);
    } else {
        let text = format!("{count} conflict{}", if count == 1 { "" } else { "s" });
        show(ui, &text, BadgeKind::Error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_kind_colors() {
        // Ensure all variants produce distinct colors
        let ok = BadgeKind::Ok.color();
        let warn = BadgeKind::Warning.color();
        let err = BadgeKind::Error.color();
        let info = BadgeKind::Info.color();
        assert_ne!(ok, warn);
        assert_ne!(ok, err);
        assert_ne!(ok, info);
        assert_ne!(warn, err);
        assert_ne!(warn, info);
        assert_ne!(err, info);
    }
}
