//! Collapsible row body: delegates to `body::show_node_body`.

use eframe::egui;

use super::body;
use super::model::AgentNodeKind;
use super::state::AgentRecord;

#[derive(Default)]
pub(super) struct BasicNodeViewer;

impl BasicNodeViewer {
    pub(super) fn numbered_name_for_kind(agents: &[AgentRecord], kind: AgentNodeKind) -> String {
        let idx = agents.iter().filter(|a| a.data.kind == kind).count() + 1;
        format!("{} {}", kind.label(), idx)
    }

    pub(super) fn show_body(&mut self, id: usize, ui: &mut egui::Ui, agents: &mut [AgentRecord]) {
        body::show_node_body(id, ui, agents);
    }
}
