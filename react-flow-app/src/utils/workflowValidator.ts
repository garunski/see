/**
 * Workflow validation utilities using JSON Schema
 */

import { Node, Edge } from '@xyflow/react';
import { Workflow, WorkflowTask } from '../types';
import { START_NODE_ID } from './layout';

export interface ValidationError {
  type: 'error' | 'warning';
  message: string;
  nodeId?: string;
}

// Note: AJV schema validator not used for function validation due to oneOf noise
// Manual validation is clearer and more user-friendly

/**
 * Validate individual task against schema
 */
function validateTaskAgainstSchema(task: WorkflowTask): ValidationError[] {
  const errors: ValidationError[] = [];
  const taskName = task.name || task.id;
  
  // Manual validation (cleaner than AJV oneOf which generates noise for all function types)
  if (task.function.name === 'cli_command' && task.function.input.command === '') {
    errors.push({
      type: 'error',
      message: `Task "${taskName}": command is required`,
      nodeId: task.id
    });
  }
  
  if (task.function.name === 'cursor_agent' && task.function.input.prompt === '') {
    errors.push({
      type: 'error',
      message: `Task "${taskName}": prompt is required`,
      nodeId: task.id
    });
  }
  
  if (task.function.name === 'user_input') {
    if (task.function.input.prompt === '') {
      errors.push({
        type: 'error',
        message: `Task "${taskName}": prompt is required`,
        nodeId: task.id
      });
    }
    if (task.function.input.input_type === '') {
      errors.push({
        type: 'error',
        message: `Task "${taskName}": input_type is required`,
        nodeId: task.id
      });
    }
  }
  
  return errors;
}

/**
 * Validate workflow structure and connections
 */
export function validateWorkflow(
  nodes: Node[],
  edges: Edge[],
  workflow: Workflow | null
): ValidationError[] {
  const errors: ValidationError[] = [];

  if (!workflow) {
    errors.push({
      type: 'error',
      message: 'No workflow loaded'
    });
    return errors;
  }

  // Filter out the START node
  const taskNodes = nodes.filter(n => n.id !== START_NODE_ID);

  if (taskNodes.length === 0) {
    errors.push({
      type: 'warning',
      message: 'Workflow has no tasks'
    });
    return errors;
  }

  // Validate: All task nodes must be reachable from START
  const connectedNodes = new Set<string>();
  connectedNodes.add(START_NODE_ID);

  // BFS to find all reachable nodes from START
  const queue = [START_NODE_ID];
  while (queue.length > 0) {
    const current = queue.shift()!;
    edges.forEach(edge => {
      if (edge.source === current && !connectedNodes.has(edge.target)) {
        connectedNodes.add(edge.target);
        queue.push(edge.target);
      }
    });
  }

  // Check if all task nodes are reachable from START
  taskNodes.forEach(node => {
    if (!connectedNodes.has(node.id)) {
      const task = node.data?.task as any;
      const taskName = task?.name || node.id;
      errors.push({
        type: 'error',
        message: `Task "${taskName}" is not connected to the workflow`,
        nodeId: node.id
      });
    }
  });

  // Validate each task against the schema
  taskNodes.forEach(node => {
    const task = node.data?.task as WorkflowTask;
    if (task) {
      const taskErrors = validateTaskAgainstSchema(task);
      errors.push(...taskErrors);
    }
  });

  // Validate workflow-level fields
  if (!workflow.name || workflow.name.trim() === '') {
    errors.push({
      type: 'error',
      message: 'Workflow must have a name'
    });
  }

  if (!workflow.id || workflow.id.trim() === '') {
    errors.push({
      type: 'error',
      message: 'Workflow must have an ID'
    });
  }

  return errors;
}

/**
 * Check if workflow has any validation errors (not warnings)
 */
export function hasValidationErrors(errors: ValidationError[]): boolean {
  return errors.some(e => e.type === 'error');
}
