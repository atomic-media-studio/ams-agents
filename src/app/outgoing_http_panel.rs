use super::AMSAgents;
use eframe::egui;

impl AMSAgents {
    pub(super) fn render_outgoing_http_panel(&mut self, ui: &mut egui::Ui, outgoing_http_height: f32) {
        let panel_border_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        let outgoing_panel = egui::Frame::default()
            .fill(egui::Color32::from_rgb(40, 40, 40))
            .stroke(egui::Stroke::new(1.0, panel_border_color))
            .rounding(4.0)
            .inner_margin(egui::Margin::same(6.0));
        let terminal_frame = egui::Frame::default()
            .fill(egui::Color32::from_rgb(0, 0, 0))
            .stroke(egui::Stroke::new(1.0, panel_border_color))
            .inner_margin(egui::Margin::same(6.0))
            .rounding(4.0);

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), outgoing_http_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                outgoing_panel.show(ui, |ui| {
                    ui.set_min_height(outgoing_http_height);
                    ui.set_max_height(outgoing_http_height);
                    ui.label(egui::RichText::new("Outgoing HTTP").strong().size(12.0));
                    ui.add_space(4.0);
                    let terminal_height = 100.0;
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), terminal_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            terminal_frame.show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.set_min_height(terminal_height);
                                ui.set_max_height(terminal_height);
                                let lines = crate::http_client::get_outgoing_http_log_lines();
                                egui::ScrollArea::vertical()
                                    .id_source(ui.id().with("outgoing_http_scroll"))
                                    .auto_shrink([false, false])
                                    .stick_to_bottom(true)
                                    .show(ui, |ui| {
                                        for line in lines {
                                            ui.label(
                                                egui::RichText::new(line)
                                                    .monospace()
                                                    .size(10.0)
                                                    .color(egui::Color32::WHITE),
                                            );
                                        }
                                    });
                            });
                        },
                    );
                });
            },
        );
    }
}
