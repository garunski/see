use crate::components::{ExecutionStatus, StatusMessage};

#[derive(Debug, Clone, Default)]
pub struct UIState {
    pub status_message: Option<StatusMessage>,
    pub is_picking_file: bool,
}

impl UIState {
    pub fn show_status(&mut self, message: String, status: ExecutionStatus) {
        self.status_message = Some(StatusMessage::new(message, status));
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    pub fn should_auto_clear(&self) -> bool {
        if let Some(ref status_msg) = self.status_message {
            match status_msg.status {
                ExecutionStatus::Complete | ExecutionStatus::Failed => {
                    let now = chrono::Local::now();
                    let elapsed = now.signed_duration_since(status_msg.timestamp);
                    elapsed.num_seconds() >= 3
                }
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn set_picking_file(&mut self, picking: bool) {
        self.is_picking_file = picking;
    }
}
