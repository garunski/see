export interface WorkflowTask {
  id: string;
  name: string;
  function: {
    name: string;
    input: Record<string, any>;
  };
  next_tasks?: WorkflowTask[];
  status?: string;
  is_root?: boolean;
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
  type: 'LOAD_WORKFLOW' | 'GET_WORKFLOW_STATE' | 'UPDATE_NODE';
  payload?: {
    workflow?: Workflow;
    workflowName?: string;
    nodeId?: string;
    name?: string;
    functionType?: string;
    command?: string;
    args?: string[];
    prompt?: string;
  };
}

export interface MessageToParent {
  type: 'SAVE_WORKFLOW' | 'WORKFLOW_STATE' | 'VALIDATION_ERROR' | 'READY';
  payload?: {
    workflow: Workflow;
    error?: string;
  };
}

