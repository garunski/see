import { Field, Label } from "../fieldset";
import { Input } from "../input";
import { Select } from "../select";
import { Textarea } from "../textarea";
import { Button } from "../button";
import { PlusIcon, XMarkIcon } from "@heroicons/react/20/solid";
import { FieldConfig } from "./types";

interface FieldRendererProps {
  config: FieldConfig;
}

export function FieldRenderer({ config }: FieldRendererProps) {
  switch (config.type) {
    case "text":
      return (
        <Field key={config.name}>
          <Label>{config.label}</Label>
          <Input
            value={config.value}
            onChange={(e) => config.onChange(e.target.value)}
            placeholder={config.placeholder}
            disabled={config.disabled}
            className={config.error ? "border-red-500" : ""}
          />
          {config.error && (
            <p className="text-red-600 dark:text-red-400 text-sm mt-1">
              {config.error}
            </p>
          )}
        </Field>
      );

    case "textarea":
      return (
        <Field key={config.name}>
          <Label>{config.label}</Label>
          <Textarea
            value={config.value}
            onChange={(e) => config.onChange(e.target.value)}
            rows={config.rows}
            placeholder={config.placeholder}
            disabled={config.disabled}
            className={config.error ? "border-red-500" : ""}
          />
          {config.error && (
            <p className="text-red-600 dark:text-red-400 text-sm mt-1">
              {config.error}
            </p>
          )}
        </Field>
      );

    case "select":
      return (
        <Field key={config.name}>
          <Label>{config.label}</Label>
          <Select
            value={config.value}
            onChange={(e) => config.onChange(e.target.value)}
          >
            {config.options.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </Select>
        </Field>
      );

    case "checkbox":
      return (
        <Field key={config.name}>
          <Label>
            <input
              type="checkbox"
              checked={config.value}
              onChange={(e) => config.onChange(e.target.checked)}
              className="mr-2"
            />
            {config.label}
          </Label>
        </Field>
      );

    case "json":
      return (
        <Field key={config.name}>
          <Label>{config.label}</Label>
          <Textarea
            value={config.value}
            onChange={(e) => config.onChange(e.target.value)}
            onBlur={config.onBlur}
            rows={config.rows}
            placeholder={config.placeholder}
            className={config.error ? "border-red-500" : ""}
            disabled={config.disabled}
          />
          {config.error && !config.disabled && (
            <p className="text-red-600 dark:text-red-400 text-sm mt-1">
              {config.error}
            </p>
          )}
        </Field>
      );

    case "array":
      return (
        <Field key={config.name}>
          <Label>{config.label}</Label>
          <div className="space-y-2">
            {config.value.map((item, index) => (
              <div key={index} className="flex gap-2">
                <Input
                  value={item}
                  onChange={(e) => {
                    const newArray = [...config.value];
                    newArray[index] = e.target.value;
                    config.onChange(newArray);
                  }}
                  placeholder={config.itemPlaceholder || `Item ${index + 1}`}
                  disabled={config.disabled}
                />
                {!config.disabled && (
                  <button
                    type="button"
                    onClick={() => {
                      const newArray = config.value.filter(
                        (_, i) => i !== index,
                      );
                      config.onChange(newArray);
                    }}
                    className="shrink-0 p-2 text-gray-400 hover:text-red-500 rounded"
                  >
                    <XMarkIcon className="w-5 h-5" />
                  </button>
                )}
              </div>
            ))}
            {!config.disabled && (
              <Button
                onClick={() => {
                  config.onChange([...config.value, ""]);
                }}
                variant="plain"
                className="w-full mt-2"
              >
                <PlusIcon className="w-4 h-4 mr-1 inline" />
                Add Argument
              </Button>
            )}
          </div>
        </Field>
      );

    default:
      return null;
  }
}
