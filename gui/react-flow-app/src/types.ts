export interface WorkflowTask {
  id: string;
  name: string;
  function: {
    name: string;
    input: Record<string, any>;
  };
}

export interface NodePosition {
  x: number;
  y: number;
}

export interface WorkflowVisualizationMetadata {
  node_positions?: Record<string, NodePosition>;
}

export interface Workflow {
  id: string;
  name: string;
  tasks: WorkflowTask[];
  metadata?: WorkflowVisualizationMetadata;
}

export interface MessageFromParent {
  type: 'LOAD_WORKFLOW';
  payload: {
    workflow: Workflow;
  };
}

export interface MessageToParent {
  type: 'SAVE_WORKFLOW';
  payload: {
    workflow: Workflow;
  };
}

