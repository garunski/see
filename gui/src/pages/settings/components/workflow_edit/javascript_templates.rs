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
    } else if (event.data && event.data.type === 'NODE_DOUBLE_CLICKED') {
        
        // Show node editor modal
        const modal = document.getElementById('node-editor-modal');
        if (modal) {
            modal.style.display = 'flex';
            
            // Populate form fields
            const nameInput = document.getElementById('node-name-input');
            const functionSelect = document.getElementById('node-function-select');
            const commandInput = document.getElementById('node-command-input');
            const argsInput = document.getElementById('node-args-input');
            const promptInput = document.getElementById('node-prompt-input');
            
            if (nameInput && event.data.payload.task) {
                nameInput.value = event.data.payload.task.name || '';
            }
            if (functionSelect && event.data.payload.task) {
                functionSelect.value = event.data.payload.task.function?.name || 'cli_command';
            }
            if (commandInput && event.data.payload.task?.function?.input) {
                commandInput.value = event.data.payload.task.function.input.command || '';
            }
            if (argsInput && event.data.payload.task?.function?.input?.args) {
                argsInput.value = event.data.payload.task.function.input.args.join(', ');
            }
            if (promptInput && event.data.payload.task?.function?.input) {
                promptInput.value = event.data.payload.task.function.input.prompt || '';
            }
            
            // Store current node ID for saving
            modal.setAttribute('data-node-id', event.data.payload.nodeId);
            
            // Set up event listeners when modal is shown
            setupModalEventListeners();
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

// Function to set up modal event listeners
function setupModalEventListeners() {
    
    const modal = document.getElementById('node-editor-modal');
    const cancelBtn = document.getElementById('node-editor-cancel');
    const saveBtn = document.getElementById('node-editor-save');
    const functionSelect = document.getElementById('node-function-select');
    const cliFields = document.getElementById('cli-fields');
    const cursorFields = document.getElementById('node-prompt-input');
    
    // Remove existing listeners to avoid duplicates
    if (cancelBtn) {
        cancelBtn.replaceWith(cancelBtn.cloneNode(true));
    }
    if (saveBtn) {
        saveBtn.replaceWith(saveBtn.cloneNode(true));
    }
    if (modal) {
        modal.replaceWith(modal.cloneNode(true));
    }
    
    // Get fresh references after cloning
    const newModal = document.getElementById('node-editor-modal');
    const newCancelBtn = document.getElementById('node-editor-cancel');
    const newSaveBtn = document.getElementById('node-editor-save');
    const newFunctionSelect = document.getElementById('node-function-select');
    const newCliFields = document.getElementById('cli-fields');
    const newCursorFields = document.getElementById('node-prompt-input');
    
    // Close modal on cancel
    if (newCancelBtn) {
        newCancelBtn.addEventListener('click', function() {
            if (newModal) newModal.style.display = 'none';
        });
    }
    
    // Close modal on backdrop click
    if (newModal) {
        newModal.addEventListener('click', function(e) {
            if (e.target === newModal) {
                newModal.style.display = 'none';
            }
        });
    }
    
    // Toggle form fields based on function type
    if (newFunctionSelect) {
        newFunctionSelect.addEventListener('change', function() {
            const isCli = this.value === 'cli_command';
            if (newCliFields) newCliFields.style.display = isCli ? 'block' : 'none';
            if (newCursorFields) newCursorFields.style.display = isCli ? 'none' : 'block';
        });
    }
    
    // Save button handler
    if (newSaveBtn) {
        newSaveBtn.addEventListener('click', function() {
            const nodeId = newModal.getAttribute('data-node-id');
            const name = document.getElementById('node-name-input').value;
            const functionType = document.getElementById('node-function-select').value;
            const command = document.getElementById('node-command-input').value;
            const args = document.getElementById('node-args-input').value;
            const prompt = document.getElementById('node-prompt-input').value;
            
            // Send update to iframe
            const iframe = document.getElementById('workflow-editor-iframe');
            if (iframe && iframe.contentWindow) {
                iframe.contentWindow.postMessage({
                    type: 'UPDATE_NODE',
                    payload: {
                        nodeId,
                        name,
                        functionType,
                        command,
                        args: args.split(',').map(s => s.trim()).filter(s => s),
                        prompt
                    }
                }, '*');
            }
            
            // Close modal
            newModal.style.display = 'none';
        });
    }
}
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
