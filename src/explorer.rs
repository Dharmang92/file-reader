use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use eframe::egui::{self, CollapsingHeader, RichText};
use walkdir::WalkDir;

// const TIME_THRESHOLD: Duration = Duration::from_secs(30);
// const ACCESS_THRESHOLD: u16 = 5;

#[derive(Clone, Debug)]
pub struct Explorer {
    pub name: String,
    pub path: PathBuf,
    pub children: Vec<Explorer>,
    pub is_file: bool,
    pub opened: bool,
    pub initial_open: bool,
    pub last_accessed_time: Instant,
    pub access_count: u16,
}

impl Explorer {
    pub fn from_pathbuf(path: &PathBuf) -> Self {
        let name = Self::name(path);
        Self::folder(name, path.clone(), Some(true))
    }

    fn name(path: &Path) -> String {
        path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    pub fn folder(name: impl Into<String>, path: PathBuf, initial_open: Option<bool>) -> Self {
        Self {
            name: name.into(),
            path,
            children: vec![],
            is_file: false,
            opened: false,
            initial_open: initial_open.unwrap_or(false),
            last_accessed_time: Instant::now(),
            access_count: 0,
        }
    }

    pub fn file(name: impl Into<String>, path: PathBuf) -> Self {
        Self {
            name: name.into(),
            path,
            children: vec![],
            is_file: true,
            opened: true,
            initial_open: true,
            last_accessed_time: Instant::now(),
            access_count: 0,
        }
    }

    fn load_children(&mut self) {
        if !self.children.is_empty() {
            return;
        }

        println!(
            "load_children {:?} {}",
            self.last_accessed_time, self.access_count
        );

        self.children = WalkDir::new(&self.path)
            .max_depth(1)
            .into_iter()
            .skip(1)
            .filter_map(Result::ok)
            .map(|entry| {
                let path = entry.path().to_path_buf();
                let name = Self::name(&path);
                if entry.file_type().is_dir() {
                    Self::folder(name, path, None)
                } else {
                    Self::file(name, path)
                }
            })
            .collect();

        self.last_accessed_time = Instant::now();
        self.access_count += 1;
    }

    // fn clear_children(&mut self) {
    //     for child in &mut self.children {
    //         if !child.is_file {
    //             child.clear_children();
    //         }
    //     }

    //     self.children.clear();
    //     self.opened = false;
    // }

    // fn should_clear_children(&self) -> bool {
    //     if self.is_file || self.opened {
    //         return false;
    //     }

    //     let time_since_access = self.last_accessed_time.elapsed();

    //     time_since_access > TIME_THRESHOLD && self.access_count < ACCESS_THRESHOLD
    // }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<PathBuf> {
        self.ui_impl(ui)
    }

    fn ui_impl(&mut self, ui: &mut egui::Ui) -> Option<PathBuf> {
        if self.is_file {
            // TODO: remove button and directly show file contents on hover
            if ui.button(RichText::new(&self.name)).clicked() {
                return Some(self.path.clone());
            }
        } else {
            let response = CollapsingHeader::new(RichText::new(&self.name))
                .default_open(self.initial_open)
                .show(ui, |ui| {
                    for child in &mut self.children {
                        if let Some(path) = child.ui_impl(ui) {
                            return Some(path);
                        }
                    }
                    None
                });

            if !self.opened && response.fully_open() {
                self.opened = true;
                self.load_children();
            } else if self.opened && response.fully_closed() {
                self.opened = false;
            }

            if let Some(body_returned) = response.body_returned {
                return body_returned;
            }
        }

        None
    }
}
