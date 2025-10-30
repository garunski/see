import { FieldConfig } from "../types";
import { FieldFactoryProps } from "../fieldFactory";

export class CustomFieldStrategy {
  static createFields(props: FieldFactoryProps): FieldConfig[] {
    return [
      {
        name: "customName",
        label: "Function Name",
        type: "text",
        placeholder: "Custom function type",
        value: props.customName,
        onChange: () => {},
        disabled: true,
      },
      {
        name: "customInputJson",
        label: "Input (JSON)",
        type: "json",
        placeholder: "Custom function input",
        rows: 6,
        value: props.customInputJson,
        onChange: () => {},
        onBlur: () => {},
        error: "",
        disabled: true,
      },
    ];
  }
}
