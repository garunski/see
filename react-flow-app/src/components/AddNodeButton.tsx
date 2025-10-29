import React, { useState } from 'react';
import { WorkflowTask } from '../types';

interface AddNodeButtonProps {
  onAddNode: (nodeData: WorkflowTask) => void;
}

const AddNodeButton: React.FC<AddNodeButtonProps> = ({ onAddNode }) => {
  const [showMenu, setShowMenu] = useState(false);

  const handleAddNode = (functionType: string) => {
    const nodeData: WorkflowTask = {
      id: `task_${Date.now()}`,
      name: `New ${functionType === 'cli_command' ? 'CLI Command' : 'Cursor Agent'} Task`,
      function: {
        name: functionType,
        input: functionType === 'cli_command' 
          ? { command: 'echo', args: ['Hello World'] }
          : { prompt: 'Enter your prompt here' }
      }
    };

    onAddNode(nodeData);
    setShowMenu(false);
  };

  return (
    <div style={{ position: 'absolute', bottom: '20px', right: '20px', zIndex: 1000 }}>
      <button
        onClick={() => setShowMenu(!showMenu)}
        style={{
          width: '56px',
          height: '56px',
          borderRadius: '50%',
          background: '#3b82f6',
          border: 'none',
          color: 'white',
          fontSize: '24px',
          cursor: 'pointer',
          boxShadow: '0 4px 12px rgba(59, 130, 246, 0.4)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          transition: 'all 0.2s ease',
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.transform = 'scale(1.1)';
          e.currentTarget.style.boxShadow = '0 6px 16px rgba(59, 130, 246, 0.5)';
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.transform = 'scale(1)';
          e.currentTarget.style.boxShadow = '0 4px 12px rgba(59, 130, 246, 0.4)';
        }}
      >
        +
      </button>

      {showMenu && (
        <div style={{
          position: 'absolute',
          bottom: '70px',
          right: '0',
          background: 'white',
          borderRadius: '8px',
          boxShadow: '0 4px 12px rgba(0,0,0,0.15)',
          padding: '8px 0',
          minWidth: '220px',
          border: '1px solid #e5e7eb',
        }}>
          <div className="text-xs font-semibold text-zinc-500 px-4 py-2">
            Add New Task
          </div>
          <div
            style={{
              padding: '12px 16px',
              cursor: 'pointer',
              fontSize: '14px',
              color: '#374151',
              transition: 'background 0.15s ease',
            }}
            onClick={() => handleAddNode('cli_command')}
            onMouseEnter={(e) => e.currentTarget.style.background = '#f3f4f6'}
            onMouseLeave={(e) => e.currentTarget.style.background = 'transparent'}
          >
            <div className="flex items-center gap-3">
              <span className="text-lg">📝</span>
              <div>
                <div className="font-medium">CLI Command</div>
                <div className="text-xs text-zinc-500">Run shell commands</div>
              </div>
            </div>
          </div>
          <div
            style={{
              padding: '12px 16px',
              cursor: 'pointer',
              fontSize: '14px',
              color: '#374151',
              transition: 'background 0.15s ease',
            }}
            onClick={() => handleAddNode('cursor_agent')}
            onMouseEnter={(e) => e.currentTarget.style.background = '#f3f4f6'}
            onMouseLeave={(e) => e.currentTarget.style.background = 'transparent'}
          >
            <div className="flex items-center gap-3">
              <span className="text-lg">🤖</span>
              <div>
                <div className="font-medium">Cursor Agent</div>
                <div className="text-xs text-zinc-500">AI-powered automation</div>
              </div>
            </div>
          </div>
        </div>
      )}

      {showMenu && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            zIndex: -1,
          }}
          onClick={() => setShowMenu(false)}
        />
      )}
    </div>
  );
};

export default AddNodeButton;
