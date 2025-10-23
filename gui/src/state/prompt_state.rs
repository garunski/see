use see_core::persistence::models::Prompt;

#[derive(Debug, Clone)]
pub struct PromptState {
    pub prompts: Vec<Prompt>,
    pub needs_reload: bool,
}

impl Default for PromptState {
    fn default() -> Self {
        Self {
            prompts: Vec::new(),
            needs_reload: true,
        }
    }
}

impl PromptState {
    pub fn load_prompts(&mut self, prompts: Vec<Prompt>) {
        self.prompts = prompts;
        self.needs_reload = false;
    }

    pub fn add_prompt(&mut self, prompt: Prompt) {
        self.prompts.push(prompt);
    }

    pub fn update_prompt(&mut self, id: String, updated_prompt: Prompt) {
        if let Some(prompt) = self.prompts.iter_mut().find(|p| p.id == id) {
            *prompt = updated_prompt;
        }
    }

    pub fn remove_prompt(&mut self, id: String) {
        self.prompts.retain(|p| p.id != id);
    }

    pub fn get_prompt(&self, id: String) -> Option<&Prompt> {
        self.prompts.iter().find(|p| p.id == id)
    }

    pub fn get_prompts(&self) -> &Vec<Prompt> {
        &self.prompts
    }
}
