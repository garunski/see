/**
 * CLI Command Field Strategy
 * Single Responsibility: Create CLI command field configurations
 */

import { FieldConfig } from '../types'
import { FieldFactoryProps } from '../fieldFactory'

export class CliCommandFieldStrategy {
  static createFields(props: FieldFactoryProps): FieldConfig[] {
    return [
      {
        name: 'command',
        label: 'Command',
        type: 'text',
        placeholder: 'e.g., echo, ls, curl',
        value: props.command,
        onChange: props.onCommandChange
      },
      {
        name: 'args',
        label: 'Arguments (comma-separated)',
        type: 'text',
        placeholder: 'e.g., Hello World, -l, /path/to/file',
        value: props.args,
        onChange: props.onArgsChange
      }
    ]
  }
}

