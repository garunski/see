# GUI Routes

This document lists all the routes and pages in the GUI application.

## Routes Overview

### Home
- **Route:** `/`
- **Component:** `HomePage`
- **Description:** Main landing page

### Workflows

#### Workflow List
- **Route:** `/workflows`
- **Component:** `WorkflowsListPage`
- **Description:** List of all workflows

#### Upload Workflow
- **Route:** `/workflows/upload`
- **Component:** `UploadPage`
- **Description:** Upload a new workflow file

#### Create New Workflow
- **Route:** `/workflows/new`
- **Component:** `WorkflowEditPageNew`
- **Description:** Create a new workflow from scratch

#### Edit Workflow
- **Route:** `/workflows/edit/:id`
- **Component:** `WorkflowEditPage`
- **Parameters:** `id` (String) - Workflow ID
- **Description:** Edit an existing workflow

#### Visualize Workflow
- **Route:** `/workflows/visualize/:id`
- **Component:** `WorkflowVisualizerPage`
- **Parameters:** `id` (String) - Workflow ID
- **Description:** Visual representation of a workflow

### Executions

#### Execution History
- **Route:** `/executions/history`
- **Component:** `HistoryPage`
- **Description:** List of execution history

#### Execution Details
- **Route:** `/executions/details/:id`
- **Component:** `WorkflowDetailsPage`
- **Parameters:** `id` (String) - Execution ID
- **Description:** Detailed view of a specific execution

### Prompts

#### Prompts List
- **Route:** `/prompts`
- **Component:** `UserPromptsListPage`
- **Description:** List of all user prompts

#### Create New Prompt
- **Route:** `/prompts/new`
- **Component:** `UserPromptEditPageNew`
- **Description:** Create a new user prompt

#### Edit Prompt
- **Route:** `/prompts/edit/:id`
- **Component:** `UserPromptEditPage`
- **Parameters:** `id` (String) - Prompt ID
- **Description:** Edit an existing user prompt

### Settings

#### Settings Page
- **Route:** `/settings`
- **Component:** `SettingsPage`
- **Description:** Application settings

### Error Handling

#### Page Not Found
- **Route:** `/:..route`
- **Component:** `PageNotFound`
- **Parameters:** `route` (Vec<String>) - Unmatched route segments
- **Description:** 404 page for unmatched routes

## Page Components by Directory

### Home (`pages/home/`)
- `HomePage` - Main page with action cards

### Workflows (`pages/workflows/`)
- `WorkflowsListPage` - Workflows list
- `UploadPage` - Upload new workflow
- `WorkflowEditPage` - Edit workflow (new)
- `WorkflowEditPageNew` - Edit workflow (existing)
- `WorkflowVisualizerPage` - Visualize workflow

### Executions (`pages/executions/`)
- `HistoryPage` - Execution history list
- `WorkflowDetailsPage` - Execution details

### Prompts (`pages/prompts/`)
- `UserPromptsListPage` - Prompts list
- `UserPromptEditPageNew` - Create new prompt
- `UserPromptEditPage` - Edit existing prompt

### Settings (`pages/settings/`)
- `SettingsPage` - Application settings

