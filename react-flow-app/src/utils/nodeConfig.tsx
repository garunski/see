import React from 'react';
import { CommandLineIcon, CursorArrowRaysIcon, Bars3Icon, Cog6ToothIcon } from '@heroicons/react/24/outline';

export interface TaskConfig {
  icon: React.ReactNode;
  colorClass: string;
}

export const getTaskConfig = (functionName: string): TaskConfig => {
  if (functionName === 'cli_command') {
    return {
      icon: <CommandLineIcon className="w-6 h-6" />,
      colorClass: 'bg-blue-600 dark:bg-blue-700'
    };
  } else if (functionName === 'cursor_agent') {
    return {
      icon: <CursorArrowRaysIcon className="w-6 h-6" />,
      colorClass: 'bg-purple-600 dark:bg-purple-700'
    };
  } else if (functionName === 'user_input') {
    return {
      icon: <Bars3Icon className="w-6 h-6" />,
      colorClass: 'bg-amber-600 dark:bg-amber-700'
    };
  } else if (functionName === 'custom') {
    return {
      icon: <Cog6ToothIcon className="w-6 h-6" />,
      colorClass: 'bg-teal-600 dark:bg-teal-700'
    };
  }
  return {
    icon: <CommandLineIcon className="w-6 h-6" />,
    colorClass: 'bg-gray-600 dark:bg-gray-700'
  };
};

