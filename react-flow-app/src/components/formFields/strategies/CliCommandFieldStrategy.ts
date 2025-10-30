/**
 * CLI Command Field Strategy
 * Single Responsibility: Create CLI command field configurations
 */

import { FieldConfig } from '../types'
import { FieldFactoryProps } from '../fieldFactory'

export class CliCommandFieldStrategy {
  static createFields(props: FieldFactoryProps): FieldConfig[] {
    const fields: FieldConfig[] = [
      {
        name: 'command',
        label: 'Command',
        type: 'text',
        value: props.command,
        onChange: props.onCommandChange,
        error: props.validationErrors['command']
      },
      {
        name: 'args',
        label: 'Arguments',
        type: 'array',
        value: props.args,
        onChange: props.onArgsChange
      }
    ]
    
    return fields
  }
}

