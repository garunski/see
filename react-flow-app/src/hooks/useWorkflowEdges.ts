import { useCallback } from 'react';
import { Edge } from '@xyflow/react';
import { Workflow, WorkflowTask } from '../types';

export const useWorkflowEdges = () => {
  const tasksToEdges = useCallback((wf: Workflow): Edge[] => {
    const edgeList: Edge[] = [];
    
    const generateEdgesRecursive = (tasks: WorkflowTask[]) => {
      for (const task of tasks) {
        if (task.next_tasks && task.next_tasks.length > 0) {
          for (const nextTask of task.next_tasks) {
            edgeList.push({
              id: `edge-${task.id}-${nextTask.id}`,
              source: task.id,
              target: nextTask.id,
              type: 'smoothstep',
              animated: true,
              style: { stroke: '#3b82f6', strokeWidth: 2 },
            });
            if (nextTask.next_tasks && nextTask.next_tasks.length > 0) {
              generateEdgesRecursive([nextTask]);
            }
          }
        }
      }
    };
    
    generateEdgesRecursive(wf.tasks);
    return edgeList;
  }, []);

  return { tasksToEdges };
};

