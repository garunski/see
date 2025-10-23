import React, { useState, useEffect } from 'react';
import { Node } from 'reactflow';

interface NodeEditorProps {
  node: Node;
  onSave: (updatedTask: any) => void;
  onClose: () => void;
}

const NodeEditor: React.FC<NodeEditorProps> = ({ node, onSave, onClose }) => {
  console.log('NodeEditor rendered with node:', node);
  
  const [formData, setFormData] = useState({
    name: node.data.task.name || '',
    functionType: node.data.task.function.name || 'cli_command',
    command: node.data.task.function.input.command || '',
    args: node.data.task.function.input.args?.join(', ') || '',
    prompt: node.data.task.function.input.prompt || '',
  });

  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [onClose]);

  const handleSave = () => {
    const updatedTask = {
      ...node.data.task,
      name: formData.name,
      function: {
        name: formData.functionType,
        input: formData.functionType === 'cli_command' 
          ? {
              command: formData.command,
              args: formData.args.split(',').map((arg: string) => arg.trim()).filter((arg: string) => arg),
            }
          : {
              prompt: formData.prompt,
            }
      }
    };

    onSave(updatedTask);
    onClose();
  };

  const handleInputChange = (field: string, value: string) => {
    setFormData(prev => ({ ...prev, [field]: value }));
  };

  return (
    <div style={{
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      background: 'rgba(0,0,0,0.5)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      zIndex: 1000,
    }}>
      <div style={{
        background: 'white',
        borderRadius: '8px',
        padding: '24px',
        minWidth: '400px',
        maxWidth: '500px',
        boxShadow: '0 10px 25px rgba(0,0,0,0.2)',
      }}>
        <h3 style={{ margin: '0 0 20px 0', fontSize: '18px', fontWeight: 'bold' }}>
          Edit Task
        </h3>

        <div style={{ marginBottom: '16px' }}>
          <label style={{ display: 'block', marginBottom: '4px', fontSize: '14px', fontWeight: '500' }}>
            Task Name
          </label>
          <input
            type="text"
            value={formData.name}
            onChange={(e) => handleInputChange('name', e.target.value)}
            style={{
              width: '100%',
              padding: '8px 12px',
              border: '1px solid #d1d5db',
              borderRadius: '6px',
              fontSize: '14px',
            }}
            placeholder="Enter task name"
          />
        </div>

        <div style={{ marginBottom: '16px' }}>
          <label style={{ display: 'block', marginBottom: '4px', fontSize: '14px', fontWeight: '500' }}>
            Function Type
          </label>
          <select
            value={formData.functionType}
            onChange={(e) => handleInputChange('functionType', e.target.value)}
            style={{
              width: '100%',
              padding: '8px 12px',
              border: '1px solid #d1d5db',
              borderRadius: '6px',
              fontSize: '14px',
            }}
          >
            <option value="cli_command">CLI Command</option>
            <option value="cursor_agent">Cursor Agent</option>
          </select>
        </div>

        {formData.functionType === 'cli_command' && (
          <>
            <div style={{ marginBottom: '16px' }}>
              <label style={{ display: 'block', marginBottom: '4px', fontSize: '14px', fontWeight: '500' }}>
                Command
              </label>
              <input
                type="text"
                value={formData.command}
                onChange={(e) => handleInputChange('command', e.target.value)}
                style={{
                  width: '100%',
                  padding: '8px 12px',
                  border: '1px solid #d1d5db',
                  borderRadius: '6px',
                  fontSize: '14px',
                }}
                placeholder="e.g., echo, ls, curl"
              />
            </div>

            <div style={{ marginBottom: '16px' }}>
              <label style={{ display: 'block', marginBottom: '4px', fontSize: '14px', fontWeight: '500' }}>
                Arguments (comma-separated)
              </label>
              <input
                type="text"
                value={formData.args}
                onChange={(e) => handleInputChange('args', e.target.value)}
                style={{
                  width: '100%',
                  padding: '8px 12px',
                  border: '1px solid #d1d5db',
                  borderRadius: '6px',
                  fontSize: '14px',
                }}
                placeholder="e.g., Hello World, -l, /path/to/file"
              />
            </div>
          </>
        )}

        {formData.functionType === 'cursor_agent' && (
          <div style={{ marginBottom: '16px' }}>
            <label style={{ display: 'block', marginBottom: '4px', fontSize: '14px', fontWeight: '500' }}>
              Prompt
            </label>
            <textarea
              value={formData.prompt}
              onChange={(e) => handleInputChange('prompt', e.target.value)}
              style={{
                width: '100%',
                padding: '8px 12px',
                border: '1px solid #d1d5db',
                borderRadius: '6px',
                fontSize: '14px',
                minHeight: '80px',
                resize: 'vertical',
              }}
              placeholder="Enter your prompt for the Cursor agent"
            />
          </div>
        )}

        <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
          <button
            onClick={onClose}
            style={{
              padding: '8px 16px',
              background: '#f3f4f6',
              border: '1px solid #d1d5db',
              borderRadius: '6px',
              cursor: 'pointer',
              fontSize: '14px',
              color: '#374151',
            }}
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            style={{
              padding: '8px 16px',
              background: '#3b82f6',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              fontSize: '14px',
              color: 'white',
            }}
          >
            Save
          </button>
        </div>
      </div>
    </div>
  );
};

export default NodeEditor;
