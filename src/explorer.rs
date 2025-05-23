use std::path::{Path, PathBuf};

use eframe::egui::{self, CollapsingHeader, RichText};
use walkdir::WalkDir;

#[derive(Clone)]
pub struct Explorer {
    pub name: String,
    pub path: PathBuf,
    pub children: Vec<Explorer>,
    pub is_file: bool,
    pub loaded: bool,
}

impl Explorer {
    pub fn default_dir(name: impl Into<String>, path: PathBuf) -> Self {
        Self {
            name: name.into(),
            path,
            children: vec![],
            is_file: false,
            loaded: false,
        }
    }

    pub fn file(name: impl Into<String>, path: PathBuf) -> Self {
        Self {
            name: name.into(),
            path,
            children: vec![],
            is_file: true,
            loaded: true,
        }
    }

    fn name(path: &Path) -> String {
        path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    pub fn from_pathbuf(path: &PathBuf) -> Self {
        let name = Self::name(path);
        let mut root = Self::default_dir(name, path.clone());
        root.load_children(); // Load children once on setup
        root
    }

    fn load_children(&mut self) {
        if self.loaded || self.is_file {
            return;
        }

        self.children = WalkDir::new(&self.path)
            .max_depth(1)
            .into_iter()
            .skip(1)
            .filter_map(Result::ok)
            .map(|entry| {
                let path = entry.path().to_path_buf();
                let name = Self::name(&path);
                if entry.file_type().is_dir() {
                    Self::default_dir(name, path)
                } else {
                    Self::file(name, path)
                }
            })
            .collect();

        self.loaded = true;
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<PathBuf> {
        self.ui_impl(ui)
    }

    fn ui_impl(&mut self, ui: &mut egui::Ui) -> Option<PathBuf> {
        if self.is_file {
            if ui.button(RichText::new(&self.name)).clicked() {
                return Some(self.path.clone());
            }
        } else {
            let response = CollapsingHeader::new(RichText::new(&self.name))
                .default_open(self.children.len() < 3)
                .show(ui, |ui| {
                    self.load_children();
                    for child in &mut self.children {
                        if let Some(path) = child.ui_impl(ui) {
                            return Some(path);
                        }
                    }
                    None
                });

            if let Some(body_returned) = response.body_returned {
                return body_returned;
            }
        }

        None
    }
}
