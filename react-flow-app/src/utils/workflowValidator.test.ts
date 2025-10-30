import { describe, it, expect } from "vitest";
import { validateWorkflow, hasValidationErrors } from "./workflowValidator";
import { Node, Edge } from "@xyflow/react";
import { Workflow, WorkflowTask } from "../types";
import { START_NODE_ID } from "./layout";

describe("workflowValidator", () => {
  const createMockWorkflow = (): Workflow => ({
    id: "test-workflow",
    name: "Test Workflow",
    tasks: [],
  });

  const createCliTask = (
    id: string,
    name: string,
    command: string = "",
  ): WorkflowTask => ({
    id,
    name,
    function: {
      name: "cli_command",
      input: { command, args: [] },
    },
  });

  const createCursorTask = (
    id: string,
    name: string,
    prompt: string = "",
  ): WorkflowTask => ({
    id,
    name,
    function: {
      name: "cursor_agent",
      input: { prompt },
    },
  });

  const createUserInputTask = (
    id: string,
    name: string,
    prompt: string = "",
    input_type: string = "string",
  ): WorkflowTask => ({
    id,
    name,
    function: {
      name: "user_input",
      input: { prompt, input_type, required: true },
    },
  });

  const createNode = (task: WorkflowTask): Node => ({
    id: task.id,
    type: "default",
    position: { x: 0, y: 0 },
    data: { task },
  });

  describe("Structural Validation", () => {
    it("should pass with no workflow", () => {
      const errors = validateWorkflow([], [], null);
      expect(errors).toHaveLength(1);
      expect(errors[0].message).toBe("No workflow loaded");
    });

    it("should warn with empty workflow", () => {
      const workflow = createMockWorkflow();
      const errors = validateWorkflow([], [], workflow);
      expect(errors).toHaveLength(1);
      expect(errors[0].type).toBe("warning");
      expect(errors[0].message).toBe("Workflow has no tasks");
    });

    it("should require workflow name", () => {
      const workflow = { ...createMockWorkflow(), name: "" };
      const task = createCliTask("task1", "Task 1", "echo");
      const nodes = [createNode(task)];
      const edges: Edge[] = [
        { id: "e1", source: START_NODE_ID, target: "task1" },
      ];

      const errors = validateWorkflow(nodes, edges, workflow);
      expect(
        errors.some((e) => e.message === "Workflow must have a name"),
      ).toBe(true);
    });

    it("should require workflow ID", () => {
      const workflow = { ...createMockWorkflow(), id: "" };
      const task = createCliTask("task1", "Task 1", "echo");
      const nodes = [createNode(task)];
      const edges: Edge[] = [
        { id: "e1", source: START_NODE_ID, target: "task1" },
      ];

      const errors = validateWorkflow(nodes, edges, workflow);
      expect(errors.some((e) => e.message === "Workflow must have an ID")).toBe(
        true,
      );
    });

    it("should detect disconnected nodes", () => {
      const workflow = createMockWorkflow();
      const task = createCliTask("task1", "Task 1", "echo");
      const nodes = [createNode(task)];
      const edges: Edge[] = [];

      const errors = validateWorkflow(nodes, edges, workflow);
      expect(
        errors.some((e) => e.message.includes("not connected to the workflow")),
      ).toBe(true);
    });

    it("should pass with properly connected nodes", () => {
      const workflow = createMockWorkflow();
      const task = createCliTask("task1", "Task 1", "echo");
      const nodes = [createNode(task)];
      const edges: Edge[] = [
        { id: "e1", source: START_NODE_ID, target: "task1" },
      ];

      const errors = validateWorkflow(nodes, edges, workflow);
      expect(hasValidationErrors(errors)).toBe(false);
    });
  });

  describe("CLI Command Validation", () => {
    it("should require command field", () => {
      const workflow = createMockWorkflow();
      const task = createCliTask("task1", "CLI Task", "");
      const nodes = [createNode(task)];
      const edges: Edge[] = [
        { id: "e1", source: START_NODE_ID, target: "task1" },
      ];

      const errors = validateWorkflow(nodes, edges, workflow);
      expect(
        errors.some(
          (e) =>
            e.message.includes("command") && e.message.includes("required"),
        ),
      ).toBe(true);
    });

    it("should pass with valid command", () => {
      const workflow = createMockWorkflow();
      const task = createCliTask("task1", "CLI Task", "echo hello");
      const nodes = [createNode(task)];
      const edges: Edge[] = [
        { id: "e1", source: START_NODE_ID, target: "task1" },
      ];

      const errors = validateWorkflow(nodes, edges, workflow);
      expect(hasValidationErrors(errors)).toBe(false);
    });
  });

  describe("Cursor Agent Validation", () => {
    it("should require prompt field", () => {
      const workflow = createMockWorkflow();
      const task = createCursorTask("task1", "Agent Task", "");
      const nodes = [createNode(task)];
      const edges: Edge[] = [
        { id: "e1", source: START_NODE_ID, target: "task1" },
      ];

      const errors = validateWorkflow(nodes, edges, workflow);
      expect(
        errors.some(
          (e) => e.message.includes("prompt") && e.message.includes("required"),
        ),
      ).toBe(true);
    });

    it("should pass with valid prompt", () => {
      const workflow = createMockWorkflow();
      const task = createCursorTask("task1", "Agent Task", "Do something");
      const nodes = [createNode(task)];
      const edges: Edge[] = [
        { id: "e1", source: START_NODE_ID, target: "task1" },
      ];

      const errors = validateWorkflow(nodes, edges, workflow);
      expect(hasValidationErrors(errors)).toBe(false);
    });
  });

  describe("User Input Validation", () => {
    it("should require prompt field", () => {
      const workflow = createMockWorkflow();
      const task = createUserInputTask("task1", "Input Task", "", "string");
      const nodes = [createNode(task)];
      const edges: Edge[] = [
        { id: "e1", source: START_NODE_ID, target: "task1" },
      ];

      const errors = validateWorkflow(nodes, edges, workflow);
      expect(
        errors.some(
          (e) => e.message.includes("prompt") && e.message.includes("required"),
        ),
      ).toBe(true);
    });

    it("should require input_type field", () => {
      const workflow = createMockWorkflow();
      const task = createUserInputTask(
        "task1",
        "Input Task",
        "Enter value",
        "",
      );
      const nodes = [createNode(task)];
      const edges: Edge[] = [
        { id: "e1", source: START_NODE_ID, target: "task1" },
      ];

      const errors = validateWorkflow(nodes, edges, workflow);
      expect(
        errors.some(
          (e) =>
            e.message.includes("input_type") && e.message.includes("required"),
        ),
      ).toBe(true);
    });

    it("should pass with valid user input", () => {
      const workflow = createMockWorkflow();
      const task = createUserInputTask(
        "task1",
        "Input Task",
        "Enter name",
        "string",
      );
      const nodes = [createNode(task)];
      const edges: Edge[] = [
        { id: "e1", source: START_NODE_ID, target: "task1" },
      ];

      const errors = validateWorkflow(nodes, edges, workflow);
      expect(hasValidationErrors(errors)).toBe(false);
    });
  });

  describe("hasValidationErrors", () => {
    it("should return true for errors", () => {
      const errors = [{ type: "error" as const, message: "Error" }];
      expect(hasValidationErrors(errors)).toBe(true);
    });

    it("should return false for warnings only", () => {
      const errors = [{ type: "warning" as const, message: "Warning" }];
      expect(hasValidationErrors(errors)).toBe(false);
    });

    it("should return false for empty array", () => {
      expect(hasValidationErrors([])).toBe(false);
    });
  });
});
