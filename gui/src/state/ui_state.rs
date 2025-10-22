#[derive(Debug, Clone, Default)]
pub struct UIState {
    pub is_picking_file: bool,
}

impl UIState {
    pub fn set_picking_file(&mut self, picking: bool) {
        self.is_picking_file = picking;
    }
}
