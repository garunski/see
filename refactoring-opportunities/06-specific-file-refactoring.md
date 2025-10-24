# Specific File Refactoring Targets

## Current State
- **Files**: Multiple large files identified by quality check
- **Priority**: 🎯 MIXED - Various priority levels based on impact

## High Priority Files (Immediate Action Required)

### 1. `workflow/edit/page.rs` (317 lines) - CRITICAL
**Status**: Already covered in [01-workflow-edit-page-refactoring.md](./01-workflow-edit-page-refactoring.md)

**Action**: Split into 6-8 focused components
- Extract visual editor component
- Extract JSON editor component  
- Extract validation logic
- Extract state management hooks

### 2. `prompts/edit/page.rs` (202 lines) - HIGH
**Problems**:
- Large form component with mixed concerns
- Duplicate validation patterns
- Complex state management

**Refactoring Plan**:
```
prompts/edit/
├── page.rs                    // Main orchestrator (50 lines)
├── components/
│   ├── mod.rs
│   ├── prompt_form.rs         // Form component
│   ├── form_fields.rs         // Individual field components
│   └── form_actions.rs        // Save/Cancel buttons
├── hooks/
│   ├── mod.rs
│   ├── use_prompt_edit.rs     // Edit state management
│   └── use_prompt_validation.rs // Validation logic
└── types.rs                   // Form data types
```

**Extract Components**:
- `PromptForm` - Main form wrapper
- `PromptIdField` - ID input with validation
- `PromptDescriptionField` - Description input
- `PromptContentField` - Content textarea
- `FormActions` - Save/Cancel button group

**Extract Hooks**:
- `use_prompt_edit` - Centralized edit state
- `use_prompt_validation` - Form validation logic

### 3. `layout/app.rs` (160 lines) - HIGH
**Status**: Already covered in [04-app-layout-refactoring.md](./04-app-layout-refactoring.md)

**Action**: Split into focused layout components
- Extract settings loading logic
- Extract error boundary component
- Extract loading screen component
- Simplify main app structure

## Medium Priority Files

### 4. `workflow/details/hooks.rs` (100 lines) - MEDIUM
**Problems**:
- Complex polling logic mixed with state management
- Long functions with multiple responsibilities
- Hard to test individual pieces

**Refactoring Plan**:
```
workflow/details/hooks/
├── mod.rs
├── use_workflow_execution.rs  // Main execution hook
├── use_polling.rs             // Polling logic
├── use_task_navigation.rs     // Task navigation
├── use_audit_filtering.rs     // Audit trail filtering
└── types.rs                   // Hook state types
```

**Extract Hooks**:
- `use_polling` - Generic polling mechanism
- `use_execution_state` - Execution state management
- `use_task_navigation` - Task navigation logic
- `use_audit_filtering` - Audit trail filtering

### 5. `settings/page.rs` (175 lines) - MEDIUM
**Problems**:
- Theme selection logic could be extracted
- Data management section is complex
- Mixed UI and business logic

**Refactoring Plan**:
```
settings/
├── page.rs                    // Main page (50 lines)
├── components/
│   ├── mod.rs
│   ├── theme_selector.rs      // Theme selection component
│   ├── data_management.rs     // Data management section
│   └── confirmation_dialog.rs // Clear data confirmation
└── hooks/
    ├── mod.rs
    ├── use_theme_management.rs // Theme state management
    └── use_data_management.rs  // Data management logic
```

**Extract Components**:
- `ThemeSelector` - Theme selection with preview
- `DataManagement` - Clear data section
- `ConfirmationDialog` - Reusable confirmation dialog

### 6. `workflow/upload/page.rs` (141 lines) - MEDIUM
**Problems**:
- File upload logic mixed with UI
- Complex error handling
- Could benefit from reusable file input component

