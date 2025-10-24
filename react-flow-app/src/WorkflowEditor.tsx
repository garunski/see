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

const NODE_WIDTH = 250;
const NODE_HEIGHT = 80;
const VERTICAL_SPACING = 150;
const INITIAL_X = 100;
const INITIAL_Y = 100;

const WorkflowEditor: React.FC = () => {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const [workflow, setWorkflow] = useState<Workflow | null>(null);
  const [isLoaded, setIsLoaded] = useState(false);
  const [workflowName, setWorkflowName] = useState<string>('');
  
  // Modal state
  const [editingNode, setEditingNode] = useState<WorkflowTask | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);

  // Convert workflow tasks to React Flow nodes
  const tasksToNodes = useCallback((wf: Workflow): Node[] => {
    const savedPositions = wf.metadata?.node_positions || {};
    
    return wf.tasks.map((task, index) => {
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
  }, []);

  // Generate sequential edges between tasks
  const tasksToEdges = useCallback((wf: Workflow): Edge[] => {
    const edgeList: Edge[] = [];
    
    for (let i = 0; i < wf.tasks.length - 1; i++) {
      edgeList.push({
        id: `edge-${wf.tasks[i].id}-${wf.tasks[i + 1].id}`,
        source: wf.tasks[i].id,
        target: wf.tasks[i + 1].id,
        type: 'smoothstep',
        animated: true,
        style: { stroke: '#3b82f6', strokeWidth: 2 },
      });
    }
    
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
        
        const newNodes = tasksToNodes(wf);
        const newEdges = tasksToEdges(wf);
        
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
            {workflow?.tasks.length || 0} task{(workflow?.tasks.length || 0) !== 1 ? 's' : ''}
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