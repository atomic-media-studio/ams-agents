use super::AMSAgents;
use eframe::egui;

impl AMSAgents {
    pub(super) fn render_agent_worker_header(
        ui: &mut egui::Ui,
        manager_name: &str,
        worker_global_id: &str,
    ) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Agent Worker").strong().size(12.0));
            ui.small(format!("Manager: {}", manager_name));
            ui.small(format!("ID: {}", worker_global_id));
        });
    }
}
