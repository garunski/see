import React from 'react';
import WorkflowEditor from './WorkflowEditor';

// Extend window interface for TypeScript
declare global {
  interface Window {
    WORKFLOW_MODE?: string;
  }
}

const App: React.FC = () => {
  // Check if we're in editor mode via global variable or URL params
  return <WorkflowEditor />;
};

export default App;

