import { WorkflowTask, TaskFunction } from '../types';

/**
 * Factory function to create new workflow task nodes
 * Centralizes task creation logic to avoid duplication
 */
export function createTaskNode(
  functionType: TaskFunction['name'],
  overrides?: Partial<WorkflowTask>
): WorkflowTask {
  const id = `task_${Date.now()}`;
  
  let taskFunction: TaskFunction;
  let name: string;

  switch (functionType) {
    case 'cli_command':
      name = 'New CLI Command Task';
      taskFunction = {
        name: 'cli_command',
        input: { command: '', args: [] }
      };
      break;

    case 'cursor_agent':
      name = 'New Cursor Agent Task';
      taskFunction = {
        name: 'cursor_agent',
        input: { prompt: '' }
      };
      break;

    case 'user_input':
      name = 'New User Input Task';
      taskFunction = {
        name: 'user_input',
        input: { 
          prompt: '',
          input_type: 'string',
          required: true
        }
      };
      break;

    case 'custom':
      name = 'New Custom Task';
      taskFunction = {
        name: 'custom',
        input: {}
      };
      break;
  }

  return {
    id,
    name,
    function: taskFunction,
    ...overrides
  };
}

/**
 * Get human-readable label for function type
 */
export function getFunctionTypeLabel(functionType: TaskFunction['name']): string {
  switch (functionType) {
    case 'cli_command':
      return 'CLI Command';
    case 'cursor_agent':
      return 'Cursor Agent';
    case 'user_input':
      return 'User Input';
    case 'custom':
      return 'Custom';
    default:
      return 'Unknown';
  }
}

/**
 * Get description for function type
 */
export function getFunctionTypeDescription(functionType: TaskFunction['name']): string {
  switch (functionType) {
    case 'cli_command':
      return 'Run shell commands';
    case 'cursor_agent':
      return 'AI-powered automation';
    case 'user_input':
      return 'Request user input';
    case 'custom':
      return 'Custom function';
    default:
      return '';
  }
}

