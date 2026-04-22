use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::agents::AMSAgents;
use crate::run::event_ledger::EventLedger;
use crate::run::manifest::runs_root;
use super::manifest_ops::sync_evaluator_researcher_activity;
use super::model::NodePayload;
use super::play_plan::{
    PlayConversationPairJson, PlayWorkerInPlayJson, build_conversation_sidecar_from_agents,
    collect_run_play_plan_from_agents,
};

impl AMSAgents {
    pub(crate) fn stop_graph(&mut self) {
        self.ollama_run_epoch.fetch_add(1, Ordering::SeqCst);
        for (_, flag, _) in &self.conversation_loop_handles {
            *flag.lock().unwrap() = false;
        }
        self.conversation_loop_handles.clear();
        self.conversation_graph_running
            .store(false, Ordering::Release);
        *self.last_message_in_chat.lock().unwrap() = None;
        self.conversation_message_events.lock().unwrap().clear();
        self.evaluator_event_queues.lock().unwrap().clear();
        self.researcher_event_queues.lock().unwrap().clear();
        self.last_evaluated_message_by_evaluator
            .lock()
            .unwrap()
            .clear();
        self.last_researched_message_by_researcher
            .lock()
            .unwrap()
            .clear();
    }

    /// Returns a UI-facing status message for the most recent run action.
    pub(crate) fn run_graph(&mut self) -> String {
        // Bulletproof behavior: re-run means stop existing graph processes first.
        self.stop_graph();
        self.last_evaluated_message_by_evaluator
            .lock()
            .unwrap()
            .clear();
        self.last_researched_message_by_researcher
            .lock()
            .unwrap()
            .clear();
        self.evaluator_event_queues.lock().unwrap().clear();
        self.researcher_event_queues.lock().unwrap().clear();
        self.evaluator_inflight_nodes.lock().unwrap().clear();
        self.researcher_inflight_nodes.lock().unwrap().clear();

        let experiment_id_override = if self.read_only_replay_mode {
            self.current_manifest
                .as_ref()
                .map(|m| m.experiment_id.clone())
        } else {
            None
        };
        let manifest =
            match self.build_run_manifest(experiment_id_override, self.read_only_replay_mode) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("[Run Graph] Manifest build failed: {e}");
                    return format!("Manifest build failed: {e}");
                }
            };
        let manifest_path = match self.persist_active_manifest(manifest) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[Run Graph] Manifest save failed: {e}");
                return format!("Manifest save failed: {e}");
            }
        };
        let mut status_message = format!("Manifest saved: {}", manifest_path.display());

        if let Some(ctx) = self.current_run_context.as_ref() {
            let run_dir = manifest_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| runs_root().join(&ctx.experiment_id).join(&ctx.run_id));
            match EventLedger::open(run_dir, ctx.experiment_id.clone(), ctx.run_id.clone()) {
                Ok(ledger) => {
                    let arc = Arc::new(ledger);
                    if let Err(e) = arc.append_system_run_started(&manifest_path) {
                        status_message = format!("Ledger start failed: {e}");
                        eprintln!("[Run Graph] Ledger start failed: {e}");
                    } else {
                        self.event_ledger = Some(arc);
                    }
                }
                Err(e) => {
                    status_message = format!("Ledger open failed: {e}");
                    eprintln!("[Run Graph] Ledger open failed: {e}");
                }
            }
        }

        sync_evaluator_researcher_activity(&mut self.nodes_panel.agents);
        let sidecars = std::sync::Arc::new(build_conversation_sidecar_from_agents(
            &self.nodes_panel.agents,
        ));

        // Workers with a non-empty topic, stable order for pairing (row order by id).
        struct EligibleWorker {
            id: usize,
            name: String,
            instruction: String,
            topic: String,
            topic_source: String,
        }

        let mut eligible: Vec<EligibleWorker> = self
            .nodes_panel
            .agents
            .iter()
            .filter_map(|r| {
                if let NodePayload::Worker(w) = &r.data.payload {
                    if !w.conversation_topic.trim().is_empty() {
                        return Some(EligibleWorker {
                            id: r.id,
                            name: w.name.clone(),
                            instruction: w.instruction.clone(),
                            topic: w.conversation_topic.clone(),
                            topic_source: w.conversation_topic_source.clone(),
                        });
                    }
                }
                None
            })
            .collect();
        eligible.sort_by_key(|w| w.id);

        if eligible.is_empty() {
            let play_plan = collect_run_play_plan_from_agents(&self.nodes_panel.agents, vec![]);
            match serde_json::to_string_pretty(&play_plan) {
                Ok(json) => println!("[Run Graph] play plan:\n{}", json),
                Err(e) => eprintln!("[Run Graph] failed to serialize play plan: {e}"),
            }
            if let Some(ref l) = self.event_ledger {
                let _ = l.try_finalize_run_stopped("no_eligible_conversation_workers");
            }
            return status_message;
        }

        let n_conversation_loops = (eligible.len() + 1) / 2;
        self.conversation_run_generation
            .fetch_add(1, Ordering::SeqCst);
        let run_generation = self.conversation_run_generation.load(Ordering::SeqCst);
        let loops_remaining = Arc::new(AtomicUsize::new(n_conversation_loops));
        let gen_counter = self.conversation_run_generation.clone();
        let graph_running_flag = self.conversation_graph_running.clone();

        let mut conversations_plan = Vec::new();
        let mut i = 0;
        while i < eligible.len() {
            if i + 1 < eligible.len() {
                let a = &eligible[i];
                let b = &eligible[i + 1];
                let gid_a = self
                    .nodes_panel
                    .agents
                    .iter()
                    .find(|r| r.id == a.id)
                    .and_then(|r| {
                        if let NodePayload::Worker(w) = &r.data.payload {
                            Some(w.global_id.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                let gid_b = self
                    .nodes_panel
                    .agents
                    .iter()
                    .find(|r| r.id == b.id)
                    .and_then(|r| {
                        if let NodePayload::Worker(w) = &r.data.payload {
                            Some(w.global_id.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                conversations_plan.push(PlayConversationPairJson {
                    loop_key_node_id: a.id,
                    agent_a: PlayWorkerInPlayJson {
                        node_id: a.id,
                        name: a.name.clone(),
                        global_id: gid_a,
                        conversation_topic: a.topic.clone(),
                        conversation_topic_source: a.topic_source.clone(),
                    },
                    agent_b: PlayWorkerInPlayJson {
                        node_id: b.id,
                        name: b.name.clone(),
                        global_id: gid_b,
                        conversation_topic: b.topic.clone(),
                        conversation_topic_source: b.topic_source.clone(),
                    },
                    solo: false,
                });
                self.start_conversation_from_node_worker_resolved(
                    sidecars.clone(),
                    run_generation,
                    gen_counter.clone(),
                    loops_remaining.clone(),
                    graph_running_flag.clone(),
                    a.id,
                    a.id,
                    a.name.clone(),
                    a.instruction.clone(),
                    a.topic.clone(),
                    a.topic_source.clone(),
                    b.id,
                    b.name.clone(),
                    b.instruction.clone(),
                    b.topic.clone(),
                    b.topic_source.clone(),
                );
                i += 2;
            } else {
                let a = &eligible[i];
                let gid = self
                    .nodes_panel
                    .agents
                    .iter()
                    .find(|r| r.id == a.id)
                    .and_then(|r| {
                        if let NodePayload::Worker(w) = &r.data.payload {
                            Some(w.global_id.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                conversations_plan.push(PlayConversationPairJson {
                    loop_key_node_id: a.id,
                    agent_a: PlayWorkerInPlayJson {
                        node_id: a.id,
                        name: a.name.clone(),
                        global_id: gid.clone(),
                        conversation_topic: a.topic.clone(),
                        conversation_topic_source: a.topic_source.clone(),
                    },
                    agent_b: PlayWorkerInPlayJson {
                        node_id: a.id,
                        name: a.name.clone(),
                        global_id: gid,
                        conversation_topic: a.topic.clone(),
                        conversation_topic_source: a.topic_source.clone(),
                    },
                    solo: true,
                });
                self.start_conversation_from_node_worker_resolved(
                    sidecars.clone(),
                    run_generation,
                    gen_counter.clone(),
                    loops_remaining.clone(),
                    graph_running_flag.clone(),
                    a.id,
                    a.id,
                    a.name.clone(),
                    a.instruction.clone(),
                    a.topic.clone(),
                    a.topic_source.clone(),
                    a.id,
                    a.name.clone(),
                    a.instruction.clone(),
                    a.topic.clone(),
                    a.topic_source.clone(),
                );
                i += 1;
            }
        }

        let play_plan =
            collect_run_play_plan_from_agents(&self.nodes_panel.agents, conversations_plan);
        match serde_json::to_string_pretty(&play_plan) {
            Ok(json) => println!("[Run Graph] play plan:\n{}", json),
            Err(e) => eprintln!("[Run Graph] failed to serialize play plan: {e}"),
        }

        self.conversation_graph_running
            .store(true, Ordering::Release);
        status_message
    }

    /// Keys the async loop by `loop_key_node_id` (first worker in each pair). Conversation output nodes were removed; pairing is automatic from eligible workers.
    fn start_conversation_from_node_worker_resolved(
        &mut self,
        sidecars: Arc<crate::agents::conversation_sidecars::ConversationSidecarConfig>,
        run_generation: u64,
        run_generation_counter: Arc<AtomicU64>,
        loops_remaining_in_run: Arc<AtomicUsize>,
        conversation_graph_running_flag: Arc<AtomicBool>,
        loop_key_node_id: usize,
        agent_a_node_id: usize,
        agent_a_name: String,
        agent_a_instruction: String,
        agent_a_topic: String,
        agent_a_topic_source: String,
        agent_b_id: usize,
        agent_b_name: String,
        agent_b_instruction: String,
        agent_b_topic: String,
        agent_b_topic_source: String,
    ) {
        let agent_a_id = agent_a_node_id;
        let active_flag = Arc::new(Mutex::new(true));
        let flag_clone = active_flag.clone();
        let endpoint = self.http_endpoint.clone();
        let ollama_host = self.ollama_host.clone();
        let last_msg = self.last_message_in_chat.clone();
        let message_events = self.conversation_message_events.clone();
        let selected_model = if self.selected_ollama_model.trim().is_empty() {
            None
        } else {
            Some(self.selected_ollama_model.clone())
        };
        let history_size = self.conversation_history_size;
        let handle = self.rt_handle.clone();
        let run_context = self.current_run_context.clone();
        let message_event_source_id = loop_key_node_id;
        let ollama_epoch = self.ollama_run_epoch.clone();
        let ollama_caught = self.ollama_run_epoch.load(Ordering::SeqCst);
        let ollama_stop_epoch = Some((ollama_epoch, ollama_caught));

        let agent_a_global_id = self
            .nodes_panel
            .agents
            .iter()
            .find(|r| r.id == agent_a_id)
            .and_then(|r| {
                if let NodePayload::Worker(w) = &r.data.payload {
                    Some(w.global_id.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let agent_b_global_id = self
            .nodes_panel
            .agents
            .iter()
            .find(|r| r.id == agent_b_id)
            .and_then(|r| {
                if let NodePayload::Worker(w) = &r.data.payload {
                    Some(w.global_id.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let ledger = self.event_ledger.clone();
        let app_state = self.app_state.clone();

        let loop_handle = handle.spawn(async move {
            crate::agents::agent_conversation_loop::start_conversation_loop(
                message_event_source_id,
                ollama_stop_epoch,
                sidecars,
                agent_a_id,
                agent_a_name,
                agent_a_instruction,
                agent_a_topic,
                agent_a_topic_source,
                agent_a_global_id,
                agent_b_id,
                agent_b_name,
                agent_b_instruction,
                agent_b_topic,
                agent_b_topic_source,
                agent_b_global_id,
                ollama_host,
                endpoint,
                flag_clone,
                last_msg,
                message_events,
                selected_model,
                history_size,
                run_context,
                run_generation,
                run_generation_counter,
                loops_remaining_in_run,
                conversation_graph_running_flag,
                ledger,
                app_state,
            )
            .await;
        });

        self.conversation_loop_handles
            .push((loop_key_node_id, active_flag, loop_handle));
    }
}
