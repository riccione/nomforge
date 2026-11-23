use std::borrow::Cow;
use std::path::Path;

use eframe::egui;

use crate::state::State;

/// Width reserved for the status column.
const STATUS_WIDTH: f32 = 88.0;
/// Minimum width for each filename column.
const MIN_NAME_WIDTH: f32 = 80.0;

/// File name for display, borrowing when the path is valid UTF-8.
fn display_name(path: &Path) -> Cow<'_, str> {
    path.file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_default()
}

/// Column layout for one table pass.
struct Columns {
    name: f32,
    status: f32,
    row_height: f32,
}

impl Columns {
    fn new(ui: &egui::Ui) -> Self {
        let spacing = ui.spacing().item_spacing.x;
        let status = STATUS_WIDTH;
        let name = ((ui.available_width() - status - 2.0 * spacing) / 2.0).max(MIN_NAME_WIDTH);
        Self {
            name,
            status,
            row_height: ui.spacing().interact_size.y,
        }
    }
}

/// A filename cell that truncates with an ellipsis and shows the full name on hover.
fn name_cell(ui: &mut egui::Ui, columns: &Columns, text: &str) -> egui::Response {
    ui.add_sized(
        [columns.name, columns.row_height],
        egui::Label::new(text).truncate(),
    )
    .on_hover_text(text)
}

/// A fixed-size cell that is not truncated.
fn text_cell(
    ui: &mut egui::Ui,
    width: f32,
    columns: &Columns,
    text: impl Into<egui::WidgetText>,
) -> egui::Response {
    ui.add_sized([width, columns.row_height], egui::Label::new(text))
}

/// Render the preview table panel showing plans and/or results.
pub fn show(ui: &mut egui::Ui, state: &State) {
    // Show conflicts if any
    if !state.pending_conflicts.is_empty() {
        ui.colored_label(
            egui::Color32::from_rgb(255, 165, 0),
            format!(
                "{} conflict(s) detected — files may be overwritten",
                state.pending_conflicts.len()
            ),
        );
        for conflict in &state.pending_conflicts {
            ui.label(
                egui::RichText::new(format!("  • {}", conflict.reason))
                    .color(egui::Color32::from_rgb(255, 165, 0)),
            );
        }
        ui.separator();
    }

    // Show plans (preview mode)
    if !state.plans.is_empty() {
        ui.heading("Preview");
        egui::ScrollArea::vertical()
            .id_salt("preview_scroll")
            .auto_shrink([false, true])
            .show(ui, |ui| {
                let columns = Columns::new(ui);

                egui::Grid::new("preview_grid")
                    .striped(true)
                    .num_columns(3)
                    .show(ui, |ui| {
                        text_cell(
                            ui,
                            columns.name,
                            &columns,
                            egui::RichText::new("FILE").strong(),
                        );
                        text_cell(
                            ui,
                            columns.name,
                            &columns,
                            egui::RichText::new("NEW NAME").strong(),
                        );
                        text_cell(ui, columns.status, &columns, "");
                        ui.end_row();

                        for plan in &state.plans {
                            let source = display_name(&plan.source);
                            let target = display_name(&plan.target);

                            if plan.source == plan.target {
                                name_cell(ui, &columns, &source);
                                text_cell(
                                    ui,
                                    columns.name,
                                    &columns,
                                    egui::RichText::new("(no change)").weak(),
                                );
                                text_cell(ui, columns.status, &columns, "");
                            } else {
                                name_cell(ui, &columns, &source);
                                name_cell(ui, &columns, &target);
                                text_cell(
                                    ui,
                                    columns.status,
                                    &columns,
                                    egui::RichText::new("->").weak(),
                                );
                            }
                            ui.end_row();
                        }
                    });
            });
    }

    // Show results (after apply)
    if !state.results.is_empty() {
        ui.heading("Results");
        egui::ScrollArea::vertical()
            .id_salt("results_scroll")
            .auto_shrink([false, true])
            .show(ui, |ui| {
                let columns = Columns::new(ui);

                egui::Grid::new("results_grid")
                    .striped(true)
                    .num_columns(3)
                    .show(ui, |ui| {
                        text_cell(
                            ui,
                            columns.name,
                            &columns,
                            egui::RichText::new("FILE").strong(),
                        );
                        text_cell(
                            ui,
                            columns.name,
                            &columns,
                            egui::RichText::new("NEW NAME").strong(),
                        );
                        text_cell(
                            ui,
                            columns.status,
                            &columns,
                            egui::RichText::new("STATUS").strong(),
                        );
                        ui.end_row();

                        for result in &state.results {
                            let source = display_name(&result.source);
                            let target = display_name(&result.target);

                            if result.source == result.target {
                                name_cell(ui, &columns, &source);
                                text_cell(
                                    ui,
                                    columns.name,
                                    &columns,
                                    egui::RichText::new("(no change)").weak(),
                                );
                                text_cell(
                                    ui,
                                    columns.status,
                                    &columns,
                                    egui::RichText::new("unchanged").weak(),
                                );
                            } else if result.success {
                                name_cell(ui, &columns, &source);
                                name_cell(ui, &columns, &target);
                                text_cell(
                                    ui,
                                    columns.status,
                                    &columns,
                                    egui::RichText::new("ok")
                                        .strong()
                                        .color(egui::Color32::GREEN),
                                );
                            } else {
                                name_cell(ui, &columns, &source);
                                name_cell(ui, &columns, &target);
                                let status = text_cell(
                                    ui,
                                    columns.status,
                                    &columns,
                                    egui::RichText::new("FAILED")
                                        .strong()
                                        .color(egui::Color32::RED),
                                );
                                if let Some(err) = &result.error {
                                    status.on_hover_text(err);
                                }
                            }
                            ui.end_row();
                        }
                    });

                // Failure details would not fit in the status column, so list
                // them at full width below the table.
                let failures: Vec<(String, String)> = state
                    .results
                    .iter()
                    .filter_map(|result| {
                        result
                            .error
                            .as_ref()
                            .map(|err| (display_name(&result.source).into_owned(), err.clone()))
                    })
                    .collect();
                for (source, err) in failures {
                    let text = format!("⚠ {source}: {err}");
                    ui.add(egui::Label::new(
                        egui::RichText::new(text.clone()).color(egui::Color32::RED),
                    ))
                    .on_hover_text(text);
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn display_name_borrows_valid_utf8() {
        let path = PathBuf::from("/tmp/photos/photo.jpg");
        let name = display_name(&path);
        assert_eq!(name, "photo.jpg");
        assert!(matches!(name, Cow::Borrowed(_)));
    }

    #[test]
    fn display_name_without_file_name_is_empty() {
        assert_eq!(display_name(Path::new("/")), "");
    }

    #[cfg(unix)]
    #[test]
    fn display_name_lossy_for_non_utf8() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let path = PathBuf::from(OsStr::from_bytes(b"bad\xFF.txt"));
        let name = display_name(&path);
        assert!(name.contains('\u{FFFD}'), "name: {name:?}");
    }

    #[test]
    fn preview_and_results_tables_do_not_clash() {
        let ctx = egui::Context::default();
        let state = State {
            plans: vec![nomforge_core::RenamePlan {
                source: PathBuf::from("/tmp/a.txt"),
                target: PathBuf::from("/tmp/b.txt"),
            }],
            results: vec![nomforge_core::RenameResult {
                source: PathBuf::from("/tmp/a.txt"),
                target: PathBuf::from("/tmp/b.txt"),
                success: true,
                error: None,
            }],
            ..Default::default()
        };

        // Both tables render in the same pass after an apply; egui paints
        // ID-clash warnings as debug text into the frame output.
        let output = ctx.run_ui(egui::RawInput::default(), |ui| show(ui, &state));
        let clash = output.shapes.iter().any(|clipped| {
            matches!(
                &clipped.shape,
                egui::epaint::Shape::Text(text) if text.galley.job.text.contains("First use of")
            )
        });
        output.drop_without_applying_deltas();

        assert!(!clash, "preview and results tables share widget IDs");
    }

    #[test]
    fn tables_fill_available_width() {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(600.0, 400.0),
            )),
            ..Default::default()
        };

        // Long names must truncate instead of widening (and clipping) the table.
        let long = "x".repeat(500);
        let source = PathBuf::from(format!("/tmp/{long}.txt"));
        let target = PathBuf::from(format!("/tmp/{long}_renamed.txt"));

        let plans = State {
            plans: vec![nomforge_core::RenamePlan {
                source: source.clone(),
                target: target.clone(),
            }],
            ..Default::default()
        };
        let results = State {
            results: vec![nomforge_core::RenameResult {
                source,
                target,
                success: true,
                error: None,
            }],
            ..Default::default()
        };

        for state in [plans, results] {
            let mut width = 0.0;
            ctx.run_ui(input.clone(), |ui| {
                show(ui, &state);
                width = ui.min_rect().width();
            })
            .drop_without_applying_deltas();

            assert!(width >= 580.0, "table width {width} does not fill 600px");
        }
    }
}
