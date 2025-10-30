import React, { useState, useCallback, useEffect } from 'react';
import {
  ReactFlow,
  Controls,
  Background,
  useNodesState,
  useEdgesState,
  addEdge,
  Connection,
  Node,
  Edge,
  BackgroundVariant,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { CommandLineIcon, CursorArrowRaysIcon, PlayIcon, Bars3Icon, ArrowsPointingOutIcon, ExclamationTriangleIcon, TrashIcon } from '@heroicons/react/24/outline';
import { CheckCircleIcon as CheckCircleIconSolid, XCircleIcon as XCircleIconSolid } from '@heroicons/react/24/solid';
import { Workflow, MessageFromParent, WorkflowTask } from './types';
import { NodeEditorModal } from './components/NodeEditorModal';
import { Input } from './components/input';
import { CustomEdge } from './components/CustomEdge';
import { useWorkflowNodes } from './hooks/useWorkflowNodes';
import { useWorkflowEdges } from './hooks/useWorkflowEdges';
import { renderNodeLabel } from './utils/nodeRenderer';
import { getLayoutedElements, NODE_WIDTH, START_NODE_ID, START_NODE_SIZE, INITIAL_X, INITIAL_Y } from './utils/layout';
import { createTaskNode } from './utils/taskFactory';
import { serializeWorkflow } from './utils/workflowSerializer';
import { validateWorkflow, hasValidationErrors, ValidationError } from './utils/workflowValidator';

const edgeTypes = {
  default: CustomEdge,
  smoothstep: CustomEdge,
};

const WorkflowEditor: React.FC = () => {
  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
  const [workflow, setWorkflow] = useState<Workflow | null>(null);
  const [isLoaded, setIsLoaded] = useState(false);
  const [workflowName, setWorkflowName] = useState<string>('');
  
  const [editingNode, setEditingNode] = useState<WorkflowTask | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);
  
  const [validationErrors, setValidationErrors] = useState<ValidationError[]>([]);
  const [showValidationPanel, setShowValidationPanel] = useState(false);
  const [isValid, setIsValid] = useState(true);
  const [selectedNodes, setSelectedNodes] = useState<string[]>([]);

  const { tasksToNodes, flattenTasks } = useWorkflowNodes();
  const { tasksToEdges } = useWorkflowEdges();

  // Track selected nodes
  const onSelectionChange = useCallback(({ nodes }: { nodes: Node[] }) => {
    setSelectedNodes(nodes.map(n => n.id));
  }, []);

  // Click handler with postMessage to Dioxus parent
  const handleNodeClick = useCallback((_event: React.MouseEvent, node: Node) => {
    const task = node.data.task as WorkflowTask | undefined;
    // Send to Dioxus parent window - only send cloneable data
    window.parent.postMessage({
      type: 'NODE_CLICKED',
      payload: {
        nodeId: node.id,
        nodeName: task?.name,
        functionName: task?.function?.name
      }
    }, '*');
  }, []);

  // Double-click handler for opening node editor modal
  const handleNodeDoubleClick = useCallback((_event: React.MouseEvent, node: Node) => {
    const task = node.data.task as WorkflowTask;
    setEditingNode(task);
    setIsModalOpen(true);
  }, []);

  const handleSaveNode = useCallback((updatedNode: WorkflowTask) => {
    setNodes((currentNodes) => 
      currentNodes.map((node) => {
        if (node.id === updatedNode.id) {
          return {
            ...node,
            data: {
              ...node.data,
              task: updatedNode,
              label: renderNodeLabel(updatedNode)
            }
          }
        }
        return node;
      })
    );
  }, [setNodes]);

  const handleDeleteNode = useCallback((nodeId: string) => {
    // Cannot delete START node
    if (nodeId === START_NODE_ID) {
      return;
    }

    // Remove node
    setNodes((nds) => nds.filter((n) => n.id !== nodeId));
    
    // Remove all edges connected to this node
    setEdges((eds) => eds.filter((e) => e.source !== nodeId && e.target !== nodeId));
    
    // Update workflow structure
    if (workflow) {
      const removeTaskRecursive = (tasks: WorkflowTask[]): WorkflowTask[] => {
        return tasks
          .filter(task => task.id !== nodeId)
          .map(task => ({
            ...task,
            next_tasks: task.next_tasks ? removeTaskRecursive(task.next_tasks) : undefined
          }));
      };
      
      const updatedWorkflow = {
        ...workflow,
        tasks: removeTaskRecursive(workflow.tasks)
      };
      setWorkflow(updatedWorkflow);
    }
  }, [setNodes, setEdges, workflow, setWorkflow]);

  // Delete selected nodes (for delete button)
  const handleDeleteSelected = useCallback(() => {
    selectedNodes.forEach(nodeId => {
      if (nodeId !== START_NODE_ID) {
        handleDeleteNode(nodeId);
      }
    });
    setSelectedNodes([]);
  }, [selectedNodes, handleDeleteNode]);

  // Keyboard handler for Delete/Backspace
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Only handle if not in an input/textarea
      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
        return;
      }

      if (event.key === 'Delete' || event.key === 'Backspace') {
        event.preventDefault();
        handleDeleteSelected();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleDeleteSelected]);

  // Periodic validation check (every 2 seconds)
  useEffect(() => {
    const doValidation = () => {
      const errors = validateWorkflow(nodes, edges, workflow);
      setValidationErrors(errors);
      const valid = !hasValidationErrors(errors);
      setIsValid(valid);
    };

    // Run initial validation
    doValidation();

    // Set up periodic validation
    const intervalId = setInterval(() => {
      doValidation();
    }, 2000); // Run every 2 seconds

    return () => clearInterval(intervalId);
  }, [nodes, edges, workflow]);

  const handleAutoLayout = useCallback(() => {
    const { nodes: layoutedNodes, edges: layoutedEdges } = getLayoutedElements(nodes, edges);
    setNodes(layoutedNodes);
    setEdges(layoutedEdges);
  }, [nodes, edges, setNodes, setEdges]);

  // Run validation and update state
  const runValidation = useCallback(() => {
    const errors = validateWorkflow(nodes, edges, workflow);
    setValidationErrors(errors);
    const valid = !hasValidationErrors(errors);
    setIsValid(valid);
    return errors;
  }, [nodes, edges, workflow]);

  // Handle manual validation button click
  const handleValidate = useCallback(() => {
    const errors = runValidation();
    setShowValidationPanel(true);
    return errors;
  }, [runValidation]);

  // Toggle validation panel
  const toggleValidationPanel = useCallback(() => {
    setShowValidationPanel(prev => !prev);
  }, []);

  const handleSaveWorkflow = useCallback(() => {
    // Validate before saving
    const errors = runValidation();
    
    if (hasValidationErrors(errors)) {
      // Show validation panel with errors
      setShowValidationPanel(true);
      console.error('Cannot save workflow with validation errors:', errors);
      return;
    }

    if (!workflow) {
      console.error('No workflow to save');
      return;
    }

    // Serialize workflow from current state
    const serializedWorkflow = serializeWorkflow(nodes, edges, workflowName, workflow.id);
    
    // Send to parent window (Dioxus/Rust)
    window.parent.postMessage({
      type: 'SAVE_WORKFLOW',
      payload: { workflow: serializedWorkflow }
    }, '*');

    console.log('Workflow saved:', serializedWorkflow);
  }, [nodes, edges, workflowName, workflow, handleValidate]);

  const handleAddNode = useCallback((taskData: WorkflowTask) => {
    // Calculate smart position based on existing nodes
    // Offset each new node so they don't overlap
    const calculatePosition = (existingNodes: Node[]) => {
      const NODE_SPACING = 50; // pixels between nodes
      const INITIAL_X = 300;
      const INITIAL_Y = 200;
      
      // Filter out the START node for counting
      const taskNodes = existingNodes.filter(n => n.id !== START_NODE_ID);
      const nodeCount = taskNodes.length;
      
      // Cascade diagonally: each new node moves right and down
      return {
        x: INITIAL_X + (nodeCount * NODE_SPACING),
        y: INITIAL_Y + (nodeCount * NODE_SPACING)
      };
    };

    setNodes((nds) => {
      const position = calculatePosition(nds);
      
      const newNode: Node = {
        id: taskData.id,
        type: 'default',
        position,
        data: {
          label: renderNodeLabel(taskData),
          task: taskData,
        },
        style: {
          background: 'none',
          border: 'none',
          borderRadius: '6px',
          width: NODE_WIDTH,
          height: 'auto',
          padding: 0,
        },
      };

      return [...nds, newNode];
    });
    
    if (workflow) {
      const updatedWorkflow = {
        ...workflow,
        tasks: [...workflow.tasks, taskData]
      };
      setWorkflow(updatedWorkflow);
    }
  }, [setNodes, workflow, setWorkflow]);

  // Handle edge deletion
  const handleDeleteEdge = useCallback((edgeId: string) => {
    setEdges((eds) => eds.filter((e) => e.id !== edgeId));
    
    // Update workflow structure
    if (workflow) {
      const updatedWorkflow = { ...workflow };
      
      // Find the edge to remove
      const edgeToRemove = edges.find(e => e.id === edgeId);
      if (edgeToRemove) {
        const removeEdgeFromTasks = (tasks: WorkflowTask[]): WorkflowTask[] => {
          return tasks.map(task => {
            if (task.id === edgeToRemove.source) {
              return {
                ...task,
                next_tasks: (task.next_tasks || []).filter(nt => nt.id !== edgeToRemove.target)
              };
            }
            if (task.next_tasks && task.next_tasks.length > 0) {
              return {
                ...task,
                next_tasks: removeEdgeFromTasks(task.next_tasks)
              };
            }
            return task;
          });
        };
        
        updatedWorkflow.tasks = removeEdgeFromTasks(updatedWorkflow.tasks);
        setWorkflow(updatedWorkflow);
      }
    }
  }, [edges, setEdges, workflow, setWorkflow]);

  // Listen for messages from parent window (Dioxus)
  useEffect(() => {
    const handleMessage = (event: MessageEvent<MessageFromParent>) => {
      if (event.data.type === 'DELETE_EDGE' && event.data.edgeId) {
        handleDeleteEdge(event.data.edgeId);
      } else if (event.data.type === 'GET_WORKFLOW_STATE') {
        // Parent is requesting current workflow state (e.g., when Save is clicked)
        handleSaveWorkflow();
      } else if (event.data.type === 'LOAD_WORKFLOW' && event.data.payload?.workflow) {
        const wf = event.data.payload.workflow;
        setWorkflow(wf);
        
        // Set workflow name from payload
        if (event.data.payload.workflowName) {
          setWorkflowName(event.data.payload.workflowName);
        } else if (wf.name) {
          setWorkflowName(wf.name);
        }
        
        const savedPositions = wf.metadata?.node_positions || {};
        const hasSavedPositions = Object.keys(savedPositions).length > 0;

        // Build base nodes and edges from workflow
        const taskNodes = tasksToNodes(wf);
        const taskEdges = tasksToEdges(wf);

        // Create visual-only START node that connects to first-level (root) tasks only
        // Determine roots by collecting all child ids from next_tasks and filtering
        const collectChildIds = (tasks: WorkflowTask[], set: Set<string>) => {
          tasks.forEach((t) => {
            t.next_tasks?.forEach((nt) => {
              set.add(nt.id);
              if (nt.next_tasks && nt.next_tasks.length > 0) {
                collectChildIds([nt], set);
              }
            });
          });
        };
        const childIds = new Set<string>();
        collectChildIds(wf.tasks, childIds);
        const allTasksFlat = flattenTasks(wf.tasks);
        const rootTasks = allTasksFlat.filter((t) => !childIds.has(t.id));
        const startNode: Node = {
          id: START_NODE_ID,
          type: 'default',
          position: { x: INITIAL_X, y: INITIAL_Y },
          data: {
            label: (
              <div className="flex items-center justify-center" style={{ width: START_NODE_SIZE, height: START_NODE_SIZE }} title="Start">
                <PlayIcon className="w-5 h-5 text-white" />
              </div>
            ),
          },
          sourcePosition: 'bottom' as any,
          targetPosition: 'top' as any,
          style: {
            width: START_NODE_SIZE,
            height: START_NODE_SIZE,
            border: 'none',
            borderRadius: '50%',
            background: '#10b981',
            padding: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          },
        };

        const startEdges: Edge[] = rootTasks.map((task) => ({
          id: `edge-${START_NODE_ID}-${task.id}`,
          source: START_NODE_ID,
          target: task.id,
          type: 'smoothstep',
          animated: true,
          style: { stroke: '#10b981', strokeWidth: 2 },
        }));

        const initialNodes = [startNode, ...taskNodes];
        const initialEdges = [...startEdges, ...taskEdges];
        
        // Apply Dagre layout only if no saved positions exist
        const { nodes: newNodes, edges: newEdges } = hasSavedPositions 
          ? { nodes: initialNodes, edges: initialEdges }
          : getLayoutedElements(initialNodes, initialEdges);
        
        setNodes(newNodes);
        setEdges(newEdges);
        setIsLoaded(true);
      }
    };

    window.addEventListener('message', handleMessage);
    return () => window.removeEventListener('message', handleMessage);
  }, [tasksToNodes, tasksToEdges, setNodes, setEdges, handleDeleteEdge]);

  const onConnect = useCallback(
    (params: Connection) => {
      const newEdge = {
        ...params,
        type: 'smoothstep',
        animated: true,
        style: { stroke: '#3b82f6', strokeWidth: 2 },
      };
      setEdges((eds) => addEdge(newEdge, eds));
    },
    [setEdges],
  );

  if (!isLoaded) {
    return (
      <div className="flex justify-center items-center h-screen flex-col gap-5">
        <div className="w-10 h-10 border-3 border-zinc-200 border-t-blue-500 rounded-full animate-spin" />
        <div className="text-zinc-600 dark:text-zinc-400">Loading workflow editor...</div>
      </div>
    );
  }

  return (
    <div className="w-full h-full min-h-0 flex flex-col">
      <nav className="relative bg-white shadow dark:bg-gray-800/50 dark:shadow-none dark:after:pointer-events-none dark:after:absolute dark:after:inset-x-0 dark:after:bottom-0 dark:after:h-px dark:after:bg-white/10">
        <div className="mx-auto max-w-full px-4 sm:px-6 lg:px-8">
          <div className="flex h-16 justify-between">
            <div className="flex px-2 lg:px-0">
              <div className="flex items-center gap-4">
                {/* Validation Status Indicator */}
                <button
                  onClick={toggleValidationPanel}
                  className={`relative shrink-0 rounded-full p-2 focus:outline focus:outline-2 focus:outline-offset-2 group transition-colors ${
                    isValid 
                      ? 'text-green-500 hover:text-green-600 focus:outline-green-500 dark:text-green-400 dark:hover:text-green-300' 
                      : 'text-red-500 hover:text-red-600 focus:outline-red-500 dark:text-red-400 dark:hover:text-red-300'
                  }`}
                  title={isValid ? 'Workflow is valid' : `${validationErrors.length} validation error(s)`}
                >
                  {isValid ? (
                    <CheckCircleIconSolid className="w-7 h-7" />
                  ) : (
                    <XCircleIconSolid className="w-7 h-7" />
                  )}
                  <span className="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 bg-gray-900 text-white text-xs rounded whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50">
                    {isValid ? 'Valid' : `${validationErrors.length} Error${validationErrors.length > 1 ? 's' : ''}`}
                  </span>
                </button>

                <Input
                  value={workflowName}
                  onChange={(e) => {
                    const newName = e.target.value;
                    setWorkflowName(newName);
                    window.parent.postMessage({
                      type: 'WORKFLOW_NAME_CHANGED',
                      payload: { name: newName }
                    }, '*');
                  }}
                  placeholder="Workflow Name"
                  className="min-w-[300px]"
                />
                
                <button
                  onClick={handleAutoLayout}
                  className="relative shrink-0 rounded-full p-2 text-gray-400 hover:text-gray-500 focus:outline focus:outline-2 focus:outline-offset-2 focus:outline-indigo-600 dark:hover:text-white dark:focus:outline-indigo-500 group"
                  title="Auto Layout"
                >
                  <ArrowsPointingOutIcon className="w-6 h-6" />
                  <span className="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 bg-gray-900 text-white text-xs rounded whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50">
                    Auto Layout
                  </span>
                </button>
                
                <button
                  onClick={handleDeleteSelected}
                  disabled={selectedNodes.length === 0 || (selectedNodes.length === 1 && selectedNodes[0] === START_NODE_ID)}
                  className="relative shrink-0 rounded-full p-2 text-gray-400 hover:text-red-500 focus:outline focus:outline-2 focus:outline-offset-2 focus:outline-red-600 dark:hover:text-red-400 dark:focus:outline-red-500 group disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:text-gray-400"
                  title={selectedNodes.length > 0 ? `Delete ${selectedNodes.length} selected node(s)` : "Delete Selected (Del)"}
                >
                  <TrashIcon className="w-6 h-6" />
                  <span className="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 bg-gray-900 text-white text-xs rounded whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50">
                    {selectedNodes.length > 0 ? `Delete (${selectedNodes.length})` : 'Delete (Del)'}
                  </span>
                </button>
                
                <div className="flex items-center gap-1 border-l border-gray-200 dark:border-white/10 pl-4">
                  <button
                    onClick={() => handleAddNode(createTaskNode('cli_command'))}
                    className="relative shrink-0 rounded-full p-2 text-gray-400 hover:text-gray-500 focus:outline focus:outline-2 focus:outline-offset-2 focus:outline-indigo-600 dark:hover:text-white dark:focus:outline-indigo-500 group"
                    title="Add CLI Command Task"
                  >
                    <CommandLineIcon className="w-6 h-6" />
                    <span className="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 bg-gray-900 text-white text-xs rounded whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50">
                      CLI Command
                    </span>
                  </button>
                  
                  <button
                    onClick={() => handleAddNode(createTaskNode('cursor_agent'))}
                    className="relative shrink-0 rounded-full p-2 text-gray-400 hover:text-gray-500 focus:outline focus:outline-2 focus:outline-offset-2 focus:outline-indigo-600 dark:hover:text-white dark:focus:outline-indigo-500 group"
                    title="Add Cursor Agent Task"
                  >
                    <CursorArrowRaysIcon className="w-6 h-6" />
                    <span className="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 bg-gray-900 text-white text-xs rounded whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50">
                      Cursor Agent
                    </span>
                  </button>
                  
                  <button
                    onClick={() => handleAddNode(createTaskNode('user_input'))}
                    className="relative shrink-0 rounded-full p-2 text-gray-400 hover:text-gray-500 focus:outline focus:outline-2 focus:outline-offset-2 focus:outline-indigo-600 dark:hover:text-white dark:focus:outline-indigo-500 group"
                    title="Add User Input Task"
                  >
                    <Bars3Icon className="w-6 h-6" />
                    <span className="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 bg-gray-900 text-white text-xs rounded whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50">
                      User Input
                    </span>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </nav>
      
      {showValidationPanel && (
        <div className="mx-4 mt-4">
          {validationErrors.length > 0 ? (
            <div className="overflow-hidden rounded-lg bg-white shadow dark:bg-gray-800 border border-red-200 dark:border-red-800">
              <div className="bg-red-50 dark:bg-red-900/20 px-4 py-3 border-b border-red-200 dark:border-red-800">
                <div className="flex items-center justify-between">
                  <div className="flex items-center">
                    <ExclamationTriangleIcon className="h-5 w-5 text-red-500 dark:text-red-400 mr-2" aria-hidden="true" />
                    <h3 className="text-sm font-semibold text-red-800 dark:text-red-200">
                      Validation {hasValidationErrors(validationErrors) ? 'Errors' : 'Warnings'} ({validationErrors.length})
                    </h3>
                  </div>
                  <button
                    onClick={() => setShowValidationPanel(false)}
                    className="rounded-md p-1.5 text-red-500 hover:bg-red-100 dark:hover:bg-red-900/40 transition-colors"
                  >
                    <span className="sr-only">Dismiss</span>
                    <svg className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                      <path d="M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z" />
                    </svg>
                  </button>
                </div>
              </div>
              <ul className="divide-y divide-gray-200 dark:divide-gray-700">
                {(() => {
                  // Group errors by nodeId or workflow-level
                  const grouped = validationErrors.reduce((acc, error) => {
                    const key = error.nodeId || '_workflow';
                    if (!acc[key]) acc[key] = [];
                    acc[key].push(error);
                    return acc;
                  }, {} as Record<string, typeof validationErrors>);

                  return Object.entries(grouped).map(([key, errors]) => {
                    const isWorkflowLevel = key === '_workflow';
                    const node = nodes.find(n => n.id === key);
                    const task = node?.data?.task as WorkflowTask | undefined;
                    const taskName = isWorkflowLevel 
                      ? 'Workflow' 
                      : errors[0].message.match(/Task "([^"]+)":/)?.[1] || task?.name || key;

                    return (
                      <li key={key} className="px-4 py-4 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                        <div className="flex items-start gap-3">
                          <div className="flex-shrink-0 mt-0.5">
                            <div className="h-8 w-8 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
                              <span className="text-xs font-medium text-red-600 dark:text-red-400">
                                {errors.length}
                              </span>
                            </div>
                          </div>
                          <div className="flex-1 min-w-0">
                            <p className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                              {taskName}
                            </p>
                            <div className="mt-2 space-y-1">
                              {errors.map((error, idx) => (
                                <p key={idx} className="text-sm text-red-600 dark:text-red-400 flex items-start">
                                  <span className="mr-2">•</span>
                                  <span className="flex-1">
                                    {error.message.replace(/^Task "[^"]+": /, '')}
                                  </span>
                                </p>
                              ))}
                            </div>
                          </div>
                        </div>
                      </li>
                    );
                  });
                })()}
              </ul>
            </div>
          ) : (
            <div className="rounded-md bg-green-50 dark:bg-green-900/20 p-4 border border-green-200 dark:border-green-800">
              <div className="flex">
                <div className="flex-shrink-0">
                  <CheckCircleIconSolid className="h-5 w-5 text-green-400" aria-hidden="true" />
                </div>
                <div className="ml-3 flex-1">
                  <h3 className="text-sm font-medium text-green-800 dark:text-green-200">
                    Workflow is Valid
                  </h3>
                  <div className="mt-2 text-sm text-green-700 dark:text-green-300">
                    No validation errors found. Your workflow is ready to save and execute.
                  </div>
                </div>
                <div className="ml-auto pl-3">
                  <div className="-mx-1.5 -my-1.5">
                    <button
                      onClick={() => setShowValidationPanel(false)}
                      className="inline-flex rounded-md bg-green-50 dark:bg-green-900/20 p-1.5 text-green-500 hover:bg-green-100 dark:hover:bg-green-900/40"
                    >
                      <span className="sr-only">Dismiss</span>
                      <svg className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                        <path d="M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z" />
                      </svg>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      )}
      
      <div className="flex-1 min-h-0">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onNodeClick={handleNodeClick}
          onNodeDoubleClick={handleNodeDoubleClick}
          onSelectionChange={onSelectionChange}
          edgeTypes={edgeTypes}
          fitView
          attributionPosition="bottom-left"
        >
          <Background variant={BackgroundVariant.Dots} gap={16} size={1} />
          <Controls />
        </ReactFlow>
      </div>
      
      <NodeEditorModal 
        isOpen={isModalOpen}
        node={editingNode}
        onSave={handleSaveNode}
        onClose={() => setIsModalOpen(false)}
      />
    </div>
  );
};

export default WorkflowEditor;