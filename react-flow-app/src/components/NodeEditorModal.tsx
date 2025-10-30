import { useState, useEffect } from "react";
import { Dialog, DialogActions, DialogBody, DialogTitle } from "./dialog";
import { Field, FieldGroup, Label } from "./fieldset";
import { Input } from "./input";
import { Select } from "./select";
import { Button } from "./button";
import { FunctionFormFields } from "./FunctionFormFields";
import { WorkflowTask } from "../types";
import { validateJson, buildTaskFunction } from "../utils/functionBuilder";

interface NodeEditorModalProps {
  isOpen: boolean;
  node: WorkflowTask | null;
  onSave: (updatedNode: WorkflowTask) => void;
  onClose: () => void;
}

export function NodeEditorModal({
  isOpen,
  node,
  onSave,
  onClose,
}: NodeEditorModalProps) {
  const [name, setName] = useState("");
  const [functionType, setFunctionType] =
    useState<WorkflowTask["function"]["name"]>("cli_command");

  const [command, setCommand] = useState("");
  const [args, setArgs] = useState<string[]>([]);

  const [validationErrors, setValidationErrors] = useState<
    Record<string, string>
  >({});

  const [prompt, setPrompt] = useState("");
  const [configJson, setConfigJson] = useState("{}");
  const [configError, setConfigError] = useState("");

  const [inputType, setInputType] = useState("string");
  const [required, setRequired] = useState(true);
  const [defaultValue, setDefaultValue] = useState("");

  const [customName, setCustomName] = useState("custom");
  const [customInputJson, setCustomInputJson] = useState("{}");
  const [customInputError, setCustomInputError] = useState("");

  useEffect(() => {
    if (!node) return;

    setName(node.name || "");
    setFunctionType(node.function.name);

    switch (node.function.name) {
      case "cli_command":
        setCommand(node.function.input.command || "");
        setArgs(node.function.input.args || []);
        break;

      case "cursor_agent":
        setPrompt(node.function.input.prompt || "");
        setConfigJson(
          JSON.stringify(node.function.input.config || {}, null, 2),
        );
        break;

      case "user_input":
        setPrompt(node.function.input.prompt || "");
        setInputType(node.function.input.input_type || "string");
        setRequired(node.function.input.required !== false);
        setDefaultValue(
          node.function.input.default !== undefined &&
            node.function.input.default !== null
            ? String(node.function.input.default)
            : "",
        );
        break;

      case "custom":
        setCustomName(node.function.name || "custom");
        setCustomInputJson(JSON.stringify(node.function.input || {}, null, 2));
        break;
    }
  }, [node]);

  const handleConfigBlur = () => {
    setConfigError(validateJson(configJson) ? "" : "Invalid JSON");
  };

  const handleCustomInputBlur = () => {
    setCustomInputError(validateJson(customInputJson) ? "" : "Invalid JSON");
  };

  const validateFields = (): boolean => {
    const errors: Record<string, string> = {};

    if (functionType === "cli_command") {
      if (!command.trim()) {
        errors.command = "Command is required";
      }
    }

    if (functionType === "cursor_agent") {
      if (!prompt.trim()) {
        errors.prompt = "Prompt is required";
      }
      if (!validateJson(configJson)) {
        errors.config = "Invalid JSON";
      }
    }

    if (functionType === "user_input") {
      if (!prompt.trim()) {
        errors.prompt = "Prompt is required";
      }
    }

    if (functionType === "custom" && !validateJson(customInputJson)) {
      errors.customInput = "Invalid JSON";
    }

    setValidationErrors(errors);
    return Object.keys(errors).length === 0;
  };

  const handleSave = () => {
    if (!node) return;

    if (!validateFields()) {
      return;
    }

    const updatedFunction = buildTaskFunction({
      functionType,
      command,
      args,
      prompt,
      configJson,
      inputType,
      required,
      defaultValue,
      customName,
      customInputJson,
    });

    if (!updatedFunction) return;

    const updatedNode: WorkflowTask = {
      ...node,
      name,
      function: updatedFunction,
    };

    onSave(updatedNode);
    onClose();
  };

  const handleCancel = () => {
    setConfigError("");
    setCustomInputError("");
    onClose();
  };

  return (
    <Dialog open={isOpen} onClose={handleCancel} size="xl">
      <DialogTitle>Edit Node</DialogTitle>
      <DialogBody>
        <FieldGroup>
          <Field>
            <Label>Node Name</Label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Enter node name"
            />
          </Field>

          <Field>
            <Label>Function Type</Label>
            <Select
              value={functionType}
              onChange={(e) =>
                setFunctionType(
                  e.target.value as WorkflowTask["function"]["name"],
                )
              }
              disabled={functionType === "custom"}
            >
              <option value="cli_command">CLI Command</option>
              <option value="cursor_agent">Cursor Agent</option>
              <option value="user_input">User Input</option>
              {functionType === "custom" && (
                <option value="custom">Custom (Read-only)</option>
              )}
            </Select>
          </Field>

          <FunctionFormFields
            functionType={functionType}
            command={command}
            args={args}
            onCommandChange={setCommand}
            onArgsChange={setArgs}
            prompt={prompt}
            configJson={configJson}
            configError={configError}
            onPromptChange={setPrompt}
            onConfigJsonChange={setConfigJson}
            onConfigBlur={handleConfigBlur}
            inputType={inputType}
            required={required}
            defaultValue={defaultValue}
            onInputTypeChange={setInputType}
            onRequiredChange={setRequired}
            onDefaultValueChange={setDefaultValue}
            customName={customName}
            customInputJson={customInputJson}
            customInputError={customInputError}
            onCustomNameChange={setCustomName}
            onCustomInputJsonChange={setCustomInputJson}
            onCustomInputBlur={handleCustomInputBlur}
            validationErrors={validationErrors}
          />
        </FieldGroup>
      </DialogBody>
      <DialogActions>
        <Button variant="plain" onClick={handleCancel}>
          Cancel
        </Button>
        <Button onClick={handleSave}>Save Changes</Button>
      </DialogActions>
    </Dialog>
  );
}
