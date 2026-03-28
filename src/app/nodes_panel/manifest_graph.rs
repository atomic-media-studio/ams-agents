use crate::reproducibility::ManifestEdge;

use super::model::NodePayload;
use super::state::AgentRecord;

/// Refresh evaluator/researcher `active` from row selections (combiners may skip the row body when collapsed).
pub(super) fn sync_evaluator_researcher_activity(agents: &mut [AgentRecord]) {
    for r in agents.iter_mut() {
        match &mut r.data.payload {
            NodePayload::Evaluator(e) => {
                e.active = e.evaluate_all_workers || e.worker_node.is_some();
            }
            NodePayload::Researcher(res) => {
                res.active = res.worker_node.is_some();
            }
            _ => {}
        }
    }
}

pub(super) fn manifest_edges_from_agents(agents: &[AgentRecord]) -> Vec<ManifestEdge> {
    let mut edges = Vec::new();
    for r in agents {
        let nid = r.id;
        match &r.data.payload {
            NodePayload::Worker(w) => {
                if let Some(m) = w.manager_node {
                    edges.push(ManifestEdge {
                        from_node_id: m,
                        from_output_pin: 0,
                        to_node_id: nid,
                        to_input_pin: 0,
                    });
                }
                if let Some(t) = w.topic_node {
                    edges.push(ManifestEdge {
                        from_node_id: t,
                        from_output_pin: 0,
                        to_node_id: nid,
                        to_input_pin: 1,
                    });
                }
            }
            NodePayload::Evaluator(e) => {
                if let Some(m) = e.manager_node {
                    edges.push(ManifestEdge {
                        from_node_id: m,
                        from_output_pin: 0,
                        to_node_id: nid,
                        to_input_pin: 0,
                    });
                }
                if let Some(wid) = e.worker_node {
                    if !e.evaluate_all_workers {
                        edges.push(ManifestEdge {
                            from_node_id: wid,
                            from_output_pin: 0,
                            to_node_id: nid,
                            to_input_pin: 1,
                        });
                    }
                }
            }
            NodePayload::Researcher(res) => {
                if let Some(m) = res.manager_node {
                    edges.push(ManifestEdge {
                        from_node_id: m,
                        from_output_pin: 0,
                        to_node_id: nid,
                        to_input_pin: 0,
                    });
                }
                if let Some(wid) = res.worker_node {
                    edges.push(ManifestEdge {
                        from_node_id: wid,
                        from_output_pin: 0,
                        to_node_id: nid,
                        to_input_pin: 1,
                    });
                }
            }
            _ => {}
        }
    }
    edges.sort_by_key(|e| {
        (
            e.from_node_id,
            e.from_output_pin,
            e.to_node_id,
            e.to_input_pin,
        )
    });
    edges
}
