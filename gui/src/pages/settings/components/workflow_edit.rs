use crate::components::{Button, ButtonSize, ButtonVariant};
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use see_core::{WorkflowDefinition, WorkflowJson};
use uuid::Uuid;

#[derive(PartialEq, Clone, Copy)]
enum EditMode {
    Visual,
    Json,
}

#[component]
pub fn WorkflowEditPage(id: String) -> Element {
    let state_provider = use_context::<AppStateProvider>();
    let navigator = use_navigator();

    let is_new = id.is_empty();

    let mut content = use_signal(String::new);
    let mut validation_error = use_signal(String::new);
    let mut is_saving = use_signal(|| false);
    let mut can_reset = use_signal(|| false);
    let mut workflow_name = use_signal(String::new);
    let mut edited_workflow_name = use_signal(String::new);
    let mut has_unsaved_changes = use_signal(|| false);
    let mut original_content = use_signal(String::new);
    let mut original_name = use_signal(String::new);
    let mut edit_mode = use_signal(|| EditMode::Visual);
    let selected_node_info = use_signal(|| String::from("No node selected"));
    let _editing_node = use_signal(|| Option::<String>::None);

    let workflow_id_for_effect = id.clone();
    use_effect(move || {
        if !is_new && !workflow_id_for_effect.is_empty() {
            if let Some(workflow) = state_provider
                .settings
                .read()
                .get_workflow(workflow_id_for_effect.clone())
            {
                content.set(workflow.content.clone());
                workflow_name.set(workflow.get_name());
                edited_workflow_name.set(workflow.get_name());
                original_content.set(workflow.content.clone());
                original_name.set(workflow.get_name());
                can_reset.set(workflow.is_default && workflow.is_edited);
            }
        }
    });

    // Update workflow name when content changes (only in JSON mode)
    use_effect(move || {
        if edit_mode() == EditMode::Json {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content()) {
                if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                    let name_str = name.to_string();
                    workflow_name.set(name_str);
                } else {
                    workflow_name.set("Unnamed Workflow".to_string());
                }
            } else {
                workflow_name.set("Invalid Workflow".to_string());
            }
        }
    });

    // Track unsaved changes - simplified logic
    use_effect(move || {
        let content_changed = content() != original_content();
        let name_changed = if edit_mode() == EditMode::Visual {
            edited_workflow_name() != original_name()
        } else {
            workflow_name() != original_name()
        };
        has_unsaved_changes.set(content_changed || name_changed);
    });

    // Prepare workflow JSON for visual editor
    let workflow_json_str = use_memo(move || {
        if let Ok(workflow_json) = serde_json::from_str::<WorkflowJson>(&content()) {
            serde_json::to_string(&workflow_json).ok()
        } else {
            None
        }
    });

    // Mode switching handlers
    let switch_to_visual = move |_| {
        // Validate JSON before switching
        if let Err(e) = serde_json::from_str::<serde_json::Value>(&content()) {
            validation_error.set(format!("Invalid JSON: {}", e));
            return;
        }

        validation_error.set(String::new());
        // Close any open modal when switching modes
        spawn(async move {
            // Use a simple approach - the modal will be hidden when the component re-renders
        });
        edit_mode.set(EditMode::Visual);
    };

    let switch_to_json = move |_| {
        // Update JSON content with edited name before switching
        if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content()) {
            json["name"] = serde_json::Value::String(edited_workflow_name());
            if let Ok(updated_content) = serde_json::to_string(&json) {
                content.set(updated_content);
            }
        }
        // Close any open modal when switching modes
        spawn(async move {
            // Use a simple approach - the modal will be hidden when the component re-renders
        });
        edit_mode.set(EditMode::Json);
    };

    let mut save_workflow = {
        let mut state_provider = state_provider.clone();
        let _ui_state = state_provider.ui;
        let workflow_id_for_save = id.clone();
        move || {
            // Update content with edited name before saving
            let mut final_content = content();
            if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&final_content) {
                json["name"] = serde_json::Value::String(edited_workflow_name());
                if let Ok(updated_content) = serde_json::to_string(&json) {
                    final_content = updated_content;
                }
            }

            if let Err(e) = serde_json::from_str::<serde_json::Value>(&final_content) {
                validation_error.set(format!("Invalid JSON: {}", e));
                return;
            }

            // Validate that JSON has a name field
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&final_content) {
                if json.get("name").and_then(|v| v.as_str()).is_none() {
                    validation_error.set("JSON must contain a 'name' field".to_string());
                    return;
                }
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
                content: final_content.clone(),
                is_default: false,
                is_edited: false,
            };

            if is_new {
                state_provider
                    .settings
                    .write()
                    .add_workflow(workflow.clone());
            } else {
                state_provider
                    .settings
                    .write()
                    .update_workflow(final_id.clone(), workflow.content.clone());
            }

            // Update local state after successful save
            content.set(final_content.clone());
            original_content.set(final_content);
            original_name.set(edited_workflow_name());
            has_unsaved_changes.set(false);

            let _ui_state = _ui_state;
            spawn(async move {
                match see_core::get_global_store() {
                    Ok(store) => {
                        match store
                            .save_settings(&state_provider.settings.read().settings)
                            .await
                        {
                            Ok(_) => {}
                            Err(_e) => {}
                        }
                    }
                    Err(_e) => {}
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
                workflow_name.set(default_workflow.get_name());
                can_reset.set(false);

                let _ui_state = _ui_state;
                spawn(async move {
                    match see_core::get_global_store() {
                        Ok(store) => {
                            match store
                                .save_settings(&state_provider.settings.read().settings)
                                .await
                            {
                                Ok(_) => {}
                                Err(_e) => {}
                            }
                        }
                        Err(_e) => {}
                    }
                });
            }
        }
    };

    rsx! {
        div { class: "space-y-8",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-4",
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Medium,
                        onclick: move |_| {
                            if has_unsaved_changes() {
                                // For now, just navigate back - in a real app you'd want a proper confirmation dialog
                                // TODO: Implement proper confirmation dialog using Dioxus components
                            }
                            // Navigate back using Dioxus router
                            navigator.go_back();
                        },
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
                    // Mode toggle buttons
                    div { class: "flex rounded-lg bg-zinc-100 dark:bg-zinc-800 p-1",
                        Button {
                            variant: if edit_mode() == EditMode::Visual { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                            size: ButtonSize::Small,
                            onclick: switch_to_visual,
                            "Visual Editor"
                        }
                        Button {
                            variant: if edit_mode() == EditMode::Json { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                            size: ButtonSize::Small,
                            onclick: switch_to_json,
                            "JSON Editor"
                        }
                    }

                    if can_reset() {
                        Button {
                            variant: ButtonVariant::Danger,
                            size: ButtonSize::Medium,
                            onclick: move |_| reset_to_default(),
                            "Reset to Default"
                        }
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Medium,
                        disabled: Some(is_saving()),
                        loading: Some(is_saving()),
                        onclick: move |_| save_workflow(),
                        if is_saving() { "Saving..." } else { "Save" }
                    }
                }
            }

            // Content area - conditional rendering based on edit mode
            match edit_mode() {
                EditMode::Visual => rsx! {
                    // Message listener for node clicks from iframe
                    script {
                        dangerous_inner_html: format!(
                            r#"
                            window.addEventListener('message', function(event) {{
                                if (event.data && event.data.type === 'NODE_CLICKED') {{
                                    // Update the info div
                                    const infoDiv = document.getElementById('selected-node-info');
                                    if (infoDiv && event.data.payload) {{
                                        const name = event.data.payload.nodeName || 'Unknown';
                                        const func = event.data.payload.functionName || 'unknown';
                                        infoDiv.textContent = 'Selected: ' + name + ' (' + func + ')';
                                    }}
                                }} else if (event.data && event.data.type === 'NODE_DOUBLE_CLICKED') {{
                                    
                                    // Show node editor modal
                                    const modal = document.getElementById('node-editor-modal');
                                    if (modal) {{
                                        modal.style.display = 'flex';
                                        
                                        // Populate form fields
                                        const nameInput = document.getElementById('node-name-input');
                                        const functionSelect = document.getElementById('node-function-select');
                                        const commandInput = document.getElementById('node-command-input');
                                        const argsInput = document.getElementById('node-args-input');
                                        const promptInput = document.getElementById('node-prompt-input');
                                        
                                        if (nameInput && event.data.payload.task) {{
                                            nameInput.value = event.data.payload.task.name || '';
                                        }}
                                        if (functionSelect && event.data.payload.task) {{
                                            functionSelect.value = event.data.payload.task.function?.name || 'cli_command';
                                        }}
                                        if (commandInput && event.data.payload.task?.function?.input) {{
                                            commandInput.value = event.data.payload.task.function.input.command || '';
                                        }}
                                        if (argsInput && event.data.payload.task?.function?.input?.args) {{
                                            argsInput.value = event.data.payload.task.function.input.args.join(', ');
                                        }}
                                        if (promptInput && event.data.payload.task?.function?.input) {{
                                            promptInput.value = event.data.payload.task.function.input.prompt || '';
                                        }}
                                        
                                        // Store current node ID for saving
                                        modal.setAttribute('data-node-id', event.data.payload.nodeId);
                                        
                                        // Set up event listeners when modal is shown
                                        setupModalEventListeners();
                                    }}
                                }} else if (event.data && event.data.type === 'WORKFLOW_NAME_CHANGED') {{
                                    // Handle workflow name changes from React editor
                                    const nameInput = document.querySelector('input[placeholder="Enter workflow name"]');
                                    if (nameInput && event.data.payload && event.data.payload.name) {{
                                        nameInput.value = event.data.payload.name;
                                        nameInput.dispatchEvent(new Event('input', {{ bubbles: true }}));
                                    }}
                                }}
                            }});
                            
                            // Function to set up modal event listeners
                            function setupModalEventListeners() {{
                                
                                const modal = document.getElementById('node-editor-modal');
                                const cancelBtn = document.getElementById('node-editor-cancel');
                                const saveBtn = document.getElementById('node-editor-save');
                                const functionSelect = document.getElementById('node-function-select');
                                const cliFields = document.getElementById('cli-fields');
                                const cursorFields = document.getElementById('node-prompt-input');
                                
                                // Remove existing listeners to avoid duplicates
                                if (cancelBtn) {{
                                    cancelBtn.replaceWith(cancelBtn.cloneNode(true));
                                }}
                                if (saveBtn) {{
                                    saveBtn.replaceWith(saveBtn.cloneNode(true));
                                }}
                                if (modal) {{
                                    modal.replaceWith(modal.cloneNode(true));
                                }}
                                
                                // Get fresh references after cloning
                                const newModal = document.getElementById('node-editor-modal');
                                const newCancelBtn = document.getElementById('node-editor-cancel');
                                const newSaveBtn = document.getElementById('node-editor-save');
                                const newFunctionSelect = document.getElementById('node-function-select');
                                const newCliFields = document.getElementById('cli-fields');
                                const newCursorFields = document.getElementById('node-prompt-input');
                                
                                // Close modal on cancel
                                if (newCancelBtn) {{
                                    newCancelBtn.addEventListener('click', function() {{
                                        if (newModal) newModal.style.display = 'none';
                                    }});
                                }}
                                
                                // Close modal on backdrop click
                                if (newModal) {{
                                    newModal.addEventListener('click', function(e) {{
                                        if (e.target === newModal) {{
                                            newModal.style.display = 'none';
                                        }}
                                    }});
                                }}
                                
                                // Toggle form fields based on function type
                                if (newFunctionSelect) {{
                                    newFunctionSelect.addEventListener('change', function() {{
                                        const isCli = this.value === 'cli_command';
                                        if (newCliFields) newCliFields.style.display = isCli ? 'block' : 'none';
                                        if (newCursorFields) newCursorFields.style.display = isCli ? 'none' : 'block';
                                    }});
                                }}
                                
                                // Save button handler
                                if (newSaveBtn) {{
                                    newSaveBtn.addEventListener('click', function() {{
                                        const nodeId = newModal.getAttribute('data-node-id');
                                        const name = document.getElementById('node-name-input').value;
                                        const functionType = document.getElementById('node-function-select').value;
                                        const command = document.getElementById('node-command-input').value;
                                        const args = document.getElementById('node-args-input').value;
                                        const prompt = document.getElementById('node-prompt-input').value;
                                        
                                        // Send update to iframe
                                        const iframe = document.getElementById('workflow-editor-iframe');
                                        if (iframe && iframe.contentWindow) {{
                                            iframe.contentWindow.postMessage({{
                                                type: 'UPDATE_NODE',
                                                payload: {{
                                                    nodeId,
                                                    name,
                                                    functionType,
                                                    command,
                                                    args: args.split(',').map(s => s.trim()).filter(s => s),
                                                    prompt
                                                }}
                                            }}, '*');
                                        }}
                                        
                                        // Close modal
                                        newModal.style.display = 'none';
                                    }});
                                }}
                            }}
                            "#
                        )
                    }

                        div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 shadow-sm",
                        div { class: "p-4 border-b border-zinc-200 dark:border-zinc-700",
                            div { class: "flex items-center justify-between",
                                h3 { class: "text-lg font-semibold text-zinc-900 dark:text-white",
                                    "Visual Editor"
                                }
                                div { class: "text-sm text-zinc-500 dark:text-zinc-400",
                                    "Drag nodes to reposition, double-click to edit"
                                }
                            }
                        }
                        div { class: "relative", style: "height: 600px",
                            // Selected node info display
                            div {
                                id: "selected-node-info",
                                class: "absolute top-4 right-4 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 px-3 py-2 rounded-lg text-sm font-medium z-10",
                                "{selected_node_info()}"
                            }

                            if let Some(json_str) = workflow_json_str() {
                                // Script to send workflow data to iframe and set up click handling
                                script {
                                    dangerous_inner_html: format!(
                                        r#"
                                        setTimeout(function() {{
                                            try {{
                                                const iframe = document.getElementById('workflow-editor-iframe');
                                                if (iframe && iframe.contentWindow) {{
                                                    const workflowData = {};
                                                    iframe.contentWindow.postMessage({{
                                                        type: 'LOAD_WORKFLOW',
                                                        payload: {{ 
                                                            workflow: workflowData,
                                                            workflowName: '{}'
                                                        }}
                                                    }}, '*');
                                                }} else {{
                                                    console.error('Editor iframe or contentWindow not available');
                                                }}
                                            }} catch (e) {{
                                                console.error('Failed to send workflow to editor:', e);
                                            }}
                                        }}, 500);
                                        "#,
                                        json_str,
                                        edited_workflow_name()
                                    )
                                }

                                iframe {
                                    id: "workflow-editor-iframe",
                                    srcdoc: format!(
                                        r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Workflow Editor</title>
    <link rel="stylesheet" href="{}" />
    <script>
      // Set mode before React app loads
      window.WORKFLOW_MODE = 'editor';
    </script>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="{}"></script>
  </body>
</html>"#,
                                        asset!("/assets/workflow-visualizer/index.css"),
                                        asset!("/assets/workflow-visualizer/index.js")
                                    ),
                                    class: "w-full h-full border-0 rounded-b-xl",
                                }
                            } else {
                                div { class: "flex items-center justify-center h-full",
                                    div { class: "text-center",
                                        div { class: "text-red-600 dark:text-red-400 mb-2", "Invalid Workflow" }
                                        p { class: "text-zinc-600 dark:text-zinc-400", "Please fix the JSON before switching to visual mode" }
                                    }
                                }
                            }
                        }
                    }

                    // Node Editor Modal
                    div {
                        id: "node-editor-modal",
                        class: "fixed inset-0 z-50 hidden items-center justify-center bg-black bg-opacity-50",
                        style: "display: none;",
                        div {
                            class: "bg-white dark:bg-zinc-800 rounded-xl shadow-xl p-6 max-w-md w-full mx-4",
                            h3 { class: "text-lg font-semibold text-zinc-900 dark:text-white mb-4", "Edit Node" }

                            div { class: "space-y-4",
                                div {
                                    label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-1", "Node Name" }
                                    input {
                                        id: "node-name-input",
                                        type: "text",
                                        class: "w-full px-3 py-2 border border-zinc-300 dark:border-zinc-600 rounded-md text-zinc-900 dark:text-white bg-zinc-50 dark:bg-zinc-700 focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                        placeholder: "Enter node name"
                                    }
                                }

                                div {
                                    label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-1", "Function Type" }
                                    select {
                                        id: "node-function-select",
                                        class: "w-full px-3 py-2 border border-zinc-300 dark:border-zinc-600 rounded-md text-zinc-900 dark:text-white bg-zinc-50 dark:bg-zinc-700 focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                        option { value: "cli_command", "CLI Command" }
                                        option { value: "cursor_agent", "Cursor Agent" }
                                    }
                                }

                                div { id: "cli-fields",
                                    div {
                                        label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-1", "Command" }
                                        input {
                                            id: "node-command-input",
                                            type: "text",
                                            class: "w-full px-3 py-2 border border-zinc-300 dark:border-zinc-600 rounded-md text-zinc-900 dark:text-white bg-zinc-50 dark:bg-zinc-700 focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                            placeholder: "e.g., echo, ls, curl"
                                        }
                                    }

                                    div {
                                        label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-1", "Arguments (comma-separated)" }
                                        input {
                                            id: "node-args-input",
                                            type: "text",
                                            class: "w-full px-3 py-2 border border-zinc-300 dark:border-zinc-600 rounded-md text-zinc-900 dark:text-white bg-zinc-50 dark:bg-zinc-700 focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                            placeholder: "e.g., Hello World, -l, /path/to/file"
                                        }
                                    }
                                }

                                div { id: "cursor-fields", style: "display: none;",
                                    label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-1", "Prompt" }
                                    textarea {
                                        id: "node-prompt-input",
                                        rows: 4,
                                        class: "w-full px-3 py-2 border border-zinc-300 dark:border-zinc-600 rounded-md text-zinc-900 dark:text-white bg-zinc-50 dark:bg-zinc-700 focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                        placeholder: "Enter your prompt for the Cursor agent"
                                    }
                                }
                            }

                            div { class: "flex gap-3 justify-end mt-6",
                                button {
                                    id: "node-editor-cancel",
                                    class: "px-4 py-2 text-sm font-medium text-zinc-700 dark:text-zinc-300 bg-zinc-100 dark:bg-zinc-700 hover:bg-zinc-200 dark:hover:bg-zinc-600 rounded-md transition-colors",
                                    "Cancel"
                                }
                                button {
                                    id: "node-editor-save",
                                    class: "px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-md transition-colors",
                                    "Save Changes"
                                }
                            }
                        }
                    }
                },
                EditMode::Json => rsx! {
                    div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-8 shadow-sm",
                        div { class: "space-y-6",
                            div {
                                label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-2",
                                    "Workflow Name"
                                }
                                div { class: "block w-full rounded-md border-0 py-1.5 text-zinc-900 dark:text-white shadow-sm ring-1 ring-inset ring-zinc-300 dark:ring-zinc-600 bg-zinc-50 dark:bg-zinc-700 sm:text-sm sm:leading-6",
                                    {workflow_name()}
                                }
                                p { class: "mt-1 text-xs text-zinc-500 dark:text-zinc-400",
                                    "Name is extracted from the JSON 'name' field"
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
    }
}

#[component]
pub fn WorkflowEditPageNew() -> Element {
    rsx! {
        WorkflowEditPage { id: "".to_string() }
    }
}
