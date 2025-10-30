import { Node, Edge } from "@xyflow/react";
import { Workflow, WorkflowTask } from "../types";
import { START_NODE_ID } from "./layout";

export function serializeWorkflow(
  nodes: Node[],
  edges: Edge[],
  workflowName: string,
  workflowId: string,
): Workflow {
  const taskNodes = nodes.filter((n) => n.id !== START_NODE_ID);

  const childrenMap = new Map<string, string[]>();
  edges.forEach((edge) => {
    if (edge.source !== START_NODE_ID) {
      const children = childrenMap.get(edge.source) || [];
      children.push(edge.target);
      childrenMap.set(edge.source, children);
    }
  });

  const rootTaskIds = new Set<string>();
  edges.forEach((edge) => {
    if (edge.source === START_NODE_ID) {
      rootTaskIds.add(edge.target);
    }
  });

  const taskMap = new Map<string, WorkflowTask>();
  taskNodes.forEach((node) => {
    const task = node.data?.task as WorkflowTask | undefined;
    if (task) {
      taskMap.set(node.id, task);
    }
  });

  const buildTaskWithChildren = (taskId: string): WorkflowTask | null => {
    const task = taskMap.get(taskId);
    if (!task) return null;

    const childIds = childrenMap.get(taskId) || [];
    const nextTasks = childIds
      .map((childId) => buildTaskWithChildren(childId))
      .filter((t): t is WorkflowTask => t !== null);

    return {
      ...task,
      next_tasks: nextTasks.length > 0 ? nextTasks : undefined,
    };
  };

  const tasks = Array.from(rootTaskIds)
    .map((rootId) => buildTaskWithChildren(rootId))
    .filter((t): t is WorkflowTask => t !== null);

  const node_positions: Record<string, { x: number; y: number }> = {};
  nodes.forEach((node) => {
    node_positions[node.id] = {
      x: node.position.x,
      y: node.position.y,
    };
  });

  return {
    id: workflowId,
    name: workflowName,
    tasks,
    metadata: {
      node_positions,
    },
  };
}
