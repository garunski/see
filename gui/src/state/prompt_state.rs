use s_e_e_core::{SystemPrompt, UserPrompt};

#[derive(Debug, Clone)]
pub struct UserPromptState {
    pub prompts: Vec<UserPrompt>,
    pub system_prompts: Vec<SystemPrompt>,
    pub needs_reload: bool,
}

impl Default for UserPromptState {
    fn default() -> Self {
        Self {
            prompts: Vec::new(),
            system_prompts: Vec::new(),
            needs_reload: true,
        }
    }
}

impl UserPromptState {
    pub fn load_prompts(&mut self, prompts: Vec<UserPrompt>) {
        self.prompts = prompts;
        self.needs_reload = false;
    }

    pub fn add_prompt(&mut self, prompt: UserPrompt) {
        self.prompts.push(prompt);
    }

    pub fn update_prompt(&mut self, id: String, updated_prompt: UserPrompt) {
        if let Some(prompt) = self.prompts.iter_mut().find(|p| p.id == id) {
            *prompt = updated_prompt;
        }
    }

    pub fn remove_prompt(&mut self, id: String) {
        self.prompts.retain(|p| p.id != id);
    }

    pub fn get_prompt(&self, id: String) -> Option<&UserPrompt> {
        self.prompts.iter().find(|p| p.id == id)
    }

    pub fn get_prompts(&self) -> &Vec<UserPrompt> {
        &self.prompts
    }

    pub fn get_system_prompts(&self) -> &Vec<SystemPrompt> {
        &self.system_prompts
    }

    pub fn set_system_prompts(&mut self, prompts: Vec<SystemPrompt>) {
        self.system_prompts = prompts;
    }
}
