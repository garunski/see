/**
 * User Input Field Strategy
 * Single Responsibility: Create user input field configurations
 */

import { FieldConfig } from '../types'
import { FieldFactoryProps } from '../fieldFactory'

export class UserInputFieldStrategy {
  static createFields(props: FieldFactoryProps): FieldConfig[] {
    return [
      {
        name: 'prompt',
        label: 'Prompt',
        type: 'textarea',
        placeholder: 'Enter prompt to show the user',
        rows: 4,
        value: props.prompt,
        onChange: props.onPromptChange
      },
      {
        name: 'inputType',
        label: 'Input Type',
        type: 'select',
        value: props.inputType,
        onChange: props.onInputTypeChange,
        options: [
          { value: 'text', label: 'Text' },
          { value: 'number', label: 'Number' },
          { value: 'boolean', label: 'Boolean' },
          { value: 'string', label: 'String' }
        ]
      },
      {
        name: 'required',
        label: 'Required',
        type: 'checkbox',
        value: props.required,
        onChange: props.onRequiredChange
      },
      {
        name: 'defaultValue',
        label: 'Default Value',
        type: 'text',
        placeholder: 'Optional default value',
        value: props.defaultValue,
        onChange: props.onDefaultValueChange
      }
    ]
  }
}

