/*
// Copyright (C) 2020-2025 Pen, Dice & Paper
//
// This program is dual-licensed under the following terms:
//
// Option 1: (Non-Commercial) GNU Affero General Public License (AGPL)
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.
//
// Option 2: Commercial License
// For commercial use, you are required to obtain a separate commercial
// license. Please contact ithai at pendicepaper.com
// for more information about commercial licensing terms.
*/
use egui::{Color32, Context, CursorIcon, Ui};

use crate::app::HexrollTestbedApp;

impl HexrollTestbedApp {
    pub fn top_app_bar(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.horizontal_centered(|ui| {
            ui.add(
                egui::Image::new(egui::include_image!("../../../assets/icon.png")).rounding(5.0),
            );
            ui.heading("HEXROLL3 / Testbed");
            ui.label(egui::RichText::new("Theme").monospace());
            egui::global_theme_preference_buttons(ui);
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            ui.label("Scroll file path:");
            ui.scope(|ui| {
                if self.scroll_in_filepath_is_valid {
                    ui.style_mut().visuals.override_text_color = Some(Color32::LIGHT_BLUE);
                } else {
                    ui.style_mut().visuals.override_text_color = Some(Color32::LIGHT_RED);
                }
                if ui
                    .text_edit_singleline(&mut self.config.main_scroll_filepath)
                    .changed()
                {
                    self.scroll_in_filepath_is_valid = false;
                    self.save_settings()
                        .map_err(|e| log::warn!("Unable to save configuration. Error is {:#}", e))
                        .ok();
                }
            });
            if !self.scroll_in_filepath_is_valid {
                ui.scope(|ui| {
                    ui.style_mut().visuals.override_text_color = Some(Color32::LIGHT_RED);
                    ui.label("Click");
                    if ui.button("Test").clicked() {
                        self.scroll_in_filepath_is_valid = HexrollTestbedApp::test_scroll_filepath(
                            &self.config.main_scroll_filepath,
                        );
                    }
                    ui.label("to verify the scroll. Check the trace logs for errors.");
                });
            }
            if self.instance.is_some() && self.scroll_in_filepath_is_valid {
                ui.menu_button("⬣", |ui| {
                    self.open_or_roll_fragment(ui);
                    ui.separator();
                    if ui.button("Reload Scroll").clicked() {
                        self.instance
                            .as_mut()
                            .unwrap()
                            .with_scroll((&self.config.main_scroll_filepath).into())
                            .expect("That's odd!");
                    }
                });
            }
            if let Some(mut path) = self.file.take_selected() {
                match self.file.mode() {
                    egui_file_dialog::DialogMode::SaveFile => {
                        ctx.set_cursor_icon(CursorIcon::Wait);
                        if path.set_extension("h3")
                            && self.roll_new_sandbox(path.to_str().unwrap()).is_err()
                        {
                            self.instance = None;
                        }
                        ctx.set_cursor_icon(CursorIcon::Default);
                    }
                    egui_file_dialog::DialogMode::SelectFile => {
                        self.open_existing_sandbox(path.to_str().unwrap())
                            .map_err(|_| self.instance = None)
                            .ok();
                    }
                    _ => {}
                }
            }
        });
    }
}
