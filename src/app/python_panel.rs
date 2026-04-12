//! Python runtime management panel — issue #16.
//! Rendered as the "Python" tab between "Ollama" and "Settings".

use std::sync::atomic::Ordering;
use std::sync::Arc;

use eframe::egui;

use super::AMSAgents;
use crate::python_runtime::{
    create_runtime, default_registry_path, default_runtimes_dir, delete_runtime,
    install_packages_in_runtime, PythonRuntimeSpec, RuntimeRegistry,
};

impl AMSAgents {
    pub(super) fn render_python_panel(&mut self, ui: &mut egui::Ui) {
        // ── Poll results from background tasks ───────────────────────────
        if let Some(result) = self.python_bg_new_runtime.lock().unwrap().take() {
            match result {
                Ok(rt) => {
                    self.python_status = format!(
                        "Runtime '{}' created (Python {}).",
                        rt.label, rt.python_version
                    );
                    self.python_active_runtime = Some(rt);
                }
                Err(e) => {
                    self.python_status = format!("Error: {e}");
                }
            }
            self.python_op_running.store(false, Ordering::Relaxed);
        }
        if let Some(msg) = self.python_bg_msg.lock().unwrap().take() {
            self.python_status = msg;
            self.python_op_running.store(false, Ordering::Relaxed);
        }
        if self.python_bg_destroyed.swap(false, Ordering::Relaxed) {
            self.python_active_runtime = None;
            self.python_status = "Runtime destroyed and removed.".to_string();
            self.python_op_running.store(false, Ordering::Relaxed);
        }

        let running = self.python_op_running.load(Ordering::Relaxed);
        if running {
            ui.ctx().request_repaint();
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(4.0);

            // ── No active runtime → creation form ────────────────────────
            if self.python_active_runtime.is_none() {
                ui.label(egui::RichText::new("New Python Environment").strong());
                ui.separator();
                ui.add_space(4.0);

                egui::Grid::new("py_create_grid")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Label:");
                        ui.add_enabled(
                            !running,
                            egui::TextEdit::singleline(&mut self.python_label_input)
                                .desired_width(220.0)
                                .hint_text("StroopTask-py3.11"),
                        );
                        ui.end_row();

                        ui.label("Interpreter:");
                        ui.add_enabled(
                            !running,
                            egui::TextEdit::singleline(&mut self.python_interpreter_input)
                                .desired_width(220.0),
                        );
                        ui.end_row();
                    });

                ui.add_space(6.0);

                if running {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Creating environment…");
                    });
                } else {
                    let label_ok = !self.python_label_input.trim().is_empty();
                    if ui
                        .add_enabled(label_ok, egui::Button::new("  Create env  "))
                        .on_disabled_hover_text("Enter a label first")
                        .clicked()
                    {
                        let label = self.python_label_input.trim().to_string();
                        let interpreter = self.python_interpreter_input.trim().to_string();
                        let bg_rt = Arc::clone(&self.python_bg_new_runtime);
                        let runtimes_dir = default_runtimes_dir();
                        self.python_op_running.store(true, Ordering::Relaxed);
                        self.python_status = "Creating…".to_string();
                        self.rt_handle.spawn_blocking(move || {
                            let spec = PythonRuntimeSpec {
                                base_interpreter: interpreter,
                                requirements: vec![],
                                post_install_commands: vec![],
                            };
                            let result = create_runtime(spec, &label, "ui", &runtimes_dir)
                                .and_then(|rt| {
                                    let reg_path = default_registry_path();
                                    let mut reg =
                                        RuntimeRegistry::load(&reg_path).unwrap_or_default();
                                    reg.runtimes.push(rt.clone());
                                    reg.save(&reg_path)?;
                                    Ok(rt)
                                })
                                .map_err(|e| e.to_string());
                            *bg_rt.lock().unwrap() = Some(result);
                        });
                    }
                }
            } else {
                // ── Active runtime → info + install + destroy ─────────────
                // Clone display data upfront to release the borrow on python_active_runtime
                // so button click handlers can freely borrow other self fields.
                let (rt_id, rt_label, rt_version, rt_path, rt_state, rt_cloned) = {
                    let rt = self.python_active_runtime.as_ref().unwrap();
                    (
                        rt.id.clone(),
                        rt.label.clone(),
                        rt.python_version.clone(),
                        rt.root_path
                            .as_ref()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| "—".to_string()),
                        format!("{:?}", rt.state),
                        rt.clone(),
                    )
                };

                ui.label(egui::RichText::new("Active Runtime").strong());
                ui.separator();
                egui::Grid::new("py_rt_info")
                    .num_columns(2)
                    .spacing([8.0, 2.0])
                    .show(ui, |ui| {
                        ui.label("ID:");
                        ui.label(egui::RichText::new(&rt_id).monospace());
                        ui.end_row();
                        ui.label("Label:");
                        ui.label(&rt_label);
                        ui.end_row();
                        ui.label("Python:");
                        ui.label(&rt_version);
                        ui.end_row();
                        ui.label("Path:");
                        ui.label(&rt_path);
                        ui.end_row();
                        ui.label("State:");
                        ui.label(&rt_state);
                        ui.end_row();
                    });

                ui.add_space(8.0);

                // ── Install packages ──────────────────────────────────────
                ui.label(egui::RichText::new("Install Packages").strong());
                ui.separator();
                ui.label(
                    egui::RichText::new("One package per line, e.g.  numpy>=1.26")
                        .small()
                        .weak(),
                );
                ui.add_space(2.0);
                ui.add_enabled(
                    !running,
                    egui::TextEdit::multiline(&mut self.python_pkg_input)
                        .desired_width(f32::INFINITY)
                        .desired_rows(4)
                        .hint_text("numpy>=1.26\npsychopy==2024.1.0"),
                );
                ui.add_space(4.0);

                if running {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Working…");
                    });
                } else {
                    ui.horizontal(|ui| {
                        // Install
                        if ui.button("Install").clicked() {
                            let packages: Vec<String> = self
                                .python_pkg_input
                                .lines()
                                .map(str::trim)
                                .filter(|l| !l.is_empty())
                                .map(String::from)
                                .collect();
                            if !packages.is_empty() {
                                let rt = rt_cloned.clone();
                                let bg_msg = Arc::clone(&self.python_bg_msg);
                                self.python_op_running.store(true, Ordering::Relaxed);
                                self.python_status = "Installing…".to_string();
                                self.rt_handle.spawn_blocking(move || {
                                    let result =
                                        install_packages_in_runtime(&rt, &packages)
                                            .map(|out| {
                                                format!("Install complete.\n{}", out.trim())
                                            })
                                            .unwrap_or_else(|e| format!("Error: {e}"));
                                    *bg_msg.lock().unwrap() = Some(result);
                                });
                            }
                        }

                        ui.add_space(16.0);

                        // Destroy
                        let destroy_btn = ui.add(egui::Button::new(
                            egui::RichText::new("Destroy env")
                                .color(ui.visuals().error_fg_color),
                        ));
                        if destroy_btn.clicked() {
                            let rt_id_del = rt_id.clone();
                            let bg_msg = Arc::clone(&self.python_bg_msg);
                            let bg_destroyed = Arc::clone(&self.python_bg_destroyed);
                            self.python_op_running.store(true, Ordering::Relaxed);
                            self.python_status = "Destroying…".to_string();
                            self.rt_handle.spawn_blocking(move || {
                                let reg_path = default_registry_path();
                                let outcome =
                                    RuntimeRegistry::load(&reg_path).and_then(|mut reg| {
                                        delete_runtime(&mut reg, &rt_id_del)?;
                                        reg.save(&reg_path)
                                    });
                                match outcome {
                                    Ok(()) => bg_destroyed.store(true, Ordering::Relaxed),
                                    Err(e) => {
                                        *bg_msg.lock().unwrap() = Some(format!("Error: {e}"))
                                    }
                                }
                            });
                        }
                    });
                }
            }

            // ── Status bar ────────────────────────────────────────────────
            if !self.python_status.is_empty() {
                ui.add_space(8.0);
                ui.separator();
                ui.label(egui::RichText::new(&self.python_status).small().weak());
            }
        });
    }
}
