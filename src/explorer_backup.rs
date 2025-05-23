use std::path::PathBuf;

use eframe::egui::{self, CollapsingHeader, RichText};
use walkdir::WalkDir;

#[derive(Clone)]
pub struct Explorer {
    pub name: String,
    pub children: Vec<Explorer>,
    pub is_file: bool,
}

impl Default for Explorer {
    fn default() -> Self {
        Self {
            name: "Unnamed".into(),
            children: Vec::new(),
            is_file: false,
        }
    }
}

impl Explorer {
    pub fn default_dir(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            children: Vec::new(),
            is_file: false,
        }
    }

    pub fn file(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            children: Vec::new(),
            is_file: true,
        }
    }

    fn name(path: PathBuf) -> String {
        path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    pub fn from_pathbuf(path: &PathBuf) -> Self {
        let mut root = Explorer::default_dir(Explorer::name(path.clone()));
        let components: Vec<PathBuf> = WalkDir::new(path)
            .max_depth(1)
            .into_iter()
            .skip(1)
            .filter_map(Result::ok)
            .map(|e| e.path().to_path_buf())
            .collect();

        Explorer::insert_path(&mut root, &components);

        root
    }

    fn insert_path(current: &mut Explorer, parts: &Vec<PathBuf>) {
        if parts.is_empty() {
            return;
        }

        for part in parts {
            current
                .children
                .push(Explorer::file(Explorer::name(part.clone())));
        }

        if current.is_file {
        } else {
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.ui_impl(ui, 0)
    }

    fn ui_impl(&mut self, ui: &mut egui::Ui, depth: usize) {
        if self.is_file {
            // ui.label(format!("{}", self.name));
            if ui.button(RichText::new(&self.name)).clicked() {}
        } else {
            CollapsingHeader::new(&self.name)
                .default_open(depth < 1)
                .show(ui, |ui| self.children_ui(ui, depth));
        }
    }

    fn children_ui(&mut self, ui: &mut egui::Ui, depth: usize) {
        for child in &mut self.children {
            child.ui_impl(ui, depth + 1);
        }
    }
}
