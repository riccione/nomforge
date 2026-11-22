use eframe::egui;

/// A colored badge indicating conflict status.
pub enum BadgeKind {
    /// No conflict (green).
    Ok,
    /// Error/conflict (red).
    Error,
}

impl BadgeKind {
    fn color(&self) -> egui::Color32 {
        match self {
            Self::Ok => egui::Color32::from_rgb(34, 139, 34), // forest green
            Self::Error => egui::Color32::from_rgb(220, 20, 60), // crimson
        }
    }
}

/// Show a conflict badge with the given text and kind.
fn show(ui: &mut egui::Ui, text: &str, kind: BadgeKind) {
    let color = kind.color();
    ui.label(
        egui::RichText::new(format!(" {text} "))
            .color(egui::Color32::WHITE)
            .background_color(color),
    );
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
    fn badge_kind_colors_differ() {
        assert_ne!(BadgeKind::Ok.color(), BadgeKind::Error.color());
    }
}
