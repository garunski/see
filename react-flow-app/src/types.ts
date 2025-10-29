// Discriminated union matching Rust TaskFunction enum from engine/src/types.rs
// This matches the serde serialization format: #[serde(tag = "name", content = "input")]
export type TaskFunction = 
  | { name: 'cli_command'; input: { command: string; args?: string[] } }
  | { name: 'cursor_agent'; input: { prompt: string; config?: Record<string, any> } }
  | { name: 'user_input'; input: { prompt: string; input_type: string; required?: boolean; default?: any } }
  | { name: 'custom'; input: Record<string, any> };

export interface WorkflowTask {
  id: string;
  name: string;
  function: TaskFunction;
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
  type: 'LOAD_WORKFLOW' | 'GET_WORKFLOW_STATE' | 'UPDATE_NODE' | 'DELETE_EDGE';
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
  edgeId?: string;
}

export interface MessageToParent {
  type: 'SAVE_WORKFLOW' | 'WORKFLOW_STATE' | 'VALIDATION_ERROR' | 'READY';
  payload?: {
    workflow: Workflow;
    error?: string;
  };
}

