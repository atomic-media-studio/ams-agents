use super::AMSAgents;
use eframe::egui;

impl AMSAgents {
    pub(super) fn render_agent_evaluator_header(
        ui: &mut egui::Ui,
        manager_name: &str,
        evaluator_global_id: &str,
    ) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Agent Evaluator").strong().size(12.0));
            ui.small(format!("Manager: {}", manager_name));
            ui.small(format!("ID: {}", evaluator_global_id));
        });
    }
}
