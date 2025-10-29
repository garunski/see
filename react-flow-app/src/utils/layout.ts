import { Node, Edge } from '@xyflow/react';
import dagre from '@dagrejs/dagre';

export const NODE_WIDTH = 250;
export const NODE_HEIGHT = 80;
export const VERTICAL_SPACING = 150;
export const INITIAL_X = 100 + NODE_WIDTH / 2; // Center the start node
export const INITIAL_Y = 100;
export const START_NODE_ID = '__start__';
export const START_NODE_SIZE = 30;

export const getLayoutedElements = (nodes: Node[], edges: Edge[], direction = 'TB') => {
  const dagreGraph = new dagre.graphlib.Graph();
  dagreGraph.setDefaultEdgeLabel(() => ({}));
  
  dagreGraph.setGraph({ 
    rankdir: direction,
    nodesep: 50,
    ranksep: 150
  });
  
  // Add nodes to dagre
  nodes.forEach((node) => {
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

