import React from 'react';
import { Panel } from '@xyflow/react';
import { CommandLineIcon, CursorArrowRaysIcon, Bars3Icon, Cog6ToothIcon } from '@heroicons/react/24/outline';
import { WorkflowTask, TaskFunction } from '../types';
import { createTaskNode } from '../utils/taskFactory';

interface NodeToolbarProps {
  onAddNode: (nodeData: WorkflowTask) => void;
}

const NodeToolbar: React.FC<NodeToolbarProps> = ({ onAddNode }) => {
  const handleAddNode = (functionType: TaskFunction['name']) => {
    const nodeData = createTaskNode(functionType);
    onAddNode(nodeData);
  };

  return (
    <Panel position="top-right">
      <div className="bg-white dark:bg-zinc-800 rounded-lg shadow-sm border border-zinc-200 dark:border-zinc-700 flex gap-1 p-1">
        <button
          onClick={() => handleAddNode('cli_command')}
          className="p-2 rounded hover:bg-zinc-100 dark:hover:bg-zinc-700 transition-colors text-zinc-700 dark:text-zinc-300 relative group"
          title="Add CLI Command Task"
        >
          <CommandLineIcon className="w-5 h-5" />
          <span className="absolute bottom-full mb-2 left-1/2 -translate-x-1/2 px-2 py-1 bg-zinc-900 text-white text-xs rounded whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
            CLI Command
          </span>
        </button>
        
        <button
          onClick={() => handleAddNode('cursor_agent')}
          className="p-2 rounded hover:bg-zinc-100 dark:hover:bg-zinc-700 transition-colors text-zinc-700 dark:text-zinc-300 relative group"
          title="Add Cursor Agent Task"
        >
          <CursorArrowRaysIcon className="w-5 h-5" />
          <span className="absolute bottom-full mb-2 left-1/2 -translate-x-1/2 px-2 py-1 bg-zinc-900 text-white text-xs rounded whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
            Cursor Agent
          </span>
        </button>
        
        <button
          onClick={() => handleAddNode('user_input')}
          className="p-2 rounded hover:bg-zinc-100 dark:hover:bg-zinc-700 transition-colors text-zinc-700 dark:text-zinc-300 relative group"
          title="Add User Input Task"
        >
          <Bars3Icon className="w-5 h-5" />
          <span className="absolute bottom-full mb-2 left-1/2 -translate-x-1/2 px-2 py-1 bg-zinc-900 text-white text-xs rounded whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
            User Input
          </span>
        </button>
        
        <button
          onClick={() => handleAddNode('custom')}
          className="p-2 rounded hover:bg-zinc-100 dark:hover:bg-zinc-700 transition-colors text-zinc-700 dark:text-zinc-300 relative group"
          title="Add Custom Task"
        >
          <Cog6ToothIcon className="w-5 h-5" />
          <span className="absolute bottom-full mb-2 left-1/2 -translate-x-1/2 px-2 py-1 bg-zinc-900 text-white text-xs rounded whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
            Custom
          </span>
        </button>
      </div>
    </Panel>
  );
};

export default NodeToolbar;

