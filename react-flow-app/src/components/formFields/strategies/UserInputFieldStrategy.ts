import { FieldConfig } from "../types";
import { FieldFactoryProps } from "../fieldFactory";

export class UserInputFieldStrategy {
  static createFields(props: FieldFactoryProps): FieldConfig[] {
    return [
      {
        name: "prompt",
        label: "Prompt",
        type: "textarea",
        rows: 4,
        value: props.prompt,
        onChange: props.onPromptChange,
        error: props.validationErrors["prompt"],
      },
      {
        name: "inputType",
        label: "Input Type",
        type: "select",
        value: props.inputType,
        onChange: props.onInputTypeChange,
        options: [
          { value: "string", label: "String" },
          { value: "number", label: "Number" },
          { value: "boolean", label: "Boolean" },
        ],
      },
      {
        name: "required",
        label: "Required",
        type: "checkbox",
        value: props.required,
        onChange: props.onRequiredChange,
      },
      {
        name: "defaultValue",
        label: "Default Value",
        type: "text",
        value: props.defaultValue,
        onChange: props.onDefaultValueChange,
      },
    ];
  }
}
