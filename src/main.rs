#![windows_subsystem = "windows"]

use eframe::{
    App, CreationContext,
    egui::{self, FontSelection},
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

mod code_editor;
use code_editor::CodeEditor;

mod explorer;
use explorer::Explorer;

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
    explorer: Explorer,
    files: Vec<PathBuf>,
    search: String,
    search_suggestions: Vec<String>,
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
            path: get_roaming_path().to_string_lossy().to_string(),
            explorer: Explorer::from_pathbuf(&get_roaming_path().into()),
            files: Vec::new(),
            search: String::new(),
            search_suggestions: Vec::new(),
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
            self.path = get_roaming_path().to_string_lossy().to_string();
        } else {
            self.error.clear();
            self.files = WalkDir::new(&self.path)
                .max_depth(1)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_path_buf())
                .collect();

            self.update_filtered_files();

            self.select_first_file();

            self.explorer = Explorer::from_pathbuf(&PathBuf::from(self.path.clone()));
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

    fn update_suggestions(&mut self) {
        let path = PathBuf::from(&self.path);
        let (dir, prefix) = if path.is_dir() {
            (path.clone(), String::new())
        } else {
            let mut dir = path.clone();
            dir.pop();
            (
                dir,
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            )
        };

        self.search_suggestions.clear();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with(&prefix) {
                    self.search_suggestions.push(file_name);
                }
            }
        }
    }
}

fn get_roaming_path() -> PathBuf {
    dirs::data_dir()
        .unwrap()
        .join("Microsoft/UserSecrets")
        .canonicalize()
        .expect("Failed to fetch local directory!")
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("path_input_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Show Files").clicked() {
                    self.setup();
                }

                let response = egui::TextEdit::singleline(&mut self.path)
                    .font(FontSelection::FontId(egui::FontId {
                        size: 14.0,
                        family: egui::FontFamily::default(),
                    }))
                    .hint_text(format!(
                        "Eg. {}",
                        get_roaming_path().to_string_lossy().to_string()
                    ))
                    .desired_width(ui.available_width())
                    .id("path_input".into())
                    .show(ui)
                    .response;

                if response.changed() {
                    self.update_suggestions();
                }
            });

            if !self.search_suggestions.is_empty() {
                egui::ComboBox::from_label("Suggestions")
                    .selected_text("Select a suggestion")
                    .width(ui.available_width())
                    .height(200.0)
                    .show_ui(ui, |ui| {
                        for suggestion in self.search_suggestions.clone() {
                            let full_path = PathBuf::from(&self.path)
                                .join(suggestion)
                                .to_string_lossy()
                                .to_string();

                            if ui.selectable_label(false, &full_path).clicked() {
                                self.path = full_path;
                                self.setup();
                                self.update_suggestions();
                                ui.close_menu();
                            }
                        }
                    });
            }
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

            if let Some(selected) = self.explorer.ui(ui) {
                self.selected_file = Some(selected.clone());
                self.file_content = fs::read_to_string(&selected)
                    .unwrap_or_else(|_| "<failed to read file>".into());
                self.edited_content = self.file_content.clone();
                self.editor.set_content(self.edited_content.clone());
            }
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
