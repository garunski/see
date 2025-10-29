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
import { CommandLineIcon, CursorArrowRaysIcon, PlayIcon, Bars3Icon } from '@heroicons/react/24/outline';
import { Workflow, MessageFromParent, WorkflowTask } from './types';
import { NodeEditorModal } from './components/NodeEditorModal';
import { Input } from './components/input';
import { CustomEdge } from './components/CustomEdge';
import { useWorkflowNodes } from './hooks/useWorkflowNodes';
import { useWorkflowEdges } from './hooks/useWorkflowEdges';
import { renderNodeLabel } from './utils/nodeRenderer';
import { getLayoutedElements, NODE_WIDTH, START_NODE_ID, START_NODE_SIZE, INITIAL_X, INITIAL_Y } from './utils/layout';

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

  const { tasksToNodes, flattenTasks } = useWorkflowNodes();
  const { tasksToEdges } = useWorkflowEdges();

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

  const handleAddNode = useCallback((taskData: WorkflowTask) => {
    const newNode: Node = {
      id: taskData.id,
      type: 'default',
      position: { x: 300, y: 200 },
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

    setNodes((nds) => [...nds, newNode]);
    
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
                
                <div className="flex items-center gap-1 border-l border-gray-200 dark:border-white/10 pl-4">
                  <button
                    onClick={() => handleAddNode({
                      id: `task_${Date.now()}`,
                      name: 'New CLI Command Task',
                      function: {
                        name: 'cli_command',
                        input: { command: 'echo', args: ['Hello World'] }
                      }
                    })}
                    className="relative shrink-0 rounded-full p-2 text-gray-400 hover:text-gray-500 focus:outline focus:outline-2 focus:outline-offset-2 focus:outline-indigo-600 dark:hover:text-white dark:focus:outline-indigo-500 group"
                    title="Add CLI Command Task"
                  >
                    <CommandLineIcon className="w-6 h-6" />
                    <span className="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 bg-gray-900 text-white text-xs rounded whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50">
                      CLI Command
                    </span>
                  </button>
                  
                  <button
                    onClick={() => handleAddNode({
                      id: `task_${Date.now()}`,
                      name: 'New Cursor Agent Task',
                      function: {
                        name: 'cursor_agent',
                        input: { prompt: 'Enter your prompt here' }
                      }
                    })}
                    className="relative shrink-0 rounded-full p-2 text-gray-400 hover:text-gray-500 focus:outline focus:outline-2 focus:outline-offset-2 focus:outline-indigo-600 dark:hover:text-white dark:focus:outline-indigo-500 group"
                    title="Add Cursor Agent Task"
                  >
                    <CursorArrowRaysIcon className="w-6 h-6" />
                    <span className="absolute top-full mt-2 left-1/2 -translate-x-1/2 px-2 py-1 bg-gray-900 text-white text-xs rounded whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50">
                      Cursor Agent
                    </span>
                  </button>
                  
                  <button
                    onClick={() => handleAddNode({
                      id: `task_${Date.now()}`,
                      name: 'New User Input Task',
                      function: {
                        name: 'user_input',
                        input: { 
                          prompt: 'Enter prompt for user',
                          input_type: 'text',
                          required: true
                        }
                      }
                    })}
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
      
      <div className="flex-1 min-h-0">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onNodeClick={handleNodeClick}
          onNodeDoubleClick={handleNodeDoubleClick}
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