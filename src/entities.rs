#[derive(Clone)]
pub struct Agent {
    pub id: usize,
    pub manager_id: usize,
    pub name: String,
    pub selected: bool,
    pub instruction_mode: String,
    pub instruction: String,
    pub analysis_mode: String,
    pub input: String,
    pub limit_token: bool,
    pub num_predict: String,
    pub in_conversation: bool,
    pub conversation_topic: String,
    pub conversation_mode: String,
    pub conversation_partner_id: Option<usize>,
    pub conversation_active: bool,
}

impl Agent {
    pub fn new(id: usize, manager_id: usize) -> Self {
        Self {
            id,
            manager_id,
            name: format!("Agent {}", id),
            selected: false,
            instruction_mode: String::new(),
            instruction: "You are an assistant".to_string(),
            analysis_mode: String::new(),
            input: String::new(),
            limit_token: false,
            num_predict: String::new(),
            in_conversation: false,
            conversation_topic: String::new(),
            conversation_mode: "Shared".to_string(),
            conversation_partner_id: None,
            conversation_active: false,
        }
    }
}

#[derive(Clone)]
pub struct Evaluator {
    pub id: usize,
    pub manager_id: usize,
    pub name: String,
    pub analysis_mode: String,
    pub instruction: String,
    pub limit_token: bool,
    pub num_predict: String,
    pub active: bool,
}

impl Evaluator {
    pub fn new(id: usize, manager_id: usize) -> Self {
        Self {
            id,
            manager_id,
            name: format!("Evaluator {}", id),
            analysis_mode: String::new(),
            instruction: " ".to_string(),
            limit_token: false,
            num_predict: String::new(),
            active: false,
        }
    }
}

#[derive(Clone)]
pub struct Researcher {
    pub id: usize,
    pub manager_id: usize,
    pub name: String,
    pub topic_mode: String,
    pub instruction: String,
    pub limit_token: bool,
    pub num_predict: String,
    pub active: bool,
}

impl Researcher {
    pub fn new(id: usize, manager_id: usize) -> Self {
        Self {
            id,
            manager_id,
            name: format!("Researcher {}", id),
            topic_mode: String::new(),
            instruction: "".to_string(),
            limit_token: false,
            num_predict: String::new(),
            active: false,
        }
    }
}

#[derive(Clone)]
pub struct AgentManager {
    pub id: usize,
    pub name: String,
}

impl AgentManager {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            name: format!("Agent Manager {}", id),
        }
    }
}
