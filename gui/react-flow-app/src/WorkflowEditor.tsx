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
import { Workflow, MessageFromParent } from './types';

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

  // Double-click handler for opening node editor
  const handleNodeDoubleClick = useCallback((_event: React.MouseEvent, node: Node) => {
    // Extract only the primitive values we need for the editor
    const task = node.data.task;
    const taskData = task ? {
      id: task.id,
      name: task.name,
      function: {
        name: task.function?.name || 'cli_command',
        input: task.function?.input || {}
      }
    } : null;
    
    // Send to Dioxus parent window - only send primitive, cloneable data
    window.parent.postMessage({
      type: 'NODE_DOUBLE_CLICKED',
      payload: {
        nodeId: node.id,
        nodeName: node.data.task?.name,
        functionName: node.data.task?.function?.name,
        task: taskData
      }
    }, '*');
  }, []);

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
      } else if (event.data.type === 'UPDATE_NODE' && event.data.payload) {
        const { nodeId, name, functionType, command, args, prompt } = event.data.payload;
        
        // Update the node in the nodes array
        setNodes((currentNodes) => 
          currentNodes.map((node) => {
            if (node.id === nodeId) {
              const updatedTask = {
                ...node.data.task,
                name: name,
                function: {
                  name: functionType,
                  input: functionType === 'cli_command' 
                    ? { command, args }
                    : { prompt }
                }
              };
              
              return {
                ...node,
                data: {
                  ...node.data,
                  task: updatedTask,
                  label: (
                    <div style={{ padding: '10px' }}>
                      <div style={{ fontWeight: 'bold', marginBottom: '5px' }}>{name}</div>
                      <div style={{ fontSize: '12px', color: '#666' }}>
                        {functionType}
                      </div>
                    </div>
                  )
                }
              };
            }
            return node;
          })
        );
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
      <div style={{ 
        display: 'flex', 
        justifyContent: 'center', 
        alignItems: 'center', 
        height: '100%',
        flexDirection: 'column',
        gap: '20px'
      }}>
        <div style={{
          width: '40px',
          height: '40px',
          border: '3px solid #e5e7eb',
          borderTopColor: '#3b82f6',
          borderRadius: '50%',
          animation: 'spin 1s linear infinite',
        }} />
        <div>Loading workflow editor...</div>
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
        
        <Panel position="top-left" style={{
          background: 'white',
          padding: '12px 16px',
          borderRadius: '8px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
          fontFamily: 'system-ui, -apple-system, sans-serif',
        }}>
          <div style={{ marginBottom: '8px' }}>
            <input
              type="text"
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
              style={{
                fontWeight: 'bold',
                fontSize: '16px',
                border: 'none',
                background: 'transparent',
                outline: 'none',
                borderBottom: '2px solid transparent',
                padding: '2px 4px',
                marginBottom: '4px',
                width: '100%',
                minWidth: '200px'
              }}
              onFocus={(e) => {
                e.target.style.borderBottomColor = '#3b82f6';
              }}
              onBlur={(e) => {
                e.target.style.borderBottomColor = 'transparent';
              }}
              placeholder="Workflow Name"
            />
          </div>
          <div style={{ fontSize: '14px', color: '#666' }}>
            {workflow?.tasks.length || 0} task{(workflow?.tasks.length || 0) !== 1 ? 's' : ''}
          </div>
        </Panel>
      </ReactFlow>
    </div>
  );
};

export default WorkflowEditor;