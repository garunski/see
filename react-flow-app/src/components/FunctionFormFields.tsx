import { TaskFunction } from "../types";
import {
  FieldConfigFactory,
  FieldFactoryProps,
} from "./formFields/fieldFactory";
import { FieldRenderer } from "./formFields/FieldRenderer";

interface FunctionFormFieldsProps extends FieldFactoryProps {
  functionType: TaskFunction["name"];
}

export function FunctionFormFields(props: FunctionFormFieldsProps) {
  const { functionType, ...factoryProps } = props;

  const fieldConfigs = FieldConfigFactory.createFields(
    functionType,
    factoryProps,
  );

  return (
    <>
      {fieldConfigs.map((config) => (
        <FieldRenderer key={config.name} config={config} />
      ))}
    </>
  );
}
