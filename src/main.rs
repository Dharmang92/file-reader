#![windows_subsystem = "windows"]

use eframe::{
    App, CreationContext,
    egui::{self, FontSelection, Frame, RichText, Sense, UiBuilder},
};
use std::{
    fs,
    path::{Path, PathBuf, absolute},
};
use walkdir::WalkDir;

mod code_editor;
use code_editor::CodeEditor;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "File Reader",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

struct MyApp {
    path: String,
    files: Vec<PathBuf>,
    search: String,
    filtered_files: Vec<PathBuf>,
    selected_file: Option<PathBuf>,
    file_content: String,
    edited_content: String,
    editor: CodeEditor,
    error: String,
}

impl MyApp {
    fn new(cc: &CreationContext) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::monospace(16.0));
        cc.egui_ctx.set_style(style);

        let mut app = Self {
            path: get_local_path(),
            files: Vec::new(),
            search: String::new(),
            filtered_files: Vec::new(),
            selected_file: None,
            file_content: String::new(),
            edited_content: String::new(),
            editor: CodeEditor::default(),
            error: String::new(),
        };

        app.setup();
        app
    }

    fn setup(&mut self) {
        if self.path.is_empty() || !Path::new(&self.path).exists() {
            self.error = "(Invalid Path) Path does not exist!".to_string();
            self.path = get_local_path();
        } else {
            self.error.clear();
            self.files = WalkDir::new(&self.path)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_path_buf())
                .collect();

            self.update_filtered_files();

            self.select_first_file();
        }
    }

    fn update_filtered_files(&mut self) {
        if self.search.is_empty() {
            self.filtered_files = self.files.clone();
        } else {
            let search_lower = self.search.to_lowercase();
            self.filtered_files = self
                .files
                .iter()
                .filter(|path| {
                    path.to_string_lossy()
                        .to_lowercase()
                        .contains(&search_lower)
                })
                .cloned()
                .collect();
        }
    }

    fn select_first_file(&mut self) {
        if let Some(first_file) = self.filtered_files.first() {
            self.selected_file = Some(first_file.clone());
            self.file_content =
                fs::read_to_string(first_file).unwrap_or_else(|_| "<failed to read file>".into());
            self.edited_content = self.file_content.clone();
            self.editor.set_content(self.edited_content.clone());
        }
    }

    fn show_error_popup(&mut self, ctx: &egui::Context) {
        egui::Window::new("Error")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(self.error.clone());
                    if ui.button("Ok").clicked() {
                        self.error.clear();
                    }
                });
            });
    }
}

fn get_local_path() -> String {
    absolute(
        dirs::data_local_dir()
            .unwrap()
            .join("Microsoft/UserSecrets"),
    )
    .expect("Failed to fetch local directory!")
    .to_string_lossy()
    .to_string()
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("path_input_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Show Files").clicked() {
                    self.setup();
                }

                egui::TextEdit::singleline(&mut self.path)
                    .font(FontSelection::FontId(egui::FontId {
                        size: 14.0,
                        family: egui::FontFamily::default(),
                    }))
                    .hint_text(format!("Eg. {}", get_local_path()))
                    .desired_width(ui.available_width())
                    .show(ui);
            });
        });

        egui::SidePanel::left("file_list_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Files");

                let search_response = egui::TextEdit::singleline(&mut self.search)
                    .font(FontSelection::FontId(egui::FontId {
                        size: 14.0,
                        family: egui::FontFamily::default(),
                    }))
                    .hint_text("Search...")
                    .desired_width(ui.available_width())
                    .show(ui)
                    .response;

                if search_response.changed() {
                    self.update_filtered_files();
                    self.select_first_file();
                }
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                for file in &self.filtered_files {
                    let folder = file
                        .parent()
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_string_lossy();
                    let fname = file.file_name().unwrap().to_string_lossy();

                    let response = ui
                        .scope_builder(
                            UiBuilder::new()
                                .id_salt("interactive_container")
                                .sense(Sense::hover()),
                            |ui| {
                                let response = ui.response();
                                let visuals = ui.style().interact(&response);
                                let text_color = visuals.text_color();

                                Frame::canvas(ui.style())
                                    .fill(visuals.bg_fill.gamma_multiply(0.3))
                                    .stroke(visuals.bg_stroke)
                                    .show(ui, |ui| {
                                        ui.output_mut(|o| {
                                            o.cursor_icon = egui::CursorIcon::PointingHand
                                        });

                                        ui.set_width(ui.available_width());

                                        ui.vertical_centered(|ui| {
                                            ui.add_space(10.0);
                                            ui.label(
                                                RichText::new(format!("📂 {} / {}", folder, fname))
                                                    .color(text_color)
                                                    .size(15.0),
                                            );
                                            ui.add_space(10.0);
                                        });
                                    });
                            },
                        )
                        .response;

                    if response.hovered() {
                        self.selected_file = Some(file.clone());
                        self.file_content = fs::read_to_string(file)
                            .unwrap_or_else(|_| "<failed to read file>".into());
                        self.edited_content = self.file_content.clone();
                        self.editor.set_content(self.edited_content.clone());
                    }
                }

                if self.filtered_files.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label("No matches found!");
                        ui.add_space(20.0);
                    });
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    if let Some(ref path) = self.selected_file {
                        if let Err(err) = fs::write(path, &self.edited_content) {
                            self.error = format!("Failed to save file: {}", err);
                        }
                    }
                }

                if let Some(ref path) = self.selected_file {
                    ui.monospace(path.file_name().unwrap_or_default().to_string_lossy());
                }
            });

            self.editor.ui(ui);
            self.edited_content = self.editor.get_content();
        });

        if !self.error.is_empty() {
            self.show_error_popup(ctx);
        }
    }
}
