use crate::router::Route;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;
use see_core::WorkflowDefinition;
use uuid::Uuid;

#[component]
pub fn WorkflowEditPage(id: String) -> Element {
    let state_provider = use_context::<AppStateProvider>();

    let is_new = id.is_empty();

    let mut name = use_signal(String::new);
    let mut content = use_signal(String::new);
    let mut validation_error = use_signal(String::new);
    let mut is_saving = use_signal(|| false);
    let mut can_reset = use_signal(|| false);

    let workflow_id_for_effect = id.clone();
    use_effect(move || {
        if !is_new && !workflow_id_for_effect.is_empty() {
            if let Some(workflow) = state_provider
                .settings
                .read()
                .get_workflow(workflow_id_for_effect.clone())
            {
                name.set(workflow.name.clone());
                content.set(workflow.content.clone());
                can_reset.set(workflow.is_default && workflow.is_edited);
            }
        }
    });

    let mut save_workflow = {
        let mut state_provider = state_provider.clone();
        let _ui_state = state_provider.ui;
        let workflow_id_for_save = id.clone();
        move || {
            if let Err(e) = serde_json::from_str::<serde_json::Value>(&content()) {
                validation_error.set(format!("Invalid JSON: {}", e));
                return;
            }

            validation_error.set(String::new());
            is_saving.set(true);

            let final_id = if is_new {
                Uuid::new_v4().to_string()
            } else {
                workflow_id_for_save.clone()
            };

            let workflow = WorkflowDefinition {
                id: final_id.clone(),
                name: name(),
                content: content(),
                is_default: false,
                is_edited: false,
            };

            if is_new {
                state_provider
                    .settings
                    .write()
                    .add_workflow(workflow.clone());
            } else {
                state_provider.settings.write().update_workflow(
                    final_id.clone(),
                    workflow.name.clone(),
                    workflow.content.clone(),
                );
            }

            let _ui_state = _ui_state;
            spawn(async move {
                match see_core::get_global_store() {
                    Ok(store) => {
                        match store
                            .save_settings(&state_provider.settings.read().settings)
                            .await
                        {
                            Ok(_) => {
                            }
                            Err(_e) => {
                            }
                        }
                    }
                    Err(_e) => {
                    }
                }
                is_saving.set(false);
            });
        }
    };

    let mut reset_to_default = {
        let mut state_provider = state_provider.clone();
        let _ui_state = state_provider.ui;
        let workflow_id_for_reset = id.clone();
        move || {
            let default_workflows = see_core::WorkflowDefinition::get_default_workflows();
            if let Some(default_workflow) = default_workflows
                .iter()
                .find(|w| w.id == workflow_id_for_reset)
            {
                state_provider.settings.write().reset_workflow_to_default(
                    workflow_id_for_reset.clone(),
                    default_workflow.content.clone(),
                );

                content.set(default_workflow.content.clone());
                can_reset.set(false);

                let _ui_state = _ui_state;
                spawn(async move {
                    match see_core::get_global_store() {
                        Ok(store) => {
                            match store
                                .save_settings(&state_provider.settings.read().settings)
                                .await
                            {
                                Ok(_) => {
                                }
                                Err(_e) => {
                                }
                            }
                        }
                        Err(_e) => {
                        }
                    }
                });
            }
        }
    };

    rsx! {
        div { class: "space-y-8",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-4",
                    Link {
                        to: Route::WorkflowsListPage {},
                        class: "inline-flex items-center gap-x-1.5 rounded-md bg-zinc-100 dark:bg-zinc-800 px-3 py-2 text-sm font-semibold text-zinc-900 dark:text-zinc-100 shadow-sm hover:bg-zinc-200 dark:hover:bg-zinc-700",
                        svg { class: "-ml-0.5 h-4 w-4", view_box: "0 0 20 20", fill: "currentColor",
                            path { fill_rule: "evenodd", d: "M17 10a.75.75 0 01-.75.75H5.612l2.158 1.96a.75.75 0 11-1.04 1.08l-3.5-3.25a.75.75 0 010-1.08l3.5-3.25a.75.75 0 111.04 1.08L5.612 9.25H16.25A.75.75 0 0117 10z", clip_rule: "evenodd" }
                        }
                        "Back"
                    }
                    div {
                        h1 { class: "text-xl font-bold text-zinc-900 dark:text-white",
                            if is_new { "Create Workflow" } else { "Edit Workflow" }
                        }
                        p { class: "mt-2 text-zinc-600 dark:text-zinc-400",
                            if is_new { "Create a new workflow definition" } else { "Edit workflow definition" }
                        }
                    }
                }
                div { class: "flex items-center gap-3",
                    if can_reset() {
                        button {
                            onclick: move |_| reset_to_default(),
                            class: "inline-flex items-center gap-x-1.5 rounded-md bg-yellow-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-yellow-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-yellow-600",
                            "Reset to Default"
                        }
                    }
                    button {
                        onclick: move |_| save_workflow(),
                        disabled: is_saving(),
                        class: "inline-flex items-center gap-x-1.5 rounded-md bg-blue-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600 disabled:opacity-50",
                        if is_saving() { "Saving..." } else { "Save" }
                    }
                }
            }

            div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-8 shadow-sm",
                div { class: "space-y-6",
                    div {
                        label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-2",
                            "Workflow Name"
                        }
                        input {
                            r#type: "text",
                            value: "{name()}",
                            oninput: move |evt| name.set(evt.value()),
                            placeholder: "Enter workflow name",
                            class: "block w-full rounded-md border-0 py-1.5 text-zinc-900 dark:text-white shadow-sm ring-1 ring-inset ring-zinc-300 dark:ring-zinc-600 placeholder:text-zinc-400 dark:placeholder:text-zinc-500 focus:ring-2 focus:ring-inset focus:ring-blue-600 dark:bg-zinc-700 sm:text-sm sm:leading-6"
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-2",
                            "Workflow Definition (JSON)"
                        }
                        textarea {
                            value: "{content()}",
                            oninput: move |evt| content.set(evt.value()),
                            placeholder: "Enter workflow JSON definition",
                            rows: 20,
                            class: "block w-full rounded-md border-0 py-1.5 text-zinc-900 dark:text-white shadow-sm ring-1 ring-inset ring-zinc-300 dark:ring-zinc-600 placeholder:text-zinc-400 dark:placeholder:text-zinc-500 focus:ring-2 focus:ring-inset focus:ring-blue-600 dark:bg-zinc-700 sm:text-sm sm:leading-6 font-mono"
                        }
                        if !validation_error().is_empty() {
                            div { class: "mt-2 text-sm text-red-600 dark:text-red-400",
                                {validation_error()}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn WorkflowEditPageNew() -> Element {
    rsx! {
        WorkflowEditPage { id: "".to_string() }
    }
}
