/**
 * Serialize React Flow state back to Workflow JSON
 */

import { Node, Edge } from '@xyflow/react';
import { Workflow, WorkflowTask } from '../types';
import { START_NODE_ID } from './layout';

/**
 * Rebuild workflow structure from nodes and edges
 */
export function serializeWorkflow(
  nodes: Node[],
  edges: Edge[],
  workflowName: string,
  workflowId: string
): Workflow {
  // Filter out START node
  const taskNodes = nodes.filter(n => n.id !== START_NODE_ID);

  // Build adjacency map: parent -> children
  const childrenMap = new Map<string, string[]>();
  edges.forEach(edge => {
    if (edge.source !== START_NODE_ID) {
      const children = childrenMap.get(edge.source) || [];
      children.push(edge.target);
      childrenMap.set(edge.source, children);
    }
  });

  // Find root tasks (connected to START node)
  const rootTaskIds = new Set<string>();
  edges.forEach(edge => {
    if (edge.source === START_NODE_ID) {
      rootTaskIds.add(edge.target);
    }
  });

  // Build task map for quick lookup
  const taskMap = new Map<string, WorkflowTask>();
  taskNodes.forEach(node => {
    const task = node.data?.task as WorkflowTask | undefined;
    if (task) {
      taskMap.set(node.id, task);
    }
  });

  // Recursively build task tree
  const buildTaskWithChildren = (taskId: string): WorkflowTask | null => {
    const task = taskMap.get(taskId);
    if (!task) return null;

    const childIds = childrenMap.get(taskId) || [];
    const nextTasks = childIds
      .map(childId => buildTaskWithChildren(childId))
      .filter((t): t is WorkflowTask => t !== null);

    return {
      ...task,
      next_tasks: nextTasks.length > 0 ? nextTasks : undefined
    };
  };

  // Build root tasks with their children
  const tasks = Array.from(rootTaskIds)
    .map(rootId => buildTaskWithChildren(rootId))
    .filter((t): t is WorkflowTask => t !== null);

  // Save node positions as metadata
  const node_positions: Record<string, { x: number; y: number }> = {};
  nodes.forEach(node => {
    node_positions[node.id] = {
      x: node.position.x,
      y: node.position.y
    };
  });

  return {
    id: workflowId,
    name: workflowName,
    tasks,
    metadata: {
      node_positions
    }
  };
}

