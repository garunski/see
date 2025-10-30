/**
 * Custom Field Strategy
 * Single Responsibility: Display custom function field configurations (read-only)
 * Note: Custom functions are placeholder/catch-all types and should not be edited in the UI
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
        placeholder: 'Custom function type',
        value: props.customName,
        onChange: () => {}, // No-op for read-only
        disabled: true
      },
      {
        name: 'customInputJson',
        label: 'Input (JSON)',
        type: 'json',
        placeholder: 'Custom function input',
        rows: 6,
        value: props.customInputJson,
        onChange: () => {}, // No-op for read-only
        onBlur: () => {}, // No-op for read-only
        error: '', // No validation for read-only
        disabled: true
      }
    ]
  }
}
