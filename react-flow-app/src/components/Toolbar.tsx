import React from 'react';
import { Panel } from 'reactflow';

interface ToolbarProps {
  onAutoLayout: () => void;
  onSave: () => void;
  isDirty: boolean;
}

const Toolbar: React.FC<ToolbarProps> = ({ onAutoLayout, onSave, isDirty }) => {
  return (
    <Panel position="top-right" style={{
      background: 'white',
      padding: '8px',
      borderRadius: '8px',
      boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
      fontFamily: 'system-ui, -apple-system, sans-serif',
      display: 'flex',
      gap: '8px',
    }}>
      <button
        onClick={onAutoLayout}
        style={{
          padding: '8px 12px',
          background: '#f3f4f6',
          border: '1px solid #d1d5db',
          borderRadius: '6px',
          cursor: 'pointer',
          fontSize: '12px',
          color: '#374151',
        }}
        onMouseEnter={(e) => e.currentTarget.style.background = '#e5e7eb'}
        onMouseLeave={(e) => e.currentTarget.style.background = '#f3f4f6'}
      >
        📐 Auto Layout
      </button>
      
      <button
        onClick={onSave}
        disabled={!isDirty}
        style={{
          padding: '8px 12px',
          background: isDirty ? '#3b82f6' : '#f3f4f6',
          border: '1px solid #d1d5db',
          borderRadius: '6px',
          cursor: isDirty ? 'pointer' : 'not-allowed',
          fontSize: '12px',
          color: isDirty ? 'white' : '#9ca3af',
        }}
        onMouseEnter={(e) => {
          if (isDirty) {
            e.currentTarget.style.background = '#2563eb';
          }
        }}
        onMouseLeave={(e) => {
          if (isDirty) {
            e.currentTarget.style.background = '#3b82f6';
          }
        }}
      >
        💾 Save
      </button>
    </Panel>
  );
};

export default Toolbar;
