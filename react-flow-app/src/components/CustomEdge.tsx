import React from 'react';
import { BaseEdge, EdgeLabelRenderer, getSmoothStepPath, EdgeProps, useReactFlow } from '@xyflow/react';
import { XMarkIcon } from '@heroicons/react/24/solid';

export const CustomEdge: React.FC<EdgeProps> = ({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  style = {},
  markerEnd,
}) => {
  const { setEdges } = useReactFlow();
  const [edgePath, labelX, labelY] = getSmoothStepPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  return (
    <>
      <BaseEdge id={id} path={edgePath} style={style} markerEnd={markerEnd} />
      <EdgeLabelRenderer>
        <button
          style={{
            position: 'absolute',
            transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
            pointerEvents: 'all',
          }}
          className="nodrag nopan flex items-center justify-center w-6 h-6 bg-orange-500 hover:bg-orange-600 rounded-full border-2 border-white dark:border-zinc-800 cursor-pointer transition-colors"
          onClick={() => {
            setEdges((es) => es.filter((e) => e.id !== id));
          }}
        >
          <XMarkIcon className="w-4 h-4 text-white" />
        </button>
      </EdgeLabelRenderer>
    </>
  );
};

