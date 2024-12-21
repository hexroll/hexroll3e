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
use egui::Ui;

use crate::app::HexrollTestbedApp;

impl HexrollTestbedApp {
    pub fn open_or_roll_panel(&mut self, ctx: &egui::Context) {
        if !self.scroll_in_filepath_is_valid {
            return;
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.set_max_width(300.0);
                ui.horizontal_centered(|ui| {
                    ui.set_max_height(50.0);
                    self.open_or_roll_fragment(ui);
                });

                let mrus = &self.config.recently_opened.clone();
                mrus.iter().rev().for_each(|v| {
                    if ui.link(v).clicked() {
                        self.open_existing_sandbox(v)
                            .map_err(|_| {
                                log::warn!("Unable to open file {}", v);
                            })
                            .ok();
                    }
                });
            });
        });
    }

    pub fn open_or_roll_fragment(&mut self, ui: &mut Ui) {
        if ui.button("Roll a new sandbox").clicked() {
            self.roll_new_sandbox_dialog(ui);
        }
        ui.separator();
        if ui.button("Open an existing sandbox").clicked() {
            self.open_existing_sandbox_dialog();
        }
    }
}
