pub mod components;

pub use components::WorkflowInfoCard;

use crate::components::ExecutionStatus;
use crate::services::workflow::run_workflow;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use rfd::FileDialog;

#[component]
pub fn UploadPage() -> Element {
    let mut state_provider = use_context::<AppStateProvider>();

    let workflow_file = use_memo(move || state_provider.workflow.read().workflow_file.clone());
    let execution_status =
        use_memo(move || state_provider.workflow.read().execution_status.clone());
    let workflow_result = use_memo(move || state_provider.workflow.read().workflow_result.clone());
    let current_step = use_memo(move || state_provider.workflow.read().current_step);
    let is_picking_file = use_memo(move || state_provider.ui.read().is_picking_file);
    let is_viewing_history = use_memo(move || false);

    let mut on_workflow_file_change = move |value: String| {
        state_provider.workflow.write().workflow_file = value;
        state_provider.history.write().clear_viewing();
    };

    let mut pick_file = move || {
        state_provider.ui.write().set_picking_file(true);
        spawn(async move {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON files", &["json"])
                .set_title("Select Workflow File")
                .pick_file()
            {
                if let Some(path_str) = path.to_str() {
                    state_provider.workflow.write().workflow_file = path_str.to_string();
                    state_provider.history.write().clear_viewing();
                }
            }
            state_provider.ui.write().set_picking_file(false);
        });
    };

    let on_execute = move || {
        let mut workflow_state = state_provider.workflow;
        let mut history_state = state_provider.history;
        spawn(async move {
            let file_path = workflow_state.read().workflow_file.clone();
            workflow_state.write().reset_before_run();

            match run_workflow(file_path, None).await {
                Ok(result) => {
                    workflow_state.write().apply_success(&result);
                    history_state.write().needs_history_reload = true;
                }
                Err(e) => {
                    workflow_state.write().apply_failure(&e.to_string());
                }
            }
        });
    };

    let on_next_step = move || {
        let current = state_provider.workflow.read().current_step;
        let total = state_provider.workflow.read().tasks.len();
        if current < total.saturating_sub(1) {
            state_provider.workflow.write().current_step = current + 1;
        }
    };

    let on_prev_step = move || {
        let current = state_provider.workflow.read().current_step;
        if current > 0 {
            state_provider.workflow.write().current_step = current - 1;
        }
    };

    let on_jump_to_step = move |step: usize| {
        let total = state_provider.workflow.read().tasks.len();
        if step < total {
            state_provider.workflow.write().current_step = step;
        }
    };

    rsx! {
        div { class: "space-y-8",
            div {
                h1 { class: "text-lg font-semibold text-zinc-950 dark:text-white", "Upload Workflow" }
                p { class: "mt-2 text-sm text-zinc-500 dark:text-zinc-400", "Upload and execute workflow files" }
            }

            div { class: "space-y-4",
                div { class: "flex items-center gap-4",
                    input {
                        r#type: "text",
                        placeholder: "Select workflow file...",
                        value: workflow_file(),
                        oninput: move |evt| on_workflow_file_change(evt.value()),
                        class: "block w-full px-3 py-2 text-sm text-zinc-950 dark:text-white bg-white dark:bg-zinc-800 border border-zinc-300 dark:border-zinc-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                    }
                    button {
                        onclick: move |_| pick_file(),
                        disabled: is_picking_file(),
                        class: "px-4 py-2 bg-zinc-900 text-white rounded-lg hover:bg-zinc-800 disabled:opacity-50 dark:bg-white dark:text-zinc-900 dark:hover:bg-zinc-100 transition-colors",
                        if is_picking_file() {
                            svg {
                                class: "w-4 h-4 animate-spin",
                                view_box: "0 0 20 20",
                                fill: "currentColor",
                                path { d: "M10 2a8 8 0 100 16 8 8 0 000-16zM8.5 10a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0z" }
                            }
                        } else { "Browse" }
                    }
                }

                if !workflow_file().is_empty() && workflow_file() != "workflow.json" {
                    div { class: "p-4 bg-zinc-50 dark:bg-zinc-800 rounded-lg border border-zinc-200 dark:border-zinc-700",
                        p { class: "text-sm text-zinc-700 dark:text-zinc-300", "Selected file: {workflow_file()}" }
                    }
                }

                button {
                    onclick: move |_| on_execute(),
                    disabled: matches!(execution_status(), ExecutionStatus::Running) || is_viewing_history(),
                    class: "w-full px-4 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-semibold",
                    if matches!(execution_status(), ExecutionStatus::Running) { "Executing..." } else { "Execute Workflow" }
                }
            }

            if !matches!(execution_status(), ExecutionStatus::Idle) {
                div { class: "p-4 bg-zinc-50 dark:bg-zinc-800 rounded-lg border border-zinc-200 dark:border-zinc-700",
                    div { class: "flex items-center gap-3",
                        div {
                            class: format!("w-3 h-3 rounded-full {}", match execution_status() {
                                ExecutionStatus::Running => "bg-blue-500 animate-pulse",
                                ExecutionStatus::Complete => "bg-emerald-500",
                                ExecutionStatus::Failed => "bg-red-500",
                                _ => "bg-zinc-400"
                            })
                        }
                        span {
                            class: "text-sm font-medium text-zinc-900 dark:text-white",
                            match execution_status() {
                                ExecutionStatus::Running => "Running",
                                ExecutionStatus::Complete => "Complete",
                                ExecutionStatus::Failed => "Failed",
                                _ => "Idle"
                            }
                        }
                    }
                }
            }

            if let Some(result) = workflow_result.read().clone() {
                div { class: "space-y-4",
                    h2 { class: "text-base font-semibold text-zinc-950 dark:text-white", "Execution Results" }
                    WorkflowInfoCard {
                        result: ReadOnlySignal::new(Signal::new(result)),
                        tasks: state_provider.workflow.read().tasks.clone(),
                        current_step: current_step(),
                        on_next_step: on_next_step,
                        on_prev_step: on_prev_step,
                        on_jump_to_step: on_jump_to_step
                    }
                }
            }
        }
    }
}
