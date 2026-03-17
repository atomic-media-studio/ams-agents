use eframe::egui;
use tokio::runtime::Handle;
use std::sync::{Arc, Mutex};
use crate::agent_entities::{Agent, AgentManager, Evaluator, Researcher};
use rand::Rng;

mod settings_panel;
mod outgoing_http_panel;
mod agent_manager_panel;
mod agent_worker_panel;
mod agent_evaluator_panel;
mod agent_researcher_panel;

pub struct AMSAgents {
    rt_handle: Handle,
    ollama_models: Arc<Mutex<Vec<String>>>,
    ollama_models_loading: Arc<Mutex<bool>>,
    selected_ollama_model: String,
    managers: Vec<AgentManager>,
    next_manager_id: usize,
    agents: Vec<Agent>,
    next_agent_id: usize,
    evaluators: Vec<Evaluator>,
    next_evaluator_id: usize,
    researchers: Vec<Researcher>,
    next_researcher_id: usize,
    selected_worker_id: Option<usize>,
    conversation_turn_delay_secs: u64,
    conversation_history_size: usize,
    http_endpoint: String,
    last_message_in_chat: Arc<Mutex<Option<String>>>,
    last_evaluated_message_by_evaluator: Arc<Mutex<std::collections::HashMap<usize, String>>>,
    last_researched_message_by_researcher: Arc<Mutex<std::collections::HashMap<usize, String>>>,
    conversation_loop_handles: Vec<(usize, Arc<Mutex<bool>>, tokio::task::JoinHandle<()>)>, // (agent_id, active_flag, handle)
    global_ids: std::collections::HashSet<String>,
}

impl AMSAgents {
    pub fn new(rt_handle: Handle) -> Self {
        Self { 
            rt_handle,
            ollama_models: Arc::new(Mutex::new(Vec::new())),
            ollama_models_loading: Arc::new(Mutex::new(false)),
            selected_ollama_model: std::env::var("OLLAMA_MODEL").unwrap_or_default(),
            managers: Vec::new(),
            next_manager_id: 1,
            agents: Vec::new(),
            next_agent_id: 1,
            evaluators: Vec::new(),
            next_evaluator_id: 1,
            researchers: Vec::new(),
            next_researcher_id: 1,
            selected_worker_id: None,
            conversation_turn_delay_secs: 3,
            conversation_history_size: 5,
            http_endpoint: std::env::var("CONVERSATION_HTTP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:3000/".to_string()),
            last_message_in_chat: Arc::new(Mutex::new(None)),
            last_evaluated_message_by_evaluator: Arc::new(Mutex::new(std::collections::HashMap::new())),
            last_researched_message_by_researcher: Arc::new(Mutex::new(std::collections::HashMap::new())),
            conversation_loop_handles: Vec::new(),
            global_ids: std::collections::HashSet::new(),
        }
    }

    fn generate_global_id(&mut self) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        const LEN: usize = 10;
        let mut rng = rand::rng();

        loop {
            let candidate: String = (0..LEN)
                .map(|_| {
                    let idx = rng.random_range(0..CHARSET.len());
                    CHARSET[idx] as char
                })
                .collect();
            if self.global_ids.insert(candidate.clone()) {
                return candidate;
            }
        }
    }
}

