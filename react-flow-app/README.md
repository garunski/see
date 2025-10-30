# Workflow Visualizer (React Flow)

This is a React-based workflow visualization component that uses [React Flow](https://reactflow.dev) to render interactive workflow diagrams.

## Overview

The visualizer displays workflow tasks as nodes and automatically connects them in sequential order. It's embedded in the Dioxus Desktop app via an iframe and communicates with the Rust backend using `postMessage` API.

## Features

- **Visual workflow display**: Tasks rendered as interactive nodes
- **Sequential auto-layout**: Tasks automatically connected in order
- **Drag & pan**: Move nodes and navigate the canvas
- **Zoom controls**: Built-in zoom controls
- **Position persistence**: Node positions saved to workflow metadata
- **Auto-save**: Changes debounced and saved automatically

## Development

### Install dependencies

```bash
npm install
```

### Run development server

```bash
npm run dev
```

### Build for production

```bash
npm run build
```

This builds to `../assets/workflow-visualizer/` which is served by the Dioxus app.

## Architecture

### Data Flow

1. **Dioxus → React**: Parent window sends workflow JSON via `postMessage`
2. **React rendering**: Workflow converted to React Flow nodes and edges
3. **User interaction**: Drag nodes, zoom, pan
4. **React → Dioxus**: Updated workflow sent back via `postMessage` (debounced)

### Message Protocol

**Load Workflow** (Dioxus → React):

```typescript
{
  type: 'LOAD_WORKFLOW',
  payload: {
    workflow: {
      id: string,
      name: string,
      tasks: [...],
      metadata?: {
        node_positions?: { [taskId: string]: { x: number, y: number } }
      }
    }
  }
}
```

**Save Workflow** (React → Dioxus):

```typescript
{
  type: 'SAVE_WORKFLOW',
  payload: {
    workflow: { /* updated workflow with new node positions */ }
  }
}
```

## File Structure

- `src/types.ts` - TypeScript type definitions
- `src/WorkflowVisualizer.tsx` - Main React Flow component
- `src/App.tsx` - App wrapper
- `src/main.tsx` - Entry point
- `vite.config.ts` - Vite build configuration

## Integration with Dioxus

The built assets are loaded by the Dioxus Desktop app from:

```
/assets/workflow-visualizer/index.html
```

The Dioxus page is at: `gui/src/pages/workflow/visualizer/mod.rs`

## Customization

### Node styling

Edit the `style` property in `tasksToNodes()` function in `WorkflowVisualizer.tsx`.

### Layout algorithm

Modify `VERTICAL_SPACING`, `NODE_WIDTH`, etc. constants, or implement a custom layout algorithm using libraries like Dagre or ELK.js.

### Edge styling

Edit the edge configuration in `tasksToEdges()` function.
