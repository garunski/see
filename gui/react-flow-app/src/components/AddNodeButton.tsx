import React, { useState } from 'react';

interface AddNodeButtonProps {
  onAddNode: (nodeData: any) => void;
}

const AddNodeButton: React.FC<AddNodeButtonProps> = ({ onAddNode }) => {
  const [showMenu, setShowMenu] = useState(false);

  const handleAddNode = (functionType: string) => {
    const nodeData = {
      id: `task_${Date.now()}`,
      name: `New ${functionType} Task`,
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
          minWidth: '200px',
        }}>
          <div
            style={{
              padding: '12px 16px',
              cursor: 'pointer',
              fontSize: '14px',
              color: '#374151',
            }}
            onClick={() => handleAddNode('cli_command')}
            onMouseEnter={(e) => e.currentTarget.style.background = '#f3f4f6'}
            onMouseLeave={(e) => e.currentTarget.style.background = 'transparent'}
          >
            📝 CLI Command
          </div>
          <div
            style={{
              padding: '12px 16px',
              cursor: 'pointer',
              fontSize: '14px',
              color: '#374151',
            }}
            onClick={() => handleAddNode('cursor_agent')}
            onMouseEnter={(e) => e.currentTarget.style.background = '#f3f4f6'}
            onMouseLeave={(e) => e.currentTarget.style.background = 'transparent'}
          >
            🤖 Cursor Agent
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
