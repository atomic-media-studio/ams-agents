use super::AMSAgents;
use eframe::egui;

impl AMSAgents {
    pub(super) fn render_settings_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let available_width = ui.available_width() - 12.0;
        let settings_bg_color = egui::Color32::from_rgb(40, 40, 40);

        egui::Frame::default()
            .fill(settings_bg_color)
            .inner_margin(egui::Margin {
                left: 6.0,
                right: 6.0,
                top: 6.0,
                bottom: 6.0,
            })
            .rounding(4.0)
            .show(ui, |ui| {
                ui.set_width(available_width);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Settings").strong().size(12.0));
                    ui.separator();

                    let panel_border_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
                    let settings_panel = egui::Frame::default()
                        .fill(settings_bg_color)
                        .stroke(egui::Stroke::new(1.0, panel_border_color))
                        .rounding(4.0)
                        .inner_margin(egui::Margin::same(6.0));

                    settings_panel.show(ui, |ui| {
                        let subpanel = egui::Frame::default()
                            .fill(settings_bg_color)
                            .stroke(egui::Stroke::new(1.0, panel_border_color))
                            .rounding(4.0)
                            .inner_margin(egui::Margin::same(6.0));

                        ui.horizontal_top(|ui| {
                            let total_agents_count = self.managers.len()
                                + self.agents.len()
                                + self.evaluators.len()
                                + self.researchers.len();

                            subpanel.show(ui, |ui| {
                                ui.set_min_width(180.0);
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new("AMSAgents").strong().size(12.0));
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new(total_agents_count.to_string()).size(16.0));
                                    ui.add_space(6.0);
                                    if ui.button("Create Manager").clicked() {
                                        let used_ids: std::collections::HashSet<usize> =
                                            self.managers.iter().map(|m| m.id).collect();
                                        let mut new_id = 1;
                                        while used_ids.contains(&new_id) {
                                            new_id += 1;
                                        }
                                        let global_id = self.generate_global_id();
                                        self.managers.push(crate::agent_entities::AgentManager::new(new_id, global_id));
                                        if new_id >= self.next_manager_id {
                                            self.next_manager_id = new_id + 1;
                                        }
                                    }
                                });
                            });

                            subpanel.show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Ollama Model:");
                                        let models = self.ollama_models.lock().unwrap().clone();
                                        if self.selected_ollama_model.is_empty() {
                                            if let Some(first) = models.first() {
                                                self.selected_ollama_model = first.clone();
                                            }
                                        }
                                        egui::ComboBox::from_id_source("ollama_model_selector")
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

                                        let loading = *self.ollama_models_loading.lock().unwrap();
                                        if ui.button(if loading { "Loading" } else { "Refresh" }).clicked()
                                            && !loading
                                        {
                                            *self.ollama_models_loading.lock().unwrap() = true;
                                            let models_arc = self.ollama_models.clone();
                                            let loading_arc = self.ollama_models_loading.clone();
                                            let ctx = ctx.clone();
                                            let handle = self.rt_handle.clone();
                                            handle.spawn(async move {
                                                let models = crate::adk_integration::fetch_ollama_models()
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
                                            egui::TextEdit::singleline(&mut self.http_endpoint)
                                                .desired_width(200.0),
                                        );
                                    });
                                    ui.add_space(5.0);

                                    ui.horizontal(|ui| {
                                        if ui.button("Test API").clicked() {
                                            println!("Pinging Ollama");
                                            let ctx = ctx.clone();
                                            let handle = self.rt_handle.clone();
                                            let model = self.selected_ollama_model.clone();
                                            handle.spawn(async move {
                                                match crate::adk_integration::test_ollama(
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
                                    ui.add_space(5.0);
                                    ui.horizontal(|ui| {
                                        ui.label("Turn Delay (s):");
                                        ui.add(
                                            egui::DragValue::new(&mut self.conversation_turn_delay_secs)
                                                .range(0..=60)
                                                .speed(0.1),
                                        );
                                        ui.label("History:");
                                        ui.add(
                                            egui::DragValue::new(&mut self.conversation_history_size)
                                                .range(1..=50)
                                                .speed(0.1),
                                        );
                                    });
                                });
                            });
                        });
                    });
                });
            });
    }
}
