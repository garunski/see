import { Node, Edge } from "@xyflow/react";
import { Workflow, WorkflowTask } from "../types";
import { START_NODE_ID } from "./layout";

export interface ValidationError {
  type: "error" | "warning";
  message: string;
  nodeId?: string;
}

function validateTaskAgainstSchema(task: WorkflowTask): ValidationError[] {
  const errors: ValidationError[] = [];
  const taskName = task.name || task.id;

  if (
    task.function.name === "cli_command" &&
    task.function.input.command === ""
  ) {
    errors.push({
      type: "error",
      message: `Task "${taskName}": command is required`,
      nodeId: task.id,
    });
  }

  if (
    task.function.name === "cursor_agent" &&
    task.function.input.prompt === ""
  ) {
    errors.push({
      type: "error",
      message: `Task "${taskName}": prompt is required`,
      nodeId: task.id,
    });
  }

  if (task.function.name === "user_input") {
    if (task.function.input.prompt === "") {
      errors.push({
        type: "error",
        message: `Task "${taskName}": prompt is required`,
        nodeId: task.id,
      });
    }
    if (task.function.input.input_type === "") {
      errors.push({
        type: "error",
        message: `Task "${taskName}": input_type is required`,
        nodeId: task.id,
      });
    }
  }

  return errors;
}

export function validateWorkflow(
  nodes: Node[],
  edges: Edge[],
  workflow: Workflow | null,
): ValidationError[] {
  const errors: ValidationError[] = [];

  if (!workflow) {
    errors.push({
      type: "error",
      message: "No workflow loaded",
    });
    return errors;
  }

  const taskNodes = nodes.filter((n) => n.id !== START_NODE_ID);

  if (taskNodes.length === 0) {
    errors.push({
      type: "warning",
      message: "Workflow has no tasks",
    });
    return errors;
  }

  const connectedNodes = new Set<string>();
  connectedNodes.add(START_NODE_ID);

  const queue = [START_NODE_ID];
  while (queue.length > 0) {
    const current = queue.shift()!;
    edges.forEach((edge) => {
      if (edge.source === current && !connectedNodes.has(edge.target)) {
        connectedNodes.add(edge.target);
        queue.push(edge.target);
      }
    });
  }

  taskNodes.forEach((node) => {
    if (!connectedNodes.has(node.id)) {
      const task = node.data?.task as any;
      const taskName = task?.name || node.id;
      errors.push({
        type: "error",
        message: `Task "${taskName}" is not connected to the workflow`,
        nodeId: node.id,
      });
    }
  });

  taskNodes.forEach((node) => {
    const task = node.data?.task as WorkflowTask;
    if (task) {
      const taskErrors = validateTaskAgainstSchema(task);
      errors.push(...taskErrors);
    }
  });

  if (!workflow.name || workflow.name.trim() === "") {
    errors.push({
      type: "error",
      message: "Workflow must have a name",
    });
  }

  if (!workflow.id || workflow.id.trim() === "") {
    errors.push({
      type: "error",
      message: "Workflow must have an ID",
    });
  }

  return errors;
}

export function hasValidationErrors(errors: ValidationError[]): boolean {
  return errors.some((e) => e.type === "error");
}
