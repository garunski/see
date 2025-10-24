/// JavaScript templates for the workflow editor
/// Extracted from workflow_edit.rs to reduce file size and improve maintainability
/// Message listener script for handling communication between iframe and parent
pub const MESSAGE_LISTENER_SCRIPT: &str = r#"
window.addEventListener('message', function(event) {
    if (event.data && event.data.type === 'NODE_CLICKED') {
        // Update the info div
        const infoDiv = document.getElementById('selected-node-info');
        if (infoDiv && event.data.payload) {
            const name = event.data.payload.nodeName || 'Unknown';
            const func = event.data.payload.functionName || 'unknown';
            infoDiv.textContent = 'Selected: ' + name + ' (' + func + ')';
        }
    } else if (event.data && event.data.type === 'WORKFLOW_NAME_CHANGED') {
        // Handle workflow name changes from React editor
        const nameInput = document.querySelector('input[placeholder="Enter workflow name"]');
        if (nameInput && event.data.payload && event.data.payload.name) {
            nameInput.value = event.data.payload.name;
            nameInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
    }
});
"#;

/// Script to load workflow data into the iframe
pub fn load_workflow_script(workflow_json: &str, workflow_name: &str) -> String {
    format!(
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
        workflow_json, workflow_name
    )
}
