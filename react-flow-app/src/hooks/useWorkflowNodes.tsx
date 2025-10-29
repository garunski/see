import { useCallback } from 'react';
import { Node } from '@xyflow/react';
import { Workflow, WorkflowTask } from '../types';
import { renderNodeLabel, NODE_STYLE } from '../utils/nodeRenderer';
import { NODE_WIDTH, NODE_HEIGHT, VERTICAL_SPACING, INITIAL_X, INITIAL_Y } from '../utils/layout';

export const useWorkflowNodes = () => {
  const flattenTasks = useCallback((tasks: WorkflowTask[]): WorkflowTask[] => {
    const flattened: WorkflowTask[] = [];
    const flattenRecursive = (taskList: WorkflowTask[]) => {
      for (const task of taskList) {
        flattened.push(task);
        if (task.next_tasks && task.next_tasks.length > 0) {
          flattenRecursive(task.next_tasks);
        }
      }
    };
    flattenRecursive(tasks);
    return flattened;
  }, []);

  const tasksToNodes = useCallback((wf: Workflow): Node[] => {
    const savedPositions = wf.metadata?.node_positions || {};
    const allTasks = flattenTasks(wf.tasks);
    
    return allTasks.map((task, index) => {
      const savedPos = savedPositions[task.id];
      const position = savedPos || {
        x: INITIAL_X,
        y: INITIAL_Y + index * (NODE_HEIGHT + VERTICAL_SPACING),
      };

      return {
        id: task.id,
        type: 'default',
        position,
        data: {
          label: renderNodeLabel(task),
          task: task,
        },
        style: {
          ...NODE_STYLE,
          width: NODE_WIDTH,
        },
      };
    });
  }, [flattenTasks]);

  return { tasksToNodes, flattenTasks };
};