impl eframe::App for AMSAgents {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-refresh model list on startup
        if self.ollama_models.lock().unwrap().is_empty() && !*self.ollama_models_loading.lock().unwrap() {
            *self.ollama_models_loading.lock().unwrap() = true;
            let models_arc = self.ollama_models.clone();
            let loading_arc = self.ollama_models_loading.clone();
            let ctx = ctx.clone();
            let handle = self.rt_handle.clone();
            handle.spawn(async move {
                let models = crate::adk_integration::fetch_ollama_models().await.unwrap_or_default();
                *models_arc.lock().unwrap() = models;
                *loading_arc.lock().unwrap() = false;
                ctx.request_repaint();
            });
        }
        let any_evaluator_active = self.evaluators.iter().any(|e| e.active);
        let any_researcher_active = self.researchers.iter().any(|r| r.active);
        if any_evaluator_active || any_researcher_active {
            ctx.request_repaint();
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.set_min_height(ui.available_height());
                self.render_settings_panel(ui, ctx);
                
                ui.separator();
                let desired_outgoing_http_height = 160.0;
                let workspace_gap = 6.0;
                let workspace_height = (ui.available_height()
                    - desired_outgoing_http_height
                    - workspace_gap)
                    .max(0.0);
                let workspace_width = ui.available_width();

                ui.allocate_ui_with_layout(
                    egui::vec2(workspace_width, workspace_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_min_height(workspace_height);
                        ui.set_max_height(workspace_height);
                        // Scrollable area for agents with green border - full width
                        let available_width = ui.available_width() - 12.0;
                        egui::Frame::default()
                            .fill(egui::Color32::from_rgb(40, 40, 40))
                            //.stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 0)))
                            .inner_margin(egui::Margin { left: 6.0, right: 6.0, top: 6.0, bottom: 6.0 })
                            .rounding(4.0)
                            .show(ui, |ui| {
                                ui.set_min_width(available_width);
                                ui.set_max_width(available_width);
                                ui.set_min_height(workspace_height);
                                ui.set_max_height(workspace_height);
                                egui::ScrollArea::vertical()
                                    .id_source(ui.id().with("workspace_left_scroll"))
                                    .show(ui, |ui| {
                        
                // Collect IDs of managers/agents/evaluators/researchers to remove
                let mut managers_to_remove = Vec::new();
                let mut agents_to_remove = Vec::new();
                let mut evaluators_to_remove = Vec::new();
                let mut researchers_to_remove = Vec::new();
                
                // Collect agent info for partner dropdown (before mutable borrow)
                let agent_names: Vec<(usize, usize, String)> = self.agents.iter()
                    .map(|a| (a.id, a.manager_id, a.name.clone()))
                    .collect();
                let agent_name_by_id: std::collections::HashMap<usize, String> = agent_names
                    .iter()
                    .map(|(id, _, name)| (*id, name.clone()))
                    .collect();
                let mut chatting_partner_by_agent: std::collections::HashMap<usize, String> =
                    std::collections::HashMap::new();
                for app_agent in &self.agents {
                    if let Some(partner_id) = app_agent.conversation_partner_id {
                        if let Some(partner_name) = agent_name_by_id.get(&partner_id) {
                            chatting_partner_by_agent.insert(app_agent.id, partner_name.clone());
                        }
                        if let Some(agent_name) = agent_name_by_id.get(&app_agent.id) {
                            chatting_partner_by_agent.insert(partner_id, agent_name.clone());
                        }
                    }
                }
                let targeted_partner_ids: std::collections::HashSet<usize> = self.agents.iter()
                    .filter_map(|a| a.conversation_partner_id)
                    .collect();
                let targeted_partner_mode_by_agent: std::collections::HashMap<usize, String> = self.agents.iter()
                    .filter_map(|a| a.conversation_partner_id.map(|pid| (pid, a.conversation_mode.clone())))
                    .collect();
                
                // Collect full agent info for conversation setup (before mutable borrow)
                let agents_info: Vec<(usize, usize, String, String, String, String)> = self.agents.iter()
                    .map(|a| (
                        a.id,
                        a.manager_id,
                        a.name.clone(),
                        a.instruction.clone(),
                        a.conversation_topic.clone(),
                        a.conversation_topic_source.clone(),
                    ))
                    .collect();
                
                let panel_border_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
                let manager_bg_color = egui::Color32::from_rgb(40, 40, 40);
                let manager_frame = egui::Frame::default()
                    .fill(manager_bg_color)
                    .stroke(egui::Stroke::new(1.0, panel_border_color))
                    .rounding(4.0)
                    .inner_margin(egui::Margin::same(5.0))
                    .outer_margin(egui::Margin::same(0.0));

                ui.label(egui::RichText::new("Workspace").strong().size(12.0));
                ui.separator();

                if self.managers.is_empty() {
                    ui.label("No Agent Manager. Click \"Agent Manager\" above to create one.");
                }

                let managers_snapshot = self.managers.clone();
                for manager in &managers_snapshot {
                    manager_frame.show(ui, |ui| {
                        let manager_width = ui.available_width();
                        ui.set_width(manager_width);
                        ui.set_max_width(manager_width);
                        ui.vertical(|ui| {
                            AMSAgents::render_agent_manager_header(ui, &manager.name, &manager.global_id);
                            ui.separator();
                            let worker_count = self
                                .agents
                                .iter()
                                .filter(|a| a.manager_id == manager.id)
                                .count();
                            let evaluator_count = self
                                .evaluators
                                .iter()
                                .filter(|e| e.manager_id == manager.id)
                                .count();
                            let researcher_count = self
                                .researchers
                                .iter()
                                .filter(|r| r.manager_id == manager.id)
                                .count();
                            ui.horizontal_top(|ui| {
                                let left_width = (ui.available_width() * 0.5 - 6.0).max(320.0);
                                ui.vertical(|ui| {
                                        ui.set_min_width(left_width);
                                        ui.set_max_width(left_width);
                                        ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing = egui::Vec2::new(5.0, 2.0);
                                ui.horizontal(|ui| {
                                    if ui.button("Create Worker").clicked() {
                                        let used_ids: std::collections::HashSet<usize> =
                                            self.agents.iter().map(|a| a.id).collect();
                                        let mut new_id = 1;
                                        while used_ids.contains(&new_id) {
                                            new_id += 1;
                                        }
                                        let global_id = self.generate_global_id();
                                        self.agents.push(Agent::new(new_id, global_id, manager.id));
                                        if new_id >= self.next_agent_id {
                                            self.next_agent_id = new_id + 1;
                                        }
                                    }
                                    if ui.button("Erase Manager").clicked() {
                                        managers_to_remove.push(manager.id);
                                    }
                                });
                                ui.separator();

                                egui::CollapsingHeader::new(format!("Agent Workers ({})", worker_count))
                                    .id_source(ui.id().with(manager.id).with("workers_section"))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        let worker_options: Vec<(usize, String)> = self
                                            .agents
                                            .iter()
                                            .filter(|a| a.manager_id == manager.id)
                                            .map(|a| (a.id, format!("{} (M{})", a.name, a.manager_id)))
                                            .collect();

                                        if worker_options.is_empty() {
                                            self.selected_worker_id = None;
                                        } else if self.selected_worker_id.is_none()
                                            || worker_options
                                                .iter()
                                                .all(|(id, _)| Some(*id) != self.selected_worker_id)
                                        {
                                            self.selected_worker_id = Some(worker_options[0].0);
                                        }

                                        ui.horizontal(|ui| {
                                            if let Some(selected_id) = self.selected_worker_id {
                                                if let Some(selected_agent) = self.agents.iter_mut().find(|a| a.id == selected_id) {
                                                    ui.label("Input:");
                                                    ui.add(egui::TextEdit::singleline(&mut selected_agent.input).desired_width(180.0));
                                                    if ui.button("Send").clicked() {
                                                        let agent_clone = selected_agent.clone();
                                                        let endpoint = self.http_endpoint.clone();
                                                        let ctx = ctx.clone();
                                                        let handle = self.rt_handle.clone();
                                                        let last_msg = self.last_message_in_chat.clone();
                                                        let selected_model = if self.selected_ollama_model.trim().is_empty() {
                                                            None
                                                        } else {
                                                            Some(self.selected_ollama_model.clone())
                                                        };
                                                        handle.spawn(async move {
                                                            match crate::adk_integration::send_to_ollama(
                                                                &agent_clone.instruction,
                                                                &agent_clone.input,
                                                                agent_clone.limit_token,
                                                                &agent_clone.num_predict,
                                                                selected_model.as_deref(),
                                                            ).await {
                                                                Ok(response) => {
                                                                    *last_msg.lock().unwrap() = Some(response.clone());
                                                                    if agent_clone.in_conversation && agent_clone.conversation_partner_id.is_some() {
                                                                        if let Some(partner_id) = agent_clone.conversation_partner_id {
                                                                            let partner_name = format!("Agent {}", partner_id);
                                                                            if let Err(e) = crate::http_client::send_conversation_message(
                                                                                &endpoint,
                                                                                agent_clone.id,
                                                                                &agent_clone.name,
                                                                                partner_id,
                                                                                &partner_name,
                                                                                &agent_clone.conversation_topic,
                                                                                &response,
                                                                            ).await {
                                                                                eprintln!("Failed to send HTTP message: {}", e);
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    eprintln!("Ollama error: {}", e);
                                                                }
                                                            }
                                                            ctx.request_repaint();
                                                        });
                                                    }
                                                    if ui.checkbox(&mut selected_agent.limit_token, "Token").changed() {
                                                        if !selected_agent.limit_token {
                                                            selected_agent.num_predict.clear();
                                                        }
                                                    }
                                                    if selected_agent.limit_token {
                                                        ui.label("num_predict:");
                                                        ui.add(egui::TextEdit::singleline(&mut selected_agent.num_predict).desired_width(80.0));
                                                    }
                                                    ui.separator();
                                                }
                                            }
                                            ui.label("Worker:");
                                            let selected_text = if let Some(selected_id) = self.selected_worker_id {
                                                worker_options
                                                    .iter()
                                                    .find(|(id, _)| *id == selected_id)
                                                    .map(|(_, name)| name.clone())
                                                    .unwrap_or_else(|| "Select".to_string())
                                            } else {
                                                "Select".to_string()
                                            };
                                            egui::ComboBox::from_id_source(ui.id().with(manager.id).with("worker_selector"))
                                                .selected_text(selected_text)
                                                .show_ui(ui, |ui| {
                                                    for (id, name) in &worker_options {
                                                        ui.selectable_value(&mut self.selected_worker_id, Some(*id), name);
                                                    }
                                                });
                                        });

                                        ui.separator();
                                        ui.add_space(6.0);

                                        // Display child blocks inside this manager rectangle
                                        // Reserve space for the vertical scrollbar so stacked worker cards
                                        // don't overflow/spread when the list gets long.
                                        let worker_card_width = (ui.available_width() - 14.0).max(0.0);
                                        for agent in &mut self.agents {
                                    if agent.manager_id != manager.id {
                                        continue;
                                    }
                                    let agent_id = agent.id;
                                    let former_mode = targeted_partner_mode_by_agent.get(&agent_id).cloned();
                                    let is_selected_by_other_agent = former_mode.is_some();
                                    // Keep the second topic field visible even when this agent is selected
                                    // by another agent in Shared mode, so each agent can keep its own topic text.
                                    let show_topic_when_selected = former_mode.is_some();
                                    
                                    let bg_color = if agent.selected {
                                        egui::Color32::from_rgb(50, 50, 50)
                                    } else {
                                        egui::Color32::from_rgb(45, 45, 45)
                                    };
                                    
                                    let frame = egui::Frame::default()
                                        .fill(bg_color)
                                        .stroke(egui::Stroke::new(1.0, panel_border_color))
                                        .rounding(4.0)
                                        .inner_margin(egui::Margin::same(5.0))
                                        .outer_margin(egui::Margin { left: 0.0, right: 0.0, top: 0.0, bottom: 0.0 });
                                    
                                    ui.set_min_width(worker_card_width);
                                    ui.set_max_width(worker_card_width);
                                    let _frame_response = frame.show(ui, |ui| {
                                        ui.set_min_width(worker_card_width);
                                        ui.set_max_width(worker_card_width);
                                        ui.vertical(|ui| {
                                                AMSAgents::render_agent_worker_header(ui, &manager.name, &agent.global_id);
                                                ui.separator();
                                                ui.vertical(|ui| {
                                                    ui.horizontal(|ui| {
                                                        let separator_width = 8.0;
                                                        let total_width = ui.available_width().max(0.0);
                                                        let column_width = ((total_width - separator_width).max(0.0)) / 2.0;

                                                        ui.vertical(|ui| {
                                                            ui.set_min_width(column_width);
                                                            ui.set_max_width(column_width);
                                                            ui.spacing_mut().item_spacing = egui::Vec2::new(5.0, 2.0);

                                                            ui.horizontal(|ui| {
                                                                ui.label("Name:");
                                                                ui.add(egui::TextEdit::singleline(&mut agent.name));
                                                            });

                                                            ui.horizontal(|ui| {
                                                                ui.label("Instruction:");
                                                                egui::ComboBox::from_id_source(ui.id().with(agent_id).with("instruction_mode"))
                                                                    .selected_text(if agent.instruction_mode.is_empty() {
                                                                        "Select".to_string()
                                                                    } else {
                                                                        agent.instruction_mode.clone()
                                                                    })
                                                                    .show_ui(ui, |ui| {
                                                                        if ui.selectable_label(agent.instruction_mode == "Assistant", "Assistant").clicked() {
                                                                            agent.instruction_mode = "Assistant".to_string();
                                                                            agent.instruction = "You are a helpful assistant. Answer clearly, stay concise, and focus on the user request.".to_string();
                                                                            println!("Agent {} instruction selected: Assistant", agent.id);
                                                                        }
                                                                        if ui.selectable_label(agent.instruction_mode == "Math Teacher", "Math Teacher").clicked() {
                                                                            agent.instruction_mode = "Math Teacher".to_string();
                                                                        }
                                                                        if ui.selectable_label(agent.instruction_mode == "Debate", "Debate").clicked() {
                                                                            agent.instruction_mode = "Debate".to_string();
                                                                        }
                                                                    });
                                                            });
                                                            
                                                            ui.horizontal(|ui| {
                                                                ui.label("Instruction:");
                                                                ui.add(egui::TextEdit::singleline(&mut agent.instruction));
                                                            });
                                                        });

                                                        ui.separator();

                                                        ui.vertical(|ui| {
                                                            ui.set_min_width(column_width);
                                                            ui.set_max_width(column_width);
                                                            ui.spacing_mut().item_spacing = egui::Vec2::new(5.0, 2.0);

                                                            if !is_selected_by_other_agent || show_topic_when_selected {
                                                                ui.horizontal(|ui| {
                                                                    ui.label("Topic:");
                                                                    egui::ComboBox::from_id_source(ui.id().with(agent_id).with("analysis_mode"))
                                                                        .selected_text(if agent.analysis_mode.is_empty() {
                                                                            "Select".to_string()
                                                                        } else {
                                                                            agent.analysis_mode.clone()
                                                                        })
                                                                        .show_ui(ui, |ui| {
                                                                            if ui.selectable_label(agent.analysis_mode == "European Politics", "European Politics").clicked() {
                                                                                agent.analysis_mode = "European Politics".to_string();
                                                                                agent.conversation_topic = "Discuss European Politics and provide a concise overview of the main issue in one or two sentences.".to_string();
                                                                                println!("Agent {} topic selected: European Politics", agent.id);
                                                                            }
                                                                            if ui.selectable_label(agent.analysis_mode == "Mental Health", "Mental Health").clicked() {
                                                                                agent.analysis_mode = "Mental Health".to_string();
                                                                                agent.conversation_topic = "Discuss Mental Health and provide one or two practical insights about the topic.".to_string();
                                                                                println!("Agent {} topic selected: Mental Health", agent.id);
                                                                            }
                                                                            if ui.selectable_label(agent.analysis_mode == "Electronics", "Electronics").clicked() {
                                                                                agent.analysis_mode = "Electronics".to_string();
                                                                                agent.conversation_topic = "Discuss Electronics and summarize one or two important points about the selected subject.".to_string();
                                                                                println!("Agent {} topic selected: Electronics", agent.id);
                                                                            }
                                                                        });
                                                                });
                                                            }

                                                            if !is_selected_by_other_agent || show_topic_when_selected {
                                                                ui.horizontal(|ui| {
                                                                    ui.label("Topic:");
                                                                    ui.add(egui::TextEdit::singleline(&mut agent.conversation_topic));
                                                                });
                                                                ui.horizontal(|ui| {
                                                                    ui.label("Topic Source:");
                                                                    egui::ComboBox::from_id_source(ui.id().with(agent_id).with("topic_source"))
                                                                        .width(100.0)
                                                                        .selected_text(agent.conversation_topic_source.clone())
                                                                        .show_ui(ui, |ui| {
                                                                            ui.selectable_value(
                                                                                &mut agent.conversation_topic_source,
                                                                                "Own".to_string(),
                                                                                "Own",
                                                                            );
                                                                            ui.selectable_value(
                                                                                &mut agent.conversation_topic_source,
                                                                                "Follow Partner".to_string(),
                                                                                "Follow Partner",
                                                                            );
                                                                        });
                                                                });
                                                            }

                                                            if !is_selected_by_other_agent {
                                                                ui.horizontal(|ui| {
                                                                    ui.label("Mode:");
                                                                    egui::ComboBox::from_id_source(ui.id().with(agent_id).with("conversation_mode"))
                                                                        .width(100.0)
                                                                        .selected_text(agent.conversation_mode.clone())
                                                                        .show_ui(ui, |ui| {
                                                                            if ui.selectable_label(agent.conversation_mode == "Shared", "Shared").clicked() {
                                                                                agent.conversation_mode = "Shared".to_string();
                                                                            }
                                                                            if ui.selectable_label(agent.conversation_mode == "Unique", "Unique").clicked() {
                                                                                agent.conversation_mode = "Unique".to_string();
                                                                                agent.conversation_partner_id = None;
                                                                            }
                                                                        });
                                                                });
                                                                if agent.conversation_mode == "Shared" && !targeted_partner_ids.contains(&agent_id) {
                                                                    ui.horizontal(|ui| {
                                                                        ui.label("With:");
                                                                        let selected_text = if let Some(pid) = agent.conversation_partner_id {
                                                                            format!("Agent {}", pid)
                                                                        } else {
                                                                            "None".to_string()
                                                                        };
                                                                        
                                                                        egui::ComboBox::from_id_source(ui.id().with(agent_id).with("partner"))
                                                                            .width(100.0)
                                                                            .selected_text(selected_text)
                                                                            .show_ui(ui, |ui| {
                                                                                ui.selectable_value(&mut agent.conversation_partner_id, None, "None");
                                                                                for (other_id, other_manager_id, other_name) in &agent_names {
                                                                                    if *other_id != agent_id && *other_manager_id == agent.manager_id {
                                                                                        ui.selectable_value(
                                                                                            &mut agent.conversation_partner_id,
                                                                                            Some(*other_id),
                                                                                            other_name,
                                                                                        );
                                                                                    }
                                                                                }
                                                                            });
                                                                    });
                                                                }
                                                            }

                                                            if let Some(partner_name) =
                                                                chatting_partner_by_agent.get(&agent_id)
                                                            {
                                                                ui.horizontal(|ui| {
                                                                    ui.label(format!(
                                                                        "Chatting with {}",
                                                                        partner_name
                                                                    ));
                                                                });
                                                            }
                                                        });
                                                    });

                                                    let button_text = if agent.conversation_active {
                                                        "Stop Conversation"
                                                    } else {
                                                        "Start Conversation"
                                                    };
                                                    let button = egui::Button::new(button_text);

                                                    if !is_selected_by_other_agent {
                                                        ui.separator();
                                                        ui.horizontal(|ui| {
                                                            if ui.add(button).clicked() {
                                                                        if agent.conversation_active {
                                                                            agent.conversation_active = false;
                                                                            agent.in_conversation = false;
                                                                            self.conversation_loop_handles.retain(|(aid, flag, _)| {
                                                                                if *aid == agent_id {
                                                                                    *flag.lock().unwrap() = false;
                                                                                    false
                                                                                } else {
                                                                                    true
                                                                                }
                                                                            });
                                                                        } else {
                                                                            if !agent.conversation_topic.is_empty() {
                                                                                let maybe_partner = if agent.conversation_mode == "Unique" {
                                                                                    Some((
                                                                                        agent.id,
                                                                                        agent.name.clone(),
                                                                                        agent.instruction.clone(),
                                                                                        agent.conversation_topic.clone(),
                                                                                        agent.conversation_topic_source.clone(),
                                                                                    ))
                                                                                } else if let Some(partner_id) = agent.conversation_partner_id {
                                                                                    agents_info
                                                                                        .iter()
                                                                                        .find(|(id, _, _, _, _, _)| *id == partner_id)
                                                                                        .map(|(_, _, partner_name, partner_instruction, partner_topic, partner_topic_source)| {
                                                                                            (
                                                                                                partner_id,
                                                                                                partner_name.clone(),
                                                                                                partner_instruction.clone(),
                                                                                                partner_topic.clone(),
                                                                                                partner_topic_source.clone(),
                                                                                            )
                                                                                        })
                                                                                } else {
                                                                                    None
                                                                                };

                                                                                if let Some((partner_id, partner_name, partner_instruction, partner_topic, partner_topic_source)) = maybe_partner {
                                                                                        agent.conversation_active = true;
                                                                                        agent.in_conversation = true;
                                                                                        let active_flag = Arc::new(Mutex::new(true));
                                                                                        let flag_clone = active_flag.clone();
                                                                                        let endpoint = self.http_endpoint.clone();
                                                                                        let handle = self.rt_handle.clone();
                                                                                        let agent_a_id = agent.id;
                                                                                        let agent_a_name = agent.name.clone();
                                                                                        let agent_a_instruction = agent.instruction.clone();
                                                                                        let agent_a_topic = agent.conversation_topic.clone();
                                                                                        let agent_a_topic_source = agent.conversation_topic_source.clone();
                                                                                        let agent_b_id = partner_id;
                                                                                        let agent_b_name = partner_name;
                                                                                        let agent_b_instruction = partner_instruction;
                                                                                        let agent_b_topic = partner_topic;
                                                                                        let agent_b_topic_source = partner_topic_source;
                                                                                        let last_msg = self.last_message_in_chat.clone();
                                                                                        let selected_model = if self.selected_ollama_model.trim().is_empty() {
                                                                                            None
                                                                                        } else {
                                                                                            Some(self.selected_ollama_model.clone())
                                                                                        };
                                                                                        let history_size = self.conversation_history_size;
                                                                                        let turn_delay_secs = self.conversation_turn_delay_secs;
                                                                                        let loop_handle = handle.spawn(async move {
                                                                                            crate::agent_conversation_loop::start_conversation_loop(
                                                                                                agent_a_id,
                                                                                                agent_a_name,
                                                                                                agent_a_instruction,
                                                                                                agent_a_topic,
                                                                                                agent_a_topic_source,
                                                                                                agent_b_id,
                                                                                                agent_b_name,
                                                                                                agent_b_instruction,
                                                                                                agent_b_topic,
                                                                                                agent_b_topic_source,
                                                                                                endpoint,
                                                                                                flag_clone,
                                                                                                last_msg,
                                                                                                selected_model,
                                                                                                history_size,
                                                                                                turn_delay_secs,
                                                                                            ).await;
                                                                                        });
                                                                                        self.conversation_loop_handles.push((agent_id, active_flag, loop_handle));
                                                                                } else {
                                                                                    println!("Cannot start conversation: need partner");
                                                                                }
                                                                            } else {
                                                                                println!("Cannot start conversation: need topic");
                                                                            }
                                                                        }
                                                                    }
                                                            if ui.button("Status").clicked() {
                                                                println!("=== Agent {} Status ===", agent.id);
                                                                println!("Global ID: {}", agent.global_id);
                                                            }
                                                            if ui.button("Erase").clicked() {
                                                                agents_to_remove.push(agent_id);
                                                            }
                                                        });
                                                    }
                                                });
                                            });
                                        });
                                    ui.add_space(6.0);
                                }
                                    });

                                // Evaluator/Researcher panels moved to right-side split pane.
                                        });
                                });
                                ui.separator();
                                ui.vertical(|ui| {
                                        ui.vertical(|ui| {
                                            ui.spacing_mut().item_spacing = egui::Vec2::new(5.0, 2.0);
                                            ui.horizontal(|ui| {
                                                if ui.button("Create Evaluator").clicked() {
                                                    let used_ids: std::collections::HashSet<usize> =
                                                        self.evaluators.iter().map(|e| e.id).collect();
                                                    let mut new_id = 1;
                                                    while used_ids.contains(&new_id) {
                                                        new_id += 1;
                                                    }
                                                    let global_id = self.generate_global_id();
                                                    self.evaluators.push(Evaluator::new(new_id, global_id, manager.id));
                                                    if new_id >= self.next_evaluator_id {
                                                        self.next_evaluator_id = new_id + 1;
                                                    }
                                                }
                                                if ui.button("Create Researcher").clicked() {
                                                    let used_ids: std::collections::HashSet<usize> =
                                                        self.researchers.iter().map(|r| r.id).collect();
                                                    let mut new_id = 1;
                                                    while used_ids.contains(&new_id) {
                                                        new_id += 1;
                                                    }
                                                    let global_id = self.generate_global_id();
                                                    self.researchers.push(Researcher::new(new_id, global_id, manager.id));
                                                    if new_id >= self.next_researcher_id {
                                                        self.next_researcher_id = new_id + 1;
                                                    }
                                                }
                                            });
                                            ui.separator();

                                            egui::CollapsingHeader::new(format!("Agent Evaluators ({})", evaluator_count))
                                                .id_source(ui.id().with(manager.id).with("evaluators_section"))
                                                .default_open(true)
                                                .show(ui, |ui| {
                                                    for evaluator in &mut self.evaluators {
                                                        if evaluator.manager_id != manager.id {
                                                            continue;
                                                        }
                                                        let eval_id = evaluator.id;
                                                        let last_msg = self.last_message_in_chat.lock().unwrap().clone();
                                                        let last_eval = self.last_evaluated_message_by_evaluator.lock().unwrap().get(&eval_id).cloned();
                                                        let should_run = evaluator.active
                                                            && last_msg.as_ref().map_or(false, |s| !s.is_empty())
                                                            && last_eval.as_ref() != last_msg.as_ref();
                                                        if should_run {
                                                            let message = last_msg.clone().unwrap_or_default();
                                                            println!("[Evaluator] Analyzing last message ({} chars), sending to Ollama...", message.len());
                                                            self.last_evaluated_message_by_evaluator.lock().unwrap().insert(eval_id, message.clone());
                                                            let eval_clone = evaluator.clone();
                                                            let endpoint = self.http_endpoint.clone();
                                                            let ctx = ctx.clone();
                                                            let handle = self.rt_handle.clone();
                                                            let selected_model = if self.selected_ollama_model.trim().is_empty() {
                                                                None
                                                            } else {
                                                                Some(self.selected_ollama_model.clone())
                                                            };
                                                            handle.spawn(async move {
                                                                match crate::adk_integration::send_to_ollama(
                                                                    &eval_clone.instruction,
                                                                    &message,
                                                                    eval_clone.limit_token,
                                                                    &eval_clone.num_predict,
                                                                    selected_model.as_deref(),
                                                                ).await {
                                                                    Ok(response) => {
                                                                        let response_lower = response.to_lowercase();
                                                                        let sentiment = match eval_clone.analysis_mode.as_str() {
                                                                            "Topic Extraction" => "topic",
                                                                            "Decision Analysis" => "decision",
                                                                            "Sentiment Classification" => {
                                                                                if response_lower.contains("positive") || response_lower.contains("happy") {
                                                                                    "sentiment"
                                                                                } else if response_lower.contains("negative")
                                                                                    || response_lower.contains("sad")
                                                                                    || response_lower.contains("angry")
                                                                                    || response_lower.contains("frustrated")
                                                                                {
                                                                                    "sentiment"
                                                                                } else if response_lower.contains("neutral") {
                                                                                    "sentiment"
                                                                                } else {
                                                                                    "unknown"
                                                                                }
                                                                            }
                                                                            _ => {
                                                                                if response_lower.contains("happy") {
                                                                                    "happy"
                                                                                } else if response_lower.contains("sad") {
                                                                                    "sad"
                                                                                } else {
                                                                                    "analysis"
                                                                                }
                                                                            }
                                                                        };
                                                                        if let Err(e) = crate::http_client::send_evaluator_result(
                                                                            &endpoint,
                                                                            "Agent Evaluator",
                                                                            sentiment,
                                                                            &response,
                                                                        ).await {
                                                                            eprintln!("[Evaluator] Failed to send to ams-chat: {}", e);
                                                                        } else {
                                                                            println!("[Evaluator] Sent to ams-chat: {} -> {}", sentiment, &response[..response.len().min(60)]);
                                                                        }
                                                                    }
                                                                    Err(e) => eprintln!("Ollama error: {}", e),
                                                                }
                                                                ctx.request_repaint();
                                                            });
                                                        }

                                                        let bg_color = egui::Color32::from_rgb(45, 45, 45);
                                                        let frame = egui::Frame::default()
                                                            .fill(bg_color)
                                                            .stroke(egui::Stroke::new(1.0, panel_border_color))
                                                            .rounding(4.0)
                                                            .inner_margin(egui::Margin::same(5.0))
                                                            .outer_margin(egui::Margin { left: 0.0, right: 0.0, top: 0.0, bottom: 0.0 });
                                                        ui.horizontal(|ui| {
                                                            ui.set_max_width(ui.available_width());
                                                            frame.show(ui, |ui| {
                                                                ui.vertical(|ui| {
                                                                    AMSAgents::render_agent_evaluator_header(
                                                                        ui,
                                                                        &manager.name,
                                                                        &evaluator.global_id,
                                                                    );
                                                                    ui.separator();
                                                                    ui.vertical(|ui| {
                                                                        ui.spacing_mut().item_spacing = egui::Vec2::new(5.0, 2.0);
                                                                        ui.horizontal(|ui| {
                                                                            ui.label("Name:");
                                                                            ui.add(egui::TextEdit::singleline(&mut evaluator.name));
                                                                        });
                                                                        ui.horizontal(|ui| {
                                                                            ui.label("Analysis:");
                                                                            egui::ComboBox::from_id_source(ui.id().with(eval_id).with("eval_analysis_mode"))
                                                                                .selected_text(if evaluator.analysis_mode.is_empty() {
                                                                                    "Select".to_string()
                                                                                } else {
                                                                                    evaluator.analysis_mode.clone()
                                                                                })
                                                                                .show_ui(ui, |ui| {
                                                                                    if ui.selectable_label(evaluator.analysis_mode == "Topic Extraction", "Topic Extraction").clicked() {
                                                                                        evaluator.analysis_mode = "Topic Extraction".to_string();
                                                                                        evaluator.instruction = "Topic Extraction: extract the topic in 1 or 2 words. Identify what is the topic of the sentence being analysed.".to_string();
                                                                                        println!("Evaluator {} analysis selected: Topic Extraction", evaluator.id);
                                                                                    }
                                                                                    if ui.selectable_label(evaluator.analysis_mode == "Decision Analysis", "Decision Analysis").clicked() {
                                                                                        evaluator.analysis_mode = "Decision Analysis".to_string();
                                                                                        evaluator.instruction = "Decision Analysis: extract a decision in 1 or 2 sentences about the agent in the message being analysed. Focus on the concrete decision and its intent.".to_string();
                                                                                        println!("Evaluator {} analysis selected: Decision Analysis", evaluator.id);
                                                                                    }
                                                                                    if ui.selectable_label(evaluator.analysis_mode == "Sentiment Classification", "Sentiment Classification").clicked() {
                                                                                        evaluator.analysis_mode = "Sentiment Classification".to_string();
                                                                                        evaluator.instruction = "Sentiment Classification: extract the sentiment of the message being analysed and return one word that is the sentiment.".to_string();
                                                                                        println!("Evaluator {} analysis selected: Sentiment Classification", evaluator.id);
                                                                                    }
                                                                                });
                                                                        });
                                                                        ui.horizontal(|ui| {
                                                                            ui.label("Instruction:");
                                                                            ui.add(egui::TextEdit::singleline(&mut evaluator.instruction));
                                                                        });
                                                                        ui.horizontal(|ui| {
                                                                            if ui.checkbox(&mut evaluator.limit_token, "Limit Token").changed() {
                                                                                if !evaluator.limit_token {
                                                                                    evaluator.num_predict.clear();
                                                                                }
                                                                            }
                                                                            if evaluator.limit_token {
                                                                                ui.label("num_predict:");
                                                                                ui.add(egui::TextEdit::singleline(&mut evaluator.num_predict).desired_width(80.0));
                                                                            }
                                                                        });
                                                                        ui.separator();
                                                                        let button_text = if evaluator.active { "Stop Evaluating" } else { "Evaluate" };
                                                                        let button = egui::Button::new(button_text); //.min_size(egui::Vec2::new(140.0, 20.0));
                                                                        ui.horizontal(|ui| {
                                                                            if ui.add(button).clicked() {
                                                                                evaluator.active = !evaluator.active;
                                                                                if evaluator.active {
                                                                                    println!("[Evaluator] ON");
                                                                                } else {
                                                                                    println!("[Evaluator] OFF");
                                                                                }
                                                                                if evaluator.active {
                                                                                    if let Some(message) = last_msg.clone() {
                                                                                        if last_eval.as_ref() != Some(&message) {
                                                                                            println!("[Evaluator] Manual trigger for last message ({} chars)", message.len());
                                                                                            self.last_evaluated_message_by_evaluator.lock().unwrap().insert(eval_id, message.clone());
                                                                                            let eval_clone = evaluator.clone();
                                                                                            let endpoint = self.http_endpoint.clone();
                                                                                            let ctx = ctx.clone();
                                                                                            let handle = self.rt_handle.clone();
                                                                                            let selected_model = if self.selected_ollama_model.trim().is_empty() {
                                                                                                None
                                                                                            } else {
                                                                                                Some(self.selected_ollama_model.clone())
                                                                                            };
                                                                                            handle.spawn(async move {
                                                                                                match crate::adk_integration::send_to_ollama(
                                                                                                    &eval_clone.instruction,
                                                                                                    &message,
                                                                                                    eval_clone.limit_token,
                                                                                                    &eval_clone.num_predict,
                                                                                                    selected_model.as_deref(),
                                                                                                ).await {
                                                                                                    Ok(response) => {
                                                                                                        let response_lower = response.to_lowercase();
                                                                                                        let sentiment = match eval_clone.analysis_mode.as_str() {
                                                                                                            "Topic Extraction" => "topic",
                                                                                                            "Decision Analysis" => "decision",
                                                                                                            "Sentiment Classification" => {
                                                                                                                if response_lower.contains("positive") || response_lower.contains("happy") {
                                                                                                                    "sentiment"
                                                                                                                } else if response_lower.contains("negative")
                                                                                                                    || response_lower.contains("sad")
                                                                                                                    || response_lower.contains("angry")
                                                                                                                    || response_lower.contains("frustrated")
                                                                                                                {
                                                                                                                    "sentiment"
                                                                                                                } else if response_lower.contains("neutral") {
                                                                                                                    "sentiment"
                                                                                                                } else {
                                                                                                                    "unknown"
                                                                                                                }
                                                                                                            }
                                                                                                            _ => {
                                                                                                                if response_lower.contains("happy") {
                                                                                                                    "happy"
                                                                                                                } else if response_lower.contains("sad") {
                                                                                                                    "sad"
                                                                                                                } else {
                                                                                                                    "analysis"
                                                                                                                }
                                                                                                            }
                                                                                                        };
                                                                                                        if let Err(e) = crate::http_client::send_evaluator_result(
                                                                                                            &endpoint,
                                                                                                            "Agent Evaluator",
                                                                                                            sentiment,
                                                                                                            &response,
                                                                                                        ).await {
                                                                                                            eprintln!("[Evaluator] Failed to send to ams-chat: {}", e);
                                                                                                        } else {
                                                                                                            println!("[Evaluator] Sent to ams-chat: {} -> {}", sentiment, &response[..response.len().min(60)]);
                                                                                                        }
                                                                                                    }
                                                                                                    Err(e) => eprintln!("[Evaluator] Ollama error: {}", e),
                                                                                                }
                                                                                                ctx.request_repaint();
                                                                                            });
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                            if ui.button("Status").clicked() {
                                                                                println!("=== Evaluator {} Status ===", evaluator.id);
                                                                                println!("Global ID: {}", evaluator.global_id);
                                                                                println!("Manager: {}", evaluator.manager_id);
                                                                                println!("Name: {}", evaluator.name);
                                                                                println!("Instruction: {}", evaluator.instruction);
                                                                                println!("========================");
                                                                            }
                                                                            if ui.button("Erase").clicked() {
                                                                                evaluators_to_remove.push(eval_id);
                                                                            }
                                                                        });
                                                                    });
                                                                });
                                                            });
                                                        });
                                                        ui.add_space(6.0);
                                                    }
                                                });

                                            egui::CollapsingHeader::new(format!("Agent Researchers ({})", researcher_count))
                                                .id_source(ui.id().with(manager.id).with("researchers_section"))
                                                .default_open(true)
                                                .show(ui, |ui| {
                                                    for researcher in &mut self.researchers {
                                                        if researcher.manager_id != manager.id {
                                                            continue;
                                                        }
                                                        let researcher_id = researcher.id;
                                                        let last_msg = self.last_message_in_chat.lock().unwrap().clone();
                                                        let last_research = self.last_researched_message_by_researcher.lock().unwrap().get(&researcher_id).cloned();
                                                        let should_run = researcher.active
                                                            && last_msg.as_ref().map_or(false, |s| !s.is_empty())
                                                            && last_research.as_ref() != last_msg.as_ref();
                                                        if should_run {
                                                            let message = last_msg.clone().unwrap_or_default();
                                                            println!("[Researcher] Generating references from last message ({} chars)...", message.len());
                                                            self.last_researched_message_by_researcher.lock().unwrap().insert(researcher_id, message.clone());
                                                            let researcher_clone = researcher.clone();
                                                            let endpoint = self.http_endpoint.clone();
                                                            let ctx = ctx.clone();
                                                            let handle = self.rt_handle.clone();
                                                            let selected_model = if self.selected_ollama_model.trim().is_empty() {
                                                                None
                                                            } else {
                                                                Some(self.selected_ollama_model.clone())
                                                            };
                                                            handle.spawn(async move {
                                                                let topic = if researcher_clone.topic_mode.trim().is_empty() {
                                                                    "Articles".to_string()
                                                                } else {
                                                                    researcher_clone.topic_mode.clone()
                                                                };
                                                                let generation_instruction = format!(
                                                                    "{}\n\nUsing the latest chat message, suggest 3 {} references related to what was said. Keep it concise with bullet points: title and one-line why it matches.",
                                                                    researcher_clone.instruction,
                                                                    topic.to_lowercase()
                                                                );
                                                                match crate::adk_integration::send_to_ollama(
                                                                    &generation_instruction,
                                                                    &message,
                                                                    researcher_clone.limit_token,
                                                                    &researcher_clone.num_predict,
                                                                    selected_model.as_deref(),
                                                                ).await {
                                                                    Ok(response) => {
                                                                        if let Err(e) = crate::http_client::send_researcher_result(
                                                                            &endpoint,
                                                                            "Agent Researcher",
                                                                            &topic,
                                                                            &response,
                                                                        ).await {
                                                                            eprintln!("[Researcher] Failed to send to ams-chat: {}", e);
                                                                        } else {
                                                                            println!("[Researcher] Sent references to ams-chat [{}]", topic);
                                                                        }
                                                                    }
                                                                    Err(e) => eprintln!("[Researcher] Ollama error: {}", e),
                                                                }
                                                                ctx.request_repaint();
                                                            });
                                                        }

                                                        let bg_color = egui::Color32::from_rgb(45, 45, 45);
                                                        let frame = egui::Frame::default()
                                                            .fill(bg_color)
                                                            .stroke(egui::Stroke::new(1.0, panel_border_color))
                                                            .rounding(4.0)
                                                            .inner_margin(egui::Margin::same(5.0))
                                                            .outer_margin(egui::Margin { left: 0.0, right: 0.0, top: 0.0, bottom: 0.0 });
                                                        ui.horizontal(|ui| {
                                                            ui.set_max_width(ui.available_width());
                                                            frame.show(ui, |ui| {
                                                                ui.vertical(|ui| {
                                                                    AMSAgents::render_agent_researcher_header(
                                                                        ui,
                                                                        &manager.name,
                                                                        &researcher.global_id,
                                                                    );
                                                                    ui.separator();
                                                                    ui.vertical(|ui| {
                                                                        ui.spacing_mut().item_spacing = egui::Vec2::new(5.0, 2.0);
                                                                        ui.horizontal(|ui| {
                                                                            ui.label("Name:");
                                                                            ui.add(egui::TextEdit::singleline(&mut researcher.name));
                                                                        });
                                                                        ui.horizontal(|ui| {
                                                                            ui.label("Topics:");
                                                                            egui::ComboBox::from_id_source(ui.id().with(researcher_id).with("research_topic_mode"))
                                                                                .selected_text(if researcher.topic_mode.is_empty() {
                                                                                    "Select".to_string()
                                                                                } else {
                                                                                    researcher.topic_mode.clone()
                                                                                })
                                                                                .show_ui(ui, |ui| {
                                                                                    if ui.selectable_label(researcher.topic_mode == "Articles", "Articles").clicked() {
                                                                                        researcher.topic_mode = "Articles".to_string();
                                                                                        researcher.instruction = "Generate article references connected to the message context. Prefer a mix of classic and recent pieces.".to_string();
                                                                                        println!("Researcher {} topic selected: Articles", researcher.id);
                                                                                    }
                                                                                    if ui.selectable_label(researcher.topic_mode == "Movies", "Movies").clicked() {
                                                                                        researcher.topic_mode = "Movies".to_string();
                                                                                        researcher.instruction = "Generate movie references connected to the message context. Prefer diverse genres and well-known titles.".to_string();
                                                                                        println!("Researcher {} topic selected: Movies", researcher.id);
                                                                                    }
                                                                                    if ui.selectable_label(researcher.topic_mode == "Music", "Music").clicked() {
                                                                                        researcher.topic_mode = "Music".to_string();
                                                                                        researcher.instruction = "Generate music references connected to the message context. Include artist and track or album when possible.".to_string();
                                                                                        println!("Researcher {} topic selected: Music", researcher.id);
                                                                                    }
                                                                                });
                                                                        });
                                                                        ui.horizontal(|ui| {
                                                                            ui.label("Instruction:");
                                                                            ui.add(egui::TextEdit::singleline(&mut researcher.instruction));
                                                                        });
                                                                        ui.horizontal(|ui| {
                                                                            if ui.checkbox(&mut researcher.limit_token, "Token").changed() {
                                                                                if !researcher.limit_token {
                                                                                    researcher.num_predict.clear();
                                                                                }
                                                                            }
                                                                            if researcher.limit_token {
                                                                                ui.label("num_predict:");
                                                                                ui.add(egui::TextEdit::singleline(&mut researcher.num_predict).desired_width(80.0));
                                                                            }
                                                                        });
                                                                        ui.separator();
                                                                        let button_text = if researcher.active { "Stop Researching" } else { "Research" };
                                                                        let button = egui::Button::new(button_text);//.min_size(egui::Vec2::new(140.0, 20.0));
                                                                        ui.horizontal(|ui| {
                                                                            if ui.add(button).clicked() {
                                                                                researcher.active = !researcher.active;
                                                                                if researcher.active {
                                                                                    println!("[Researcher] ON");
                                                                                } else {
                                                                                    println!("[Researcher] OFF");
                                                                                }
                                                                                if researcher.active {
                                                                                    if let Some(message) = last_msg.clone() {
                                                                                        if last_research.as_ref() != Some(&message) {
                                                                                            println!("[Researcher] Manual trigger for last message ({} chars)", message.len());
                                                                                            self.last_researched_message_by_researcher.lock().unwrap().insert(researcher_id, message.clone());
                                                                                            let researcher_clone = researcher.clone();
                                                                                            let endpoint = self.http_endpoint.clone();
                                                                                            let ctx = ctx.clone();
                                                                                            let handle = self.rt_handle.clone();
                                                                                            let selected_model = if self.selected_ollama_model.trim().is_empty() {
                                                                                                None
                                                                                            } else {
                                                                                                Some(self.selected_ollama_model.clone())
                                                                                            };
                                                                                            handle.spawn(async move {
                                                                                                let topic = if researcher_clone.topic_mode.trim().is_empty() {
                                                                                                    "Articles".to_string()
                                                                                                } else {
                                                                                                    researcher_clone.topic_mode.clone()
                                                                                                };
                                                                                                let generation_instruction = format!(
                                                                                                    "{}\n\nUsing the latest chat message, suggest 3 {} references related to what was said. Keep it concise with bullet points: title and one-line why it matches.",
                                                                                                    researcher_clone.instruction,
                                                                                                    topic.to_lowercase()
                                                                                                );
                                                                                                match crate::adk_integration::send_to_ollama(
                                                                                                    &generation_instruction,
                                                                                                    &message,
                                                                                                    researcher_clone.limit_token,
                                                                                                    &researcher_clone.num_predict,
                                                                                                    selected_model.as_deref(),
                                                                                                ).await {
                                                                                                    Ok(response) => {
                                                                                                        if let Err(e) = crate::http_client::send_researcher_result(
                                                                                                            &endpoint,
                                                                                                            "Agent Researcher",
                                                                                                            &topic,
                                                                                                            &response,
                                                                                                        ).await {
                                                                                                            eprintln!("[Researcher] Failed to send to ams-chat: {}", e);
                                                                                                        } else {
                                                                                                            println!("[Researcher] Sent references to ams-chat [{}]", topic);
                                                                                                        }
                                                                                                    }
                                                                                                    Err(e) => eprintln!("[Researcher] Ollama error: {}", e),
                                                                                                }
                                                                                                ctx.request_repaint();
                                                                                            });
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                            if ui.button("Status").clicked() {
                                                                                println!("=== Researcher {} Status ===", researcher.id);
                                                                                println!("Global ID: {}", researcher.global_id);
                                                                                println!("Manager: {}", researcher.manager_id);
                                                                                println!("Name: {}", researcher.name);
                                                                                println!("Topic: {}", researcher.topic_mode);
                                                                                println!("Instruction: {}", researcher.instruction);
                                                                                println!("=========================");
                                                                            }
                                                                            if ui.button("Erase").clicked() {
                                                                                researchers_to_remove.push(researcher_id);
                                                                            }
                                                                        });
                                                                    });
                                                                });
                                                            });
                                                        });
                                                        ui.add_space(6.0);
                                                    }
                                                });
                                        });
                                });
                            });
                        });
                    });
                    ui.add_space(6.0);
                }
                
                // Remove managers and all their owned workers/evaluators/researchers
                for manager_id in managers_to_remove {
                    let manager_agent_ids: Vec<usize> = self.agents.iter()
                        .filter(|a| a.manager_id == manager_id)
                        .map(|a| a.id)
                        .collect();
                    let manager_global_ids: Vec<String> = self.managers.iter()
                        .filter(|m| m.id == manager_id)
                        .map(|m| m.global_id.clone())
                        .collect();
                    let agent_global_ids: Vec<String> = self.agents.iter()
                        .filter(|a| a.manager_id == manager_id)
                        .map(|a| a.global_id.clone())
                        .collect();
                    let evaluator_global_ids: Vec<String> = self.evaluators.iter()
                        .filter(|e| e.manager_id == manager_id)
                        .map(|e| e.global_id.clone())
                        .collect();
                    let researcher_global_ids: Vec<String> = self.researchers.iter()
                        .filter(|r| r.manager_id == manager_id)
                        .map(|r| r.global_id.clone())
                        .collect();
                    self.managers.retain(|m| m.id != manager_id);
                    self.agents.retain(|a| a.manager_id != manager_id);
                    self.evaluators.retain(|e| e.manager_id != manager_id);
                    self.researchers.retain(|r| r.manager_id != manager_id);
                    for global_id in manager_global_ids
                        .into_iter()
                        .chain(agent_global_ids)
                        .chain(evaluator_global_ids)
                        .chain(researcher_global_ids)
                    {
                        self.global_ids.remove(&global_id);
                    }
                    self.conversation_loop_handles.retain(|(aid, flag, _)| {
                        if manager_agent_ids.contains(aid) {
                            *flag.lock().unwrap() = false;
                            false
                        } else {
                            true
                        }
                    });
                }

                // Remove agents, evaluators, and researchers that were marked for deletion
                for id in agents_to_remove {
                    if let Some(agent) = self.agents.iter().find(|a| a.id == id) {
                        self.global_ids.remove(&agent.global_id);
                    }
                    self.agents.retain(|a| a.id != id);
                }
                for id in evaluators_to_remove {
                    if let Some(evaluator) = self.evaluators.iter().find(|e| e.id == id) {
                        self.global_ids.remove(&evaluator.global_id);
                    }
                    self.evaluators.retain(|e| e.id != id);
                }
                for id in researchers_to_remove {
                    if let Some(researcher) = self.researchers.iter().find(|r| r.id == id) {
                        self.global_ids.remove(&researcher.global_id);
                    }
                    self.researchers.retain(|r| r.id != id);
                }
                                            });
                    },
                );

                ui.add_space(workspace_gap);
                let outgoing_http_height = ui.available_height().max(0.0);
                self.render_outgoing_http_panel(ui, outgoing_http_height);
                });
            });
        });
    }
}

pub struct AMSAgentsApp {
    ams_agents: AMSAgents,
}

impl AMSAgentsApp {
    pub fn new(rt_handle: Handle) -> Self {
        Self {
            ams_agents: AMSAgents::new(rt_handle),
        }
    }
}

impl eframe::App for AMSAgentsApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        eframe::App::update(&mut self.ams_agents, ctx, frame);
    }
}

