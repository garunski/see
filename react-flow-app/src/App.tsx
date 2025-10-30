import React from "react";
import WorkflowEditor from "./WorkflowEditor";

declare global {
  interface Window {
    WORKFLOW_MODE?: string;
  }
}

const App: React.FC = () => {
  return <WorkflowEditor />;
};

export default App;
