# Workflow Visualizer Implementation

## Overview

A React Flow-based interactive workflow visualizer has been integrated into the Dioxus Desktop app. Users can view and interact with workflow diagrams through a dedicated visualizer page accessible from workflow lists and edit pages.

## Implementation Summary

### 1. React Flow Application

**Location**: `react-flow-app/`

A standalone React + TypeScript + Vite application that:
- Displays workflows as interactive node graphs
- Auto-connects tasks in sequential order (task[0] → task[1] → task[2])
- Supports drag, zoom, and pan interactions
- Persists node positions in workflow metadata
- Communicates with Dioxus via `postMessage` API

**Key Files**:
- `src/WorkflowEditor.tsx` - Main editor component with full editing capabilities
- `src/types.ts` - TypeScript definitions matching Rust structs
- `src/components/AddNodeButton.tsx` - Floating action button for creating new tasks
- `vite.config.ts` - Builds to `../assets/workflow-visualizer/`

**Build Output**: `gui/assets/workflow-visualizer/index.html` (and bundled JS/CSS)

**Features**:
- ✅ Interactive node graph with drag, zoom, and pan
- ✅ Add new tasks via floating "+" button
- ✅ Create connections by dragging between tasks
- ✅ Edit task properties via double-click
- ✅ Auto-layout with Dagre algorithm
- ✅ Visual quick guide for new users
- ✅ Animated edge connections
- ✅ Persistent node positions in metadata

### 2. Rust Workflow Schema Extensions

**Location**: `core/src/persistence/models.rs`

Added support for visual metadata in workflow JSON:

```rust
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

pub struct WorkflowVisualizationMetadata {
    pub node_positions: Option<HashMap<String, NodePosition>>,
}

pub struct WorkflowJson {
    pub id: String,
    pub name: String,
    pub tasks: Vec<serde_json::Value>,
    pub metadata: Option<WorkflowVisualizationMetadata>,
}
```

**Note**: The `metadata` field is optional with `#[serde(skip_serializing_if = "Option::is_none")]`, so workflows without metadata continue to work.

### 3. Dioxus Visualizer Page

**Location**: `gui/src/pages/workflow/visualizer/mod.rs`

A full-screen page that:
- Loads workflow by ID from settings/database
- Embeds React Flow app in an iframe
- Sends workflow JSON to iframe via `postMessage`
- Displays loading states and error messages
- Provides "Back" navigation button

**Route**: `/workflows/visualize/:id`

### 4. UI Integration Points

#### Workflows List Page
**Location**: `gui/src/pages/settings/components/workflows_list.rs`

Added "Visualize" link in the Actions column for each workflow.

#### Workflow Edit Page
**Location**: `gui/src/pages/settings/components/workflow_edit.rs`

Added "Visualize" button in the header toolbar when editing existing workflows (not shown for new workflows).

### 5. Router Integration

**Location**: `gui/src/router.rs`

Added route:
```rust
#[route("/workflows/visualize/:id")]
WorkflowVisualizerPage { id: String },
```

### 6. Build Configuration

**Location**: `Taskfile.yml`

Added tasks:
- `build-visualizer` - Builds React Flow app
- Updated `run-gui`, `serve-gui`, `build-release` to depend on visualizer build

## Usage

### View a Workflow

1. Navigate to "Workflows" page
2. Click "Visualize" next to any workflow
3. View interactive diagram with zoom/pan controls

### Add New Tasks

1. Click the blue **+** button in the bottom-right corner
2. Select task type:
   - **📝 CLI Command** - Execute shell commands
   - **🤖 Cursor Agent** - AI-powered automation tasks
3. The new task appears on the canvas and can be repositioned
4. Double-click the task to edit its properties

### Create Task Connections

1. Hover over a task node - you'll see connection handles on the edges
2. Click and drag from one task's handle to another task
3. A connection line (edge) appears showing workflow direction
4. Edges are animated and styled for visual clarity

### Edit Node Positions

1. Drag nodes to reposition them
2. Changes auto-save after 1 second debounce
3. Positions persist in workflow metadata

### Edit Task Details

1. Double-click any task node
2. Edit task name, function type, and parameters
3. Save changes to update the task

### Development Workflow

```bash
# Build React Flow app
cd react-flow-app && npm run build

# Or use Task
task build-visualizer

# Run GUI
task serve-gui
```

## Architecture

### Communication Flow

```
┌─────────────────┐                           ┌──────────────────┐
│  Dioxus Page    │  1. Load workflow from DB │  React Flow App  │
│  (Rust/Dioxus)  │──────────────────────────>│  (TypeScript)    │
│                 │                           │                  │
│  iframe parent  │  2. postMessage           │  iframe content  │
│  window         │     LOAD_WORKFLOW         │  window          │
│                 │──────────────────────────>│                  │
│                 │                           │                  │
│                 │  3. User drags nodes      │  User interacts  │
│                 │                           │  with diagram    │
│                 │                           │                  │
│                 │  4. postMessage           │                  │
│                 │<──────────────────────────│  Auto-save       │
│                 │     SAVE_WORKFLOW         │  (debounced)     │
│                 │     (not yet implemented) │                  │
└─────────────────┘                           └──────────────────┘
```

