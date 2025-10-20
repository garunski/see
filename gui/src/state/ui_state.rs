#[derive(Debug, Clone, Default)]
pub struct UIState {
    pub toast_message: Option<String>,
    pub is_picking_file: bool,
}

impl UIState {
    pub fn show_toast(&mut self, message: String) {
        self.toast_message = Some(message);
    }

    pub fn dismiss_toast(&mut self) {
        self.toast_message = None;
    }

    pub fn set_picking_file(&mut self, picking: bool) {
        self.is_picking_file = picking;
    }
}
