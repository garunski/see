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
            <div style={{ padding: '10px' }}>
              <div style={{ fontWeight: 'bold', marginBottom: '5px' }}>{task.name}</div>
              <div style={{ fontSize: '12px', color: '#666' }}>
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
      <div style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        height: '100vh',
        fontFamily: 'system-ui, -apple-system, sans-serif',
        color: '#666',
      }}>
        <div>
          <div style={{
            width: '40px',
            height: '40px',
            border: '3px solid #e5e7eb',
            borderTopColor: '#3b82f6',
            borderRadius: '50%',
            animation: 'spin 1s linear infinite',
            margin: '0 auto 16px',
          }} />
          <div>Loading workflow...</div>
        </div>
        <style>{`
          @keyframes spin {
            to { transform: rotate(360deg); }
          }
        `}</style>
      </div>
    );
  }

  return (
    <div style={{ width: '100vw', height: '100vh' }}>
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
        <Panel position="top-left" style={{
          background: 'white',
          padding: '12px 16px',
          borderRadius: '8px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
          fontFamily: 'system-ui, -apple-system, sans-serif',
        }}>
          <div style={{ fontWeight: 'bold', fontSize: '16px', marginBottom: '4px' }}>
            {workflow.name}
          </div>
          <div style={{ fontSize: '14px', color: '#666' }}>
            {workflow.tasks.length} task{workflow.tasks.length !== 1 ? 's' : ''}
          </div>
        </Panel>
      </ReactFlow>
    </div>
  );
};

export default WorkflowVisualizer;

