import { WorkflowTask } from "../types";
import { getTaskConfig } from "./nodeConfig";

export const renderNodeLabel = (task: WorkflowTask) => {
  const config = getTaskConfig(task.function.name);

  return (
    <div className="flex rounded-md shadow-sm dark:shadow-none">
      <div
        className={`flex w-16 shrink-0 items-center justify-center rounded-l-md text-sm font-medium text-white ${config.colorClass}`}
        style={{ minHeight: "60px" }}
      >
        {config.icon}
      </div>
      <div className="flex flex-1 items-center justify-between truncate rounded-r-md border-b border-r border-t border-gray-200 bg-white dark:border-white/10 dark:bg-gray-800/50">
        <div className="flex-1 truncate px-4 py-2 text-sm">
          <div className="font-medium text-gray-900 dark:text-white truncate">
            {task.name}
          </div>
          <p className="text-gray-500 dark:text-gray-400 text-xs truncate">
            {task.function.name}
          </p>
        </div>
      </div>
    </div>
  );
};

export const NODE_STYLE = {
  background: "none",
  border: "none",
  borderRadius: "6px",
  padding: 0,
  height: "auto",
};
