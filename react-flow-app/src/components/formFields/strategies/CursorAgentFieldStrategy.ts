/**
 * Cursor Agent Field Strategy
 * Single Responsibility: Create cursor agent field configurations
 */

import { FieldConfig } from '../types'
import { FieldFactoryProps } from '../fieldFactory'

export class CursorAgentFieldStrategy {
  static createFields(props: FieldFactoryProps): FieldConfig[] {
    return [
      {
        name: 'prompt',
        label: 'Prompt',
        type: 'textarea',
        rows: 4,
        value: props.prompt,
        onChange: props.onPromptChange,
        error: props.validationErrors['prompt']
      },
      {
        name: 'configJson',
        label: 'Config (JSON)',
        type: 'json',
        rows: 4,
        value: props.configJson,
        onChange: props.onConfigJsonChange,
        onBlur: props.onConfigBlur,
        error: props.configError || props.validationErrors['config']
      }
    ]
  }
}

