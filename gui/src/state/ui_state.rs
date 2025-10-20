use crate::state::SidebarTab;

#[derive(Debug, Clone)]
pub struct UIState {
    pub show_logs: bool,
    pub show_context: bool,
    pub toast_message: Option<String>,
    pub is_picking_file: bool,
    pub sidebar_tab: SidebarTab,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            show_logs: true,
            show_context: true,
            toast_message: None,
            is_picking_file: false,
            sidebar_tab: SidebarTab::Upload,
        }
    }
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

    pub fn switch_tab(&mut self, tab: SidebarTab) {
        self.sidebar_tab = tab;
    }
}
