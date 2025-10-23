import React from 'react';
import WorkflowVisualizer from './WorkflowVisualizer';
import WorkflowEditor from './WorkflowEditor';

const App: React.FC = () => {
  // Check if we're in editor mode via URL params or other means
  const urlParams = new URLSearchParams(window.location.search);
  const isEditor = urlParams.get('mode') === 'editor';

  if (isEditor) {
    return <WorkflowEditor />;
  }

  return <WorkflowVisualizer />;
};

export default App;

