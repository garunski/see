import React from 'react';
import WorkflowVisualizer from './WorkflowVisualizer';
import WorkflowEditor from './WorkflowEditor';

// Extend window interface for TypeScript
declare global {
  interface Window {
    WORKFLOW_MODE?: string;
  }
}

const App: React.FC = () => {
  // Check if we're in editor mode via global variable or URL params
  const isEditor = window.WORKFLOW_MODE === 'editor' || 
                   new URLSearchParams(window.location.search).get('mode') === 'editor';

  if (isEditor) {
    return <WorkflowEditor />;
  }

  return <WorkflowVisualizer />;
};

export default App;

