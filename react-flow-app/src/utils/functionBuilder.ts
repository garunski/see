import { TaskFunction } from "../types";

export function parseDefaultValue(value: string, type: string): any {
  if (value === "") return undefined;

  switch (type) {
    case "number":
      const num = Number(value);
      return isNaN(num) ? value : num;
    case "boolean":
      return value.toLowerCase() === "true";
    default:
      return value;
  }
}

export function validateJson(jsonString: string): boolean {
  try {
    JSON.parse(jsonString);
    return true;
  } catch {
    return false;
  }
}

interface BuildFunctionParams {
  functionType: TaskFunction["name"];
  command?: string;
  args?: string[];
  prompt?: string;
  configJson?: string;
  inputType?: string;
  required?: boolean;
  defaultValue?: string;
  customName?: string;
  customInputJson?: string;
}

export function buildTaskFunction(
  params: BuildFunctionParams,
): TaskFunction | null {
  const { functionType } = params;

  switch (functionType) {
    case "cli_command":
      return {
        name: "cli_command",
        input: {
          command: params.command || "",
          ...(params.args && params.args.length > 0
            ? { args: params.args.filter((arg) => arg.trim() !== "") }
            : {}),
        },
      };

    case "cursor_agent":
      if (!params.configJson) return null;
      const config = JSON.parse(params.configJson);
      return {
        name: "cursor_agent",
        input: {
          prompt: params.prompt || "",
          ...(Object.keys(config).length > 0 ? { config } : {}),
        },
      };

    case "user_input":
      const parsedDefault = params.defaultValue
        ? parseDefaultValue(params.defaultValue, params.inputType || "string")
        : undefined;
      return {
        name: "user_input",
        input: {
          prompt: params.prompt || "",
          input_type: params.inputType || "string",
          ...(params.required !== true ? { required: params.required } : {}),
          ...(parsedDefault !== undefined ? { default: parsedDefault } : {}),
        },
      };

    case "custom":
      if (!params.customInputJson) return null;
      return {
        name: "custom",
        input: JSON.parse(params.customInputJson),
      };

    default:
      return null;
  }
}
