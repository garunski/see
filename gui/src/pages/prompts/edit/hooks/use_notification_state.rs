use crate::components::{NotificationData, NotificationType};
use dioxus::prelude::*;

pub fn use_notification_state() -> Signal<NotificationData> {
    use_signal(|| NotificationData {
        r#type: NotificationType::Success,
        title: String::new(),
        message: String::new(),
        show: false,
    })
}
