use eframe::egui::{self, TextStyle, TextWrapMode};

pub struct Table {
    pub selection: usize,
    pub resizable: bool,
    pub clickable: bool,
    pub num_rows: usize,
}

impl Default for Table {
    fn default() -> Self {
        Self {
            selection: 0,
            resizable: true,
            clickable: true,
            num_rows: 0,
        }
    }
}

impl Table {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let mut reset = false;

        ui.vertical(|ui| {
            reset = ui.button("Reset").clicked();
        });

        ui.separator();

        use egui_extras::{Size, StripBuilder};
        StripBuilder::new(ui)
            .size(Size::remainder().at_least(100.0))
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        self.table_ui(ui, reset);
                    });
                });
            });
    }

    fn table_ui(&mut self, ui: &mut egui::Ui, reset: bool) {
        use egui_extras::{Column, TableBuilder};

        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        let available_height = ui.available_height();
        let mut table = TableBuilder::new(ui)
            .resizable(self.resizable)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(
                Column::remainder()
                    .at_least(40.0)
                    .clip(true)
                    .resizable(true),
            )
            .column(Column::auto())
            .column(Column::remainder())
            .column(Column::remainder())
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);

        if self.clickable {
            table = table.sense(egui::Sense::click());
        }

        if reset {
            table.reset();
        }

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("ID");
                });
                header.col(|ui| {
                    ui.strong("File Name");
                });
            })
            .body(|body| {
                body.rows(text_height, self.num_rows, |mut row| {
                    let row_index = row.index();

                    row.set_selected(self.selection == row_index);

                    row.col(|ui| {
                        ui.label(row_index.to_string());
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new("Thousands of rows of even height")
                                .wrap_mode(TextWrapMode::Extend),
                        );
                    });

                    self.toggle_row_selection(row_index, &row.response());
                });
            });
    }

    fn toggle_row_selection(&mut self, row_index: usize, row_response: &egui::Response) {
        if row_response.clicked() {
            if self.selection == row_index {
                self.selection = 0;
            } else {
                self.selection = row_index;
            }
        }
    }
}
