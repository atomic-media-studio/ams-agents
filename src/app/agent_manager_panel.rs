use super::AMSAgents;
use eframe::egui;

impl AMSAgents {
    pub(super) fn render_agent_manager_header(
        ui: &mut egui::Ui,
        manager_name: &str,
        manager_global_id: &str,
    ) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(manager_name).strong().size(12.0));
            ui.small(format!("Manager: {}", manager_name));
            ui.small(format!("ID: {}", manager_global_id));
        });
    }
}
