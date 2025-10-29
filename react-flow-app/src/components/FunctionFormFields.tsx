import { Field, Label } from './fieldset'
import { Input } from './input'
import { Select } from './select'
import { Textarea } from './textarea'
import { TaskFunction } from '../types'

interface FunctionFormFieldsProps {
  functionType: TaskFunction['name']
  // CLI Command fields
  command: string
  args: string
  onCommandChange: (value: string) => void
  onArgsChange: (value: string) => void
  // Cursor Agent fields
  prompt: string
  configJson: string
  configError: string
  onPromptChange: (value: string) => void
  onConfigJsonChange: (value: string) => void
  onConfigBlur: () => void
  // User Input fields
  inputType: string
  required: boolean
  defaultValue: string
  onInputTypeChange: (value: string) => void
  onRequiredChange: (value: boolean) => void
  onDefaultValueChange: (value: string) => void
  // Custom fields
  customName: string
  customInputJson: string
  customInputError: string
  onCustomNameChange: (value: string) => void
  onCustomInputJsonChange: (value: string) => void
  onCustomInputBlur: () => void
}

export function FunctionFormFields(props: FunctionFormFieldsProps) {
  const {
    functionType,
    command,
    args,
    onCommandChange,
    onArgsChange,
    prompt,
    configJson,
    configError,
    onPromptChange,
    onConfigJsonChange,
    onConfigBlur,
    inputType,
    required,
    defaultValue,
    onInputTypeChange,
    onRequiredChange,
    onDefaultValueChange,
    customName,
    customInputJson,
    customInputError,
    onCustomNameChange,
    onCustomInputJsonChange,
    onCustomInputBlur,
  } = props

  switch (functionType) {
    case 'cli_command':
      return (
        <>
          <Field>
            <Label>Command</Label>
            <Input 
              value={command} 
              onChange={(e) => onCommandChange(e.target.value)}
              placeholder="e.g., echo, ls, curl"
            />
          </Field>
          <Field>
            <Label>Arguments (comma-separated)</Label>
            <Input 
              value={args} 
              onChange={(e) => onArgsChange(e.target.value)}
              placeholder="e.g., Hello World, -l, /path/to/file"
            />
          </Field>
        </>
      )

    case 'cursor_agent':
      return (
        <>
          <Field>
            <Label>Prompt</Label>
            <Textarea 
              value={prompt} 
              onChange={(e) => onPromptChange(e.target.value)}
              rows={4}
              placeholder="Enter your prompt for the Cursor agent"
            />
          </Field>
          <Field>
            <Label>Config (JSON)</Label>
            <Textarea 
              value={configJson} 
              onChange={(e) => onConfigJsonChange(e.target.value)}
              onBlur={onConfigBlur}
              rows={4}
              placeholder='{"key": "value"}'
              className={configError ? 'border-red-500' : ''}
            />
            {configError && (
              <p className="text-red-600 dark:text-red-400 text-sm mt-1">{configError}</p>
            )}
          </Field>
        </>
      )

    case 'user_input':
      return (
        <>
          <Field>
            <Label>Prompt</Label>
            <Textarea 
              value={prompt} 
              onChange={(e) => onPromptChange(e.target.value)}
              rows={4}
              placeholder="Enter prompt to show the user"
            />
          </Field>
          <Field>
            <Label>Input Type</Label>
            <Select 
              value={inputType} 
              onChange={(e) => onInputTypeChange(e.target.value)}
            >
              <option value="text">Text</option>
              <option value="number">Number</option>
              <option value="boolean">Boolean</option>
              <option value="string">String</option>
            </Select>
          </Field>
          <Field>
            <Label>
              <input 
                type="checkbox" 
                checked={required} 
                onChange={(e) => onRequiredChange(e.target.checked)}
                className="mr-2"
              />
              Required
            </Label>
          </Field>
          <Field>
            <Label>Default Value</Label>
            <Input 
              value={defaultValue} 
              onChange={(e) => onDefaultValueChange(e.target.value)}
              placeholder="Optional default value"
            />
          </Field>
        </>
      )

    case 'custom':
      return (
        <>
          <Field>
            <Label>Function Name</Label>
            <Input 
              value={customName} 
              onChange={(e) => onCustomNameChange(e.target.value)}
              placeholder="e.g., my_custom_function"
            />
          </Field>
          <Field>
            <Label>Input (JSON)</Label>
            <Textarea 
              value={customInputJson} 
              onChange={(e) => onCustomInputJsonChange(e.target.value)}
              onBlur={onCustomInputBlur}
              rows={6}
              placeholder='{"key": "value", "param": 123}'
              className={customInputError ? 'border-red-500' : ''}
            />
            {customInputError && (
              <p className="text-red-600 dark:text-red-400 text-sm mt-1">{customInputError}</p>
            )}
          </Field>
        </>
      )

    default:
      return null
  }
}