**Refactoring Plan**:
```
workflow/upload/
├── page.rs                    // Main page (50 lines)
├── components/
│   ├── mod.rs
│   ├── file_upload.rs         // File upload component
│   ├── file_preview.rs        // Selected file preview
│   └── upload_actions.rs      // Upload button and actions
└── hooks/
    ├── mod.rs
    ├── use_file_upload.rs     // File upload state
    └── use_workflow_parsing.rs // Workflow parsing logic
```

## Low Priority Files

### 7. `workflow/visualizer/page.rs` (158 lines) - LOW
**Problems**:
- Iframe setup logic could be extracted
- Error handling could be standardized
- Loading states could be componentized

**Refactoring Plan**:
```
workflow/visualizer/
├── page.rs                    // Main page (80 lines)
├── components/
│   ├── mod.rs
│   ├── iframe_viewer.rs       // Iframe wrapper component
│   ├── loading_state.rs       // Loading component
│   └── error_state.rs         // Error display
└── hooks/
    ├── mod.rs
    └── use_workflow_loading.rs // Workflow loading logic
```

### 8. `workflow/list/page.rs` (102 lines) - LOW
**Problems**:
- Table rendering could use generic table component
- Empty state could be componentized
- Action buttons could be extracted

**Refactoring Plan**:
```
workflow/list/
├── page.rs                    // Main page (50 lines)
├── components/
│   ├── mod.rs
│   ├── workflow_table.rs      // Table component
│   ├── workflow_row.rs        // Table row component
│   └── empty_state.rs         // Empty state component
└── hooks/
    ├── mod.rs
    └── use_workflow_list.rs   // List state management
```

## Implementation Priority Matrix

| File | Lines | Priority | Impact | Effort | ROI |
|------|-------|----------|--------|--------|-----|
| workflow/edit/page.rs | 317 | 🚨 CRITICAL | High | High | High |
| prompts/edit/page.rs | 202 | 🔄 HIGH | High | Medium | High |
| layout/app.rs | 160 | 🔄 HIGH | High | Medium | High |
| workflow/details/hooks.rs | 100 | 📊 MEDIUM | Medium | Low | High |
| settings/page.rs | 175 | 📊 MEDIUM | Medium | Medium | Medium |
| workflow/upload/page.rs | 141 | 📊 MEDIUM | Medium | Low | Medium |
| workflow/visualizer/page.rs | 158 | 📱 LOW | Low | Low | Low |
| workflow/list/page.rs | 102 | 📱 LOW | Low | Low | Low |

## Recommended Implementation Order

### Phase 1: Critical Issues (Week 1)
1. **WorkflowEditPage** - Split into focused components
2. **Common Hooks** - Extract shared patterns
3. **App Layout** - Simplify main structure

### Phase 2: High Impact (Week 2)
4. **PromptEditPage** - Extract form components
5. **UI Component Library** - Create reusable components
6. **Service Layer** - Standardize API patterns

### Phase 3: Medium Impact (Week 3)
7. **Workflow Details Hooks** - Extract polling logic
8. **Settings Page** - Extract theme selection
9. **Upload Page** - Extract file upload component

### Phase 4: Polish (Week 4)
10. **Visualizer Page** - Extract iframe component
11. **List Page** - Extract table component
12. **Testing** - Add comprehensive tests

## Success Metrics

### Phase 1 Success
- WorkflowEditPage < 100 lines
- Common hooks extracted and working
- App structure simplified

### Phase 2 Success
- All edit pages using shared components
- UI consistency improved
- Service layer standardized

### Phase 3 Success
- All large files < 150 lines
- Reusable components created
- Better separation of concerns

### Phase 4 Success
- All files < 100 lines
- Comprehensive test coverage
- Documentation updated

## Risk Mitigation

1. **Incremental Changes** - Refactor one file at a time
2. **Thorough Testing** - Test after each refactoring
3. **Backup Strategy** - Keep original files until confirmed working
4. **User Testing** - Verify functionality after each change
5. **Performance Monitoring** - Ensure no performance regression