### Workflow JSON Format

```json
{
  "id": "my_workflow",
  "name": "My Workflow",
  "tasks": [
    {
      "id": "task_1",
      "name": "First Task",
      "function": { "name": "cli_command", "input": {...} }
    },
    {
      "id": "task_2",
      "name": "Second Task",
      "function": { "name": "cli_command", "input": {...} }
    }
  ],
  "metadata": {
    "node_positions": {
      "task_1": { "x": 100, "y": 50 },
      "task_2": { "x": 100, "y": 250 }
    }
  }
}
```

### Node Layout Strategy

- **Initial layout**: Vertical arrangement with 150px spacing
- **Preserved layout**: Node positions loaded from `metadata.node_positions`
- **Auto-edges**: Sequential connections between tasks in array order

## Future Enhancements

### Planned Features

1. **Save functionality**: Implement workflow persistence from React to Rust
2. **Manual connections**: Allow users to create custom edges
3. **Node editing**: Edit task properties from the visualizer
4. **Layout algorithms**: Integrate Dagre or ELK.js for auto-layout
5. **Minimap**: Add minimap for large workflows
6. **Export**: Export diagram as PNG/SVG
7. **Validation**: Visual indicators for invalid workflows

### Known Limitations

1. Save workflow changes not yet implemented (positions saved but not persisted)
2. No validation of workflow structure in visualizer
3. ~~Edge creation/deletion not yet supported~~ ✅ Edge creation now supported - users can drag connections between tasks
4. ~~No node creation/deletion from visualizer~~ ✅ Node creation now supported via the "+" button

## Testing Checklist

- [x] React Flow app builds successfully
- [x] Assets load correctly in Dioxus webview
- [x] Workflow JSON loads and renders as nodes
- [x] Nodes are auto-connected sequentially
- [x] Drag/zoom/pan interactions work
- [x] Add node button displays menu with task types
- [x] New tasks can be added to the workflow
- [x] Users can drag connections between tasks
- [x] Edges are styled and animated
- [x] Double-click opens task editor modal
- [x] Quick guide panel displays usage instructions
- [ ] Saving workflow updates the database (pending implementation)
- [x] Existing workflows without metadata still work
- [x] Navigation to/from visualizer works smoothly

## File Manifest

### New Files
- `react-flow-app/` - Complete React application
  - `package.json` - NPM dependencies
  - `vite.config.ts` - Build configuration
  - `tsconfig.json` - TypeScript configuration
  - `src/WorkflowEditor.tsx` - Main editor component (renamed from WorkflowVisualizer)
  - `src/types.ts` - Type definitions
  - `src/App.tsx`, `src/main.tsx` - Entry points
  - `src/components/AddNodeButton.tsx` - Floating action button for adding tasks
  - `src/components/NodeEditorModal.tsx` - Modal for editing task details
  - `src/components/Toolbar.tsx` - Toolbar with layout and save actions
  - `README.md` - Documentation
- `gui/assets/workflow-visualizer/` - Built React assets
- `gui/src/pages/workflow/visualizer/mod.rs` - Dioxus page

### Modified Files
- `gui/src/router.rs` - Added visualizer route
- `gui/src/pages/workflow/mod.rs` - Export visualizer page
- `core/src/persistence/models.rs` - Added metadata structs
- `core/src/lib.rs` - Export new types
- `gui/src/pages/settings/components/workflows_list.rs` - Visualize button
- `gui/src/pages/settings/components/workflow_edit.rs` - Visualize button
- `Taskfile.yml` - Build tasks

## Dependencies

### Rust
- No new Rust dependencies required

### JavaScript (React Flow App)
- `react@^18.3.1`
- `react-dom@^18.3.1`
- `reactflow@^11.11.4`
- `vite@^5.4.11`
- `typescript@^5.6.3`

## Build & Deployment

### Development
```bash
task serve-gui
```

### Production
```bash
task build-release
```

The visualizer assets are automatically bundled with the Dioxus Desktop app.

## Troubleshooting

### Blank visualizer page
- Ensure React Flow app is built: `task build-visualizer`
- Check browser console for JavaScript errors
- Verify `gui/assets/workflow-visualizer/index.html` exists

### Workflow not loading
- Check workflow ID is valid
- Ensure workflow content is valid JSON
- Check Dioxus logs for parse errors

### Styling issues
- Ensure React Flow CSS is imported in `WorkflowVisualizer.tsx`
- Check that Vite build output includes CSS file
- Verify iframe has proper dimensions

## Credits

- **React Flow**: https://reactflow.dev
- **Dioxus**: https://dioxuslabs.com
- **Vite**: https://vitejs.dev

