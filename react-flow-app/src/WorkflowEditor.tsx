import React, { useState, useCallback, useEffect } from 'react';
import ReactFlow, {
  Controls,
  Background,
  useNodesState,
  useEdgesState,
  addEdge,
  Connection,
  Node,
  Edge,
  Panel,
  BackgroundVariant,
} from 'reactflow';
import 'reactflow/dist/style.css';
import { Workflow, MessageFromParent, WorkflowTask } from './types';
import { NodeEditorModal } from './components/NodeEditorModal';
import { Input } from './components/input';
import dagre from '@dagrejs/dagre';

const NODE_WIDTH = 250;
const NODE_HEIGHT = 80;
const VERTICAL_SPACING = 150;
const INITIAL_X = 100;
const INITIAL_Y = 100;
const START_NODE_ID = '__start__';
const START_NODE_SIZE = 30;

// Dagre layout function
const getLayoutedElements = (nodes: Node[], edges: Edge[], direction = 'TB') => {
  const dagreGraph = new dagre.graphlib.Graph();
  dagreGraph.setDefaultEdgeLabel(() => ({}));
  
  dagreGraph.setGraph({ 
    rankdir: direction,
    nodesep: 50,
    ranksep: 150
  });
  
  // Add nodes to dagre
  nodes.forEach((node) => {
    // Use per-node styled dimensions when available (for visual-only start node)
    const width = (node.style as any)?.width ?? (node.id === START_NODE_ID ? START_NODE_SIZE : NODE_WIDTH);
    const height = (node.style as any)?.minHeight ?? (node.id === START_NODE_ID ? START_NODE_SIZE : NODE_HEIGHT);
    dagreGraph.setNode(node.id, { width, height });
  });
  
  // Add edges to dagre
  edges.forEach((edge) => {
    dagreGraph.setEdge(edge.source, edge.target);
  });
  
  // Calculate layout
  dagre.layout(dagreGraph);
  
  // Apply calculated positions
  const layoutedNodes = nodes.map((node) => {
    const nodeWithPosition = dagreGraph.node(node.id);
    return {
      ...node,
      position: {
        x: nodeWithPosition.x - NODE_WIDTH / 2,
        y: nodeWithPosition.y - NODE_HEIGHT / 2,
      },
    };
  });
  
  return { nodes: layoutedNodes, edges };
};

const WorkflowEditor: React.FC = () => {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const [workflow, setWorkflow] = useState<Workflow | null>(null);
  const [isLoaded, setIsLoaded] = useState(false);
  const [workflowName, setWorkflowName] = useState<string>('');
  
  // Modal state
  const [editingNode, setEditingNode] = useState<WorkflowTask | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);

  // Recursively flatten tasks from next_tasks tree
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

  // Convert workflow tasks to React Flow nodes
  const tasksToNodes = useCallback((wf: Workflow): Node[] => {
    const savedPositions = wf.metadata?.node_positions || {};
    const allTasks = flattenTasks(wf.tasks);
    
    return allTasks.map((task, index) => {
      const savedPos = savedPositions[task.id];
      // Use saved position if available, otherwise use placeholder (will be replaced by Dagre)
      const position = savedPos || {
        x: INITIAL_X,
        y: INITIAL_Y + index * (NODE_HEIGHT + VERTICAL_SPACING),
      };

      return {
        id: task.id,
        type: 'default',
        position,
        data: {
          label: (
            <div className="p-2.5">
              <div className="font-bold mb-1.5 text-zinc-900 dark:text-white">{task.name}</div>
              <div className="text-xs text-zinc-500 dark:text-zinc-400">
                {task.function.name}
              </div>
            </div>
          ),
          task: task,
        },
        style: {
          background: '#fff',
          border: '2px solid #3b82f6',
          borderRadius: '8px',
          width: NODE_WIDTH,
          minHeight: NODE_HEIGHT,
          boxShadow: '0 2px 8px rgba(0,0,0,0.1)',
        },
      };
    });
  }, [flattenTasks]);

  // Generate edges from next_tasks relationships
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
            // Recursively generate edges for nested tasks
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

  // Click handler with postMessage to Dioxus parent
  const handleNodeClick = useCallback((_event: React.MouseEvent, node: Node) => {
    // Send to Dioxus parent window - only send cloneable data
    window.parent.postMessage({
      type: 'NODE_CLICKED',
      payload: {
        nodeId: node.id,
        nodeName: node.data.task?.name,
        functionName: node.data.task?.function?.name
      }
    }, '*');
  }, []);

  // Double-click handler for opening node editor modal
  const handleNodeDoubleClick = useCallback((_event: React.MouseEvent, node: Node) => {
    setEditingNode(node.data.task);
    setIsModalOpen(true);
  }, []);

  // Save handler for modal
  const handleSaveNode = useCallback((updatedNode: WorkflowTask) => {
    setNodes((currentNodes) => 
      currentNodes.map((node) => {
        if (node.id === updatedNode.id) {
          return {
            ...node,
            data: {
              ...node.data,
              task: updatedNode,
              label: (
                <div className="p-2.5">
                  <div className="font-bold mb-1.5 text-zinc-900 dark:text-white">{updatedNode.name}</div>
                  <div className="text-xs text-zinc-500 dark:text-zinc-400">{updatedNode.function.name}</div>
                </div>
              )
            }
          }
        }
        return node;
      })
    );
  }, [setNodes]);

  // Listen for messages from parent window (Dioxus)
  useEffect(() => {
    const handleMessage = (event: MessageEvent<MessageFromParent>) => {
      if (event.data.type === 'LOAD_WORKFLOW' && event.data.payload?.workflow) {
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
              <div
                style={{
                  width: START_NODE_SIZE,
                  height: START_NODE_SIZE,
                  borderRadius: START_NODE_SIZE / 2,
                  background: '#10b981',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  color: 'white',
                  fontSize: 10,
                }}
                title="Start"
              >
                ●
              </div>
            ),
          },
          style: {
            width: START_NODE_SIZE,
            minHeight: START_NODE_SIZE,
            border: '2px solid #10b981',
            borderRadius: START_NODE_SIZE / 2,
            background: 'white',
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
  }, [tasksToNodes, tasksToEdges, setNodes, setEdges]);

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
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
    <div className="w-full h-full min-h-0">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onNodeClick={handleNodeClick}
        onNodeDoubleClick={handleNodeDoubleClick}
        fitView
        attributionPosition="bottom-left"
      >
        <Background variant={BackgroundVariant.Dots} gap={16} size={1} />
        <Controls />
        
        <Panel position="top-left" className="bg-white dark:bg-zinc-800 p-3 rounded-lg shadow-sm font-sans">
          <div className="mb-2">
            <Input
              value={workflowName}
              onChange={(e) => {
                const newName = e.target.value;
                setWorkflowName(newName);
                // Send name change to parent window
                window.parent.postMessage({
                  type: 'WORKFLOW_NAME_CHANGED',
                  payload: { name: newName }
                }, '*');
              }}
              placeholder="Workflow Name"
            />
          </div>
          <div className="text-sm text-zinc-500 dark:text-zinc-400">
            {workflow ? flattenTasks(workflow.tasks).length : 0} task{(workflow ? flattenTasks(workflow.tasks).length : 0) !== 1 ? 's' : ''}
          </div>
        </Panel>
      </ReactFlow>
      
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