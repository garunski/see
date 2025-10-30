import { TaskFunction } from "../../types";
import { FieldConfig } from "./types";
import {
  CliCommandFieldStrategy,
  CursorAgentFieldStrategy,
  UserInputFieldStrategy,
  CustomFieldStrategy,
} from "./strategies";

export interface FieldFactoryProps {
  command: string;
  args: string[];
  onCommandChange: (value: string) => void;
  onArgsChange: (value: string[]) => void;

  prompt: string;
  configJson: string;
  configError: string;
  onPromptChange: (value: string) => void;
  onConfigJsonChange: (value: string) => void;
  onConfigBlur: () => void;

  inputType: string;
  required: boolean;
  defaultValue: string;
  onInputTypeChange: (value: string) => void;
  onRequiredChange: (value: boolean) => void;
  onDefaultValueChange: (value: string) => void;

  customName: string;
  customInputJson: string;
  customInputError: string;
  onCustomNameChange: (value: string) => void;
  onCustomInputJsonChange: (value: string) => void;
  onCustomInputBlur: () => void;

  validationErrors: Record<string, string>;
}

export class FieldConfigFactory {
  private static readonly strategies = {
    cli_command: CliCommandFieldStrategy,
    cursor_agent: CursorAgentFieldStrategy,
    user_input: UserInputFieldStrategy,
    custom: CustomFieldStrategy,
  } as const;

  static createFields(
    functionType: TaskFunction["name"],
    props: FieldFactoryProps,
  ): FieldConfig[] {
    const strategy = this.strategies[functionType];
    return strategy ? strategy.createFields(props) : [];
  }
}
