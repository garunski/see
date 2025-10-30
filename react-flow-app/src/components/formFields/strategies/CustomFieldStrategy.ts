/**
 * Custom Field Strategy
 * Single Responsibility: Create custom function field configurations
 */

import { FieldConfig } from '../types'
import { FieldFactoryProps } from '../fieldFactory'

export class CustomFieldStrategy {
  static createFields(props: FieldFactoryProps): FieldConfig[] {
    return [
      {
        name: 'customName',
        label: 'Function Name',
        type: 'text',
        placeholder: 'e.g., my_custom_function',
        value: props.customName,
        onChange: props.onCustomNameChange
      },
      {
        name: 'customInputJson',
        label: 'Input (JSON)',
        type: 'json',
        placeholder: '{"key": "value", "param": 123}',
        rows: 6,
        value: props.customInputJson,
        onChange: props.onCustomInputJsonChange,
        onBlur: props.onCustomInputBlur,
        error: props.customInputError
      }
    ]
  }
}

