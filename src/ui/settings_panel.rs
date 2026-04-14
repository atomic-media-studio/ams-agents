use crate::agents::AMSAgents;
use crate::ui::AMSAgentsUiState;
use eframe::egui;

/// Preset values for how many recent agent messages are included in the next dialogue prompt.
const CHAT_HISTORY_PRESETS: &[usize] = &[1, 2, 3, 5, 8, 10, 15, 20, 30, 50];

impl AMSAgents {
    /// Chat / dialogue history size (Settings tab, above Reproducibility).
    fn render_chat_settings_widgets(&mut self, ui: &mut egui::Ui) {
        let chat_fold = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id("settings_section_chat"),
            true,
        )
        .show_header(ui, |ui| {
            ui.label(egui::RichText::new("Chat Settings").strong());
        });
        let _ = chat_fold.body(|ui| {
            let mut choices: Vec<usize> = CHAT_HISTORY_PRESETS.to_vec();
            if !choices.contains(&self.conversation_history_size) {
                choices.push(self.conversation_history_size);
                choices.sort_unstable();
            }
            ui.horizontal(|ui| {
                ui.label("History Size:");
                egui::ComboBox::from_id_salt("chat_history_size")
                    .selected_text(format!("{}", self.conversation_history_size))
                    .show_ui(ui, |ui| {
                        for &n in &choices {
                            let label = if n == 1 {
                                "1 message".to_string()
                            } else {
                                format!("{n} messages")
                            };
                            ui.selectable_value(&mut self.conversation_history_size, n, label);
                        }
                    });
            });
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(
                    "Number of recent agent replies kept in context for the next turn.",
                )
                .small()
                .weak(),
            );
        });
    }

    pub(super) fn render_ollama_settings_widgets(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        ui_state: &mut AMSAgentsUiState,
    ) {
        ui.vertical(|ui| {
            let settings_fold = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                ui.make_persistent_id("ollama_section_settings"),
                true,
            )
            .show_header(ui, |ui| {
                ui.label(egui::RichText::new("Settings Ollama").strong());
            });
            let _ = settings_fold.body(|ui| {
                ui.horizontal(|ui| {
                    ui.label("API host / URL:");
                    ui.add(egui::TextEdit::singleline(&mut self.ollama_host).desired_width(300.0));
                });
            });

            let test_fold = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                ui.make_persistent_id("ollama_section_test"),
                true,
            )
            .show_header(ui, |ui| {
                ui.label(egui::RichText::new("Test Ollama").strong());
            });
            let _ = test_fold.body(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Ollama Model:");
                    let models = ui_state.ollama.models.lock().unwrap().clone();
                    if self.selected_ollama_model.is_empty() {
                        if let Some(first) = models.first() {
                            self.selected_ollama_model = first.clone();
                        }
                    }
                    egui::ComboBox::from_id_salt("ollama_model_selector")
                        .selected_text(if self.selected_ollama_model.is_empty() {
                            "Select model".to_string()
                        } else {
                            self.selected_ollama_model.clone()
                        })
                        .show_ui(ui, |ui| {
                            for model in &models {
                                ui.selectable_value(
                                    &mut self.selected_ollama_model,
                                    model.clone(),
                                    model,
                                );
                            }
                        });

                    let loading = *ui_state.ollama.models_loading.lock().unwrap();
                    if ui
                        .button(if loading { "Loading" } else { "Refresh" })
                        .clicked()
                        && !loading
                    {
                        *ui_state.ollama.models_loading.lock().unwrap() = true;
                        let models_arc = ui_state.ollama.models.clone();
                        let loading_arc = ui_state.ollama.models_loading.clone();
                        let ctx = ctx.clone();
                        let handle = self.rt_handle.clone();
                        let ollama_host = self.ollama_host.clone();
                        handle.spawn(async move {
                            let models = crate::ollama::fetch_ollama_models(&ollama_host)
                                .await
                                .unwrap_or_default();
                            *models_arc.lock().unwrap() = models;
                            *loading_arc.lock().unwrap() = false;
                            ctx.request_repaint();
                        });
                    }
                });
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("Chat HTTP Endpoint:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.http_endpoint).desired_width(260.0),
                    );
                });
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    if ui.button("Test API").clicked() {
                        println!("Pinging Ollama");
                        let ctx = ctx.clone();
                        let handle = self.rt_handle.clone();
                        let model = self.selected_ollama_model.clone();
                        let ollama_host = self.ollama_host.clone();
                        handle.spawn(async move {
                            match crate::ollama::test_ollama(
                                ollama_host.as_str(),
                                if model.trim().is_empty() {
                                    None
                                } else {
                                    Some(model.as_str())
                                },
                            )
                            .await
                            {
                                Ok(_) => {}
                                Err(e) => eprintln!("Ollama error: {}", e),
                            }
                            ctx.request_repaint();
                        });
                    }
                });
            });
        });
    }

    pub(super) fn render_reproducibility_settings_widgets(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            self.render_chat_settings_widgets(ui);
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Air-gap Policy").strong().size(12.0));
            ui.separator();

            let mut policy_changed = false;
            ui.horizontal(|ui| {
                if ui
                    .checkbox(
                        &mut self.air_gap_enabled,
                        "Enable air-gap mode (block non-loopback HTTP)",
                    )
                    .changed()
                {
                    policy_changed = true;
                }
            });
            ui.horizontal(|ui| {
                if ui
                    .checkbox(
                        &mut self.allow_local_ollama,
                        "Allow local Ollama (127.0.0.1 / ::1 / localhost)",
                    )
                    .changed()
                {
                    policy_changed = true;
                }
            });

            if self.air_gap_enabled {
                ui.label(
                    egui::RichText::new(
                        "Verification: outbound HTTP is blocked unless target is loopback; blocked attempts are written to the run ledger as transport.http_blocked.",
                    )
                    .small(),
                );
                if self.allow_local_ollama {
                    ui.label(
                        egui::RichText::new("Allowed: loopback Ollama + local file I/O.")
                            .small()
                            .weak(),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(
                            "Allowed: local file I/O only (Ollama requests are blocked).",
                        )
                        .small()
                        .weak(),
                    );
                }
            } else {
                ui.label(
                    egui::RichText::new("Verification: outbound HTTP is currently enabled.")
                        .small()
                        .weak(),
                );
            }

            if policy_changed {
                self.sync_http_policy();
            }
        });
    }
}
