//! Panel tab selection and the Agents list (`NodesPanelState`).

use eframe::egui;

use super::model::NodeData;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PanelTab {
    Overview,
    Agents,
    Ollama,
    Settings,
}

/// One row in the Agents list (stable `id` for manifests and conversation loops).
#[derive(Clone)]
pub(super) struct AgentRecord {
    pub id: usize,
    pub position: egui::Pos2,
    pub open: bool,
    pub data: super::model::NodeData,
}

pub struct NodesPanelState {
    pub(super) next_agent_id: usize,
    pub(super) agents: Vec<AgentRecord>,
    pub(super) selected_add_kind: super::model::AgentNodeKind,
    pub(super) active_tab: PanelTab,
}

impl Default for NodesPanelState {
    fn default() -> Self {
        Self {
            next_agent_id: 0,
            agents: Vec::new(),
            selected_add_kind: super::model::AgentNodeKind::Worker,
            active_tab: PanelTab::Overview,
        }
    }
}

impl NodesPanelState {
    pub(super) fn push_agent(&mut self, pos: egui::Pos2, data: NodeData) -> usize {
        let id = self.next_agent_id;
        self.next_agent_id += 1;
        self.agents.push(AgentRecord {
            id,
            position: pos,
            open: true,
            data,
        });
        id
    }

    pub(super) fn insert_agent_with_id(
        &mut self,
        id: usize,
        pos: egui::Pos2,
        open: bool,
        data: NodeData,
    ) {
        self.agents.push(AgentRecord {
            id,
            position: pos,
            open,
            data,
        });
        if id + 1 > self.next_agent_id {
            self.next_agent_id = id + 1;
        }
    }

    pub(super) fn remove_agent(&mut self, id: usize) {
        self.agents.retain(|a| a.id != id);
    }

    pub(super) fn agent_by_id_mut(&mut self, id: usize) -> Option<&mut AgentRecord> {
        self.agents.iter_mut().find(|a| a.id == id)
    }
}
