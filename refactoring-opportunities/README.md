# Refactoring Opportunities

This directory contains detailed refactoring plans for the Speculative Execution Engine GUI application after the successful pages folder reorganization.

## 🎯 Overview

The pages reorganization was completed successfully, but several refactoring opportunities remain to improve code quality, maintainability, and developer experience.

## 📋 Refactoring Plans

### 🚨 Critical Priority (Immediate Action Required)

1. **[WorkflowEditPage Refactoring](./01-workflow-edit-page-refactoring.md)**
   - **File**: `gui/src/pages/workflow/edit/page.rs` (317 lines)
   - **Priority**: CRITICAL
   - **Action**: Split into 6-8 focused components
   - **Impact**: Massive single component violating SRP

2. **[Common Hooks Extraction](./02-common-hooks-extraction.md)**
   - **Files**: Multiple page components
   - **Priority**: HIGH
   - **Action**: Extract shared state management patterns
   - **Impact**: Will benefit all pages immediately

### 🔄 High Priority (Next 2 Weeks)

3. **[UI Component Library](./03-ui-component-library.md)**
   - **Files**: Multiple page components with repeated UI patterns
   - **Priority**: MEDIUM
   - **Action**: Create reusable component library
   - **Impact**: Improve consistency and maintainability

4. **[App Layout Refactoring](./04-app-layout-refactoring.md)**
   - **File**: `gui/src/layout/app.rs` (160 lines)
   - **Priority**: MEDIUM
   - **Action**: Split into focused layout components
   - **Impact**: Simplify main application structure

5. **[Service Layer Standardization](./05-service-layer-standardization.md)**
   - **Files**: `gui/src/services/` directory
   - **Priority**: MEDIUM
   - **Action**: Standardize API patterns and error handling
   - **Impact**: Improve API consistency and reliability

### 📊 Medium Priority (Next Month)

6. **[Specific File Refactoring](./06-specific-file-refactoring.md)**
   - **Files**: Multiple large files identified by quality check
   - **Priority**: MIXED
   - **Action**: Refactor individual files based on priority matrix
   - **Impact**: Reduce file sizes and improve maintainability

### 🏗️ Long-term (Next Quarter)

7. **[Architecture Improvements](./07-architecture-improvements.md)**
   - **Scope**: Overall application architecture
   - **Priority**: LONG-TERM
   - **Action**: Implement domain-driven design and comprehensive testing
   - **Impact**: Foundation for future development

## 🎯 Implementation Strategy

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

## 📈 Success Metrics

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

## 🚀 Quick Start

1. **Start with WorkflowEditPage** - It's the biggest pain point
2. **Extract common hooks** - Will benefit all pages immediately
3. **Create reusable components** - Reduce duplication across forms
4. **Refactor app.rs** - Simplify the main application structure

## 📊 Current State

### File Size Analysis
```
317 gui/src/pages/workflow/edit/page.rs      (CRITICAL)
202 gui/src/pages/prompts/edit/page.rs       (HIGH)
192 gui/src/pages/workflow/edit/handlers.rs  (MEDIUM)
175 gui/src/pages/settings/page.rs           (MEDIUM)
161 gui/src/pages/prompts/list/page.rs       (MEDIUM)
160 gui/src/layout/app.rs                    (HIGH)
158 gui/src/pages/workflow/visualizer/page.rs (LOW)
141 gui/src/pages/workflow/upload/page.rs    (MEDIUM)
```

### Quality Check Results
- ✅ **Formatting**: All code properly formatted
- ✅ **Clippy**: No warnings or errors
- ✅ **Tests**: All tests pass
- ✅ **Compilation**: Clean build across all packages
- ⚠️ **Large Files**: 6 files identified for refactoring

## 🔧 Tools and Commands

### Quality Check
```bash
task quality
```

### Format Code
```bash
cargo fmt
```

### Lint Code
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Run Tests
```bash
cargo test
```

### Build Application
```bash
cargo build --package s_e_e_gui
```

## 📚 Additional Resources

- [Rust Best Practices](https://doc.rust-lang.org/book/)
- [Dioxus Documentation](https://dioxuslabs.com/)
- [Component Design Patterns](https://atomicdesign.bradfrost.com/)
- [State Management Patterns](https://redux.js.org/understanding/thinking-in-redux/three-principles)

## 🤝 Contributing

When working on refactoring:

1. **Read the specific plan** for the file/component you're refactoring
2. **Follow the implementation steps** outlined in each plan
3. **Test thoroughly** after each change
4. **Update documentation** as you go
5. **Run quality checks** before committing

## 📝 Notes

- All refactoring plans preserve existing functionality
- Each plan includes before/after examples
- Success metrics are defined for each phase
- Risk mitigation strategies are included
- Implementation order is optimized for maximum impact
