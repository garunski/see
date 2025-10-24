import React, { useCallback, useEffect, useState } from 'react';
import ReactFlow, {
  Node,
  Edge,
  Controls,
  Background,
  BackgroundVariant,
  useNodesState,
  useEdgesState,
  addEdge,
  Connection,
  NodeChange,
  EdgeChange,
  Panel,
} from 'reactflow';
import 'reactflow/dist/style.css';
import { Workflow, MessageFromParent, MessageToParent } from './types';

const NODE_WIDTH = 250;
const NODE_HEIGHT = 80;
const VERTICAL_SPACING = 150;
const INITIAL_X = 100;
const INITIAL_Y = 50;

interface WorkflowVisualizerProps {}

const WorkflowVisualizer: React.FC<WorkflowVisualizerProps> = () => {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const [workflow, setWorkflow] = useState<Workflow | null>(null);
  const [isLoaded, setIsLoaded] = useState(false);

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
        },
        style: {
          background: '#fff',
          border: '2px solid #3b82f6',
          borderRadius: '8px',
          width: NODE_WIDTH,
          minHeight: NODE_HEIGHT,
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

  // Save workflow with updated node positions
  const saveWorkflow = useCallback(() => {
    if (!workflow) return;

    const nodePositions: Record<string, { x: number; y: number }> = {};
    nodes.forEach((node) => {
      nodePositions[node.id] = { x: node.position.x, y: node.position.y };
    });

    const updatedWorkflow: Workflow = {
      ...workflow,
      metadata: {
        ...workflow.metadata,
        node_positions: nodePositions,
      },
    };

    const message: MessageToParent = {
      type: 'SAVE_WORKFLOW',
      payload: { workflow: updatedWorkflow },
    };

    window.parent.postMessage(message, '*');
  }, [workflow, nodes]);

  // Debounced save on node changes
  useEffect(() => {
    if (!isLoaded || !workflow) return;

    const timer = setTimeout(() => {
      saveWorkflow();
    }, 1000);

    return () => clearTimeout(timer);
  }, [nodes, isLoaded, workflow, saveWorkflow]);

  // Listen for messages from parent window (Dioxus)
  useEffect(() => {
    const handleMessage = (event: MessageEvent<MessageFromParent>) => {
      if (event.data.type === 'LOAD_WORKFLOW' && event.data.payload?.workflow) {
        const wf = event.data.payload.workflow;
        setWorkflow(wf);
        
        const newNodes = tasksToNodes(wf);
        const newEdges = tasksToEdges(wf);
        
        setNodes(newNodes);
        setEdges(newEdges);
        setIsLoaded(true);
      }
    };

    window.addEventListener('message', handleMessage);
    
    // Signal to parent that we're ready
    window.parent.postMessage({ type: 'READY' }, '*');

    return () => {
      window.removeEventListener('message', handleMessage);
    };
  }, [tasksToNodes, tasksToEdges, setNodes, setEdges]);

  const onConnect = useCallback(
    (connection: Connection) => {
      setEdges((eds) => addEdge(connection, eds));
    },
    [setEdges]
  );

  const handleNodesChange = useCallback(
    (changes: NodeChange[]) => {
      onNodesChange(changes);
    },
    [onNodesChange]
  );

  const handleEdgesChange = useCallback(
    (changes: EdgeChange[]) => {
      onEdgesChange(changes);
    },
    [onEdgesChange]
  );

  if (!workflow) {
    return (
      <div className="flex items-center justify-center h-screen font-sans text-zinc-500 dark:text-zinc-400">
        <div>
          <div className="w-10 h-10 border-3 border-zinc-200 border-t-blue-500 rounded-full animate-spin mx-auto mb-4" />
          <div>Loading workflow...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="w-full h-full min-h-0">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={handleNodesChange}
        onEdgesChange={handleEdgesChange}
        onConnect={onConnect}
        fitView
        attributionPosition="bottom-left"
      >
        <Background variant={BackgroundVariant.Dots} gap={16} size={1} />
        <Controls />
        <Panel position="top-left" className="bg-white dark:bg-zinc-800 p-3 rounded-lg shadow-sm font-sans">
          <div className="font-bold text-base mb-1 text-zinc-900 dark:text-white">
            {workflow.name}
          </div>
          <div className="text-sm text-zinc-500 dark:text-zinc-400">
            {workflow.tasks.length} task{workflow.tasks.length !== 1 ? 's' : ''}
          </div>
        </Panel>
      </ReactFlow>
    </div>
  );
};

export default WorkflowVisualizer;

