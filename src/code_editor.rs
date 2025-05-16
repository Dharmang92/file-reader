use std::f32;

use eframe::egui::{self, FontSelection, Ui};

pub struct CodeEditor {
    pub language: String,
    pub code: String,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            language: "rs".into(),
            code: "// Select a file to view its contents\n".into(),
        }
    }
}

impl CodeEditor {
    pub fn set_content(&mut self, content: String) {
        self.code = content;
    }

    pub fn get_content(&self) -> String {
        self.code.clone()
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let Self { language, code } = self;

        let mut theme =
            egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
        ui.collapsing("Theme", |ui| {
            ui.group(|ui| {
                theme.ui(ui);
                theme.clone().store_in_memory(ui.ctx());
            });
        });

        let mut layouter = |ui: &Ui, buf: &str, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                ui.style(),
                &theme,
                buf,
                language,
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        egui::ScrollArea::vertical()
            .max_height(f32::INFINITY)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(code)
                        .font(FontSelection::FontId(egui::FontId {
                            size: 20.0,
                            family: egui::FontFamily::default(),
                        }))
                        .code_editor()
                        .desired_rows(10)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });
    }
}
