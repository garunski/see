pub mod alert;
pub mod badge;
pub mod button;
pub mod dialog;
pub mod forms;
pub mod layout;
pub mod notification;
pub mod slideout;

pub use alert::{Alert, AlertType};
pub use badge::{Badge, BadgeButton, BadgeColor};
pub use button::{Button, ButtonSize, ButtonVariant};
pub use dialog::ConfirmDialog;
pub use forms::{TextInput, TextareaInput, UserInputForm, ValidationMessage};
pub use layout::{EmptyState, List, PageHeader, SectionCard};
pub use notification::{Notification, NotificationData, NotificationType};
pub use slideout::Slideout;

#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowExecutionStatus {
    Idle,
    Running,
}
