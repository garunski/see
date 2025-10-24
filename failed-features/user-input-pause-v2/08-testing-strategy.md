# Testing Strategy - User Input Pause Feature

## Overview

This document outlines the comprehensive testing strategy for the user input pause feature implementation. The strategy is designed to catch issues early and ensure each phase is thoroughly tested before proceeding to the next.

## Testing Principles

### 1. Phase-by-Phase Testing
- **Each phase must be tested independently** before proceeding
- **No phase can be skipped** - all must pass
- **Compilation testing** after every file edit
- **Functional testing** after each phase completion

### 2. Incremental Validation
- **Start simple** - test basic functionality first
- **Build complexity** - add more complex tests as features mature
- **Verify no regressions** - ensure existing functionality unchanged

### 3. Multiple Test Types
- **Compilation tests** - ensure code compiles
- **Unit tests** - test individual components
- **Integration tests** - test component interactions
- **Manual tests** - test user interactions
- **Database tests** - verify data persistence

## Phase 0 Testing: Code Duplication Refactoring

### Compilation Test
```bash
cd /Users/garunnvagidov/code/see
cargo build -p s_e_e_core
```

**Expected Result**: Compiles without errors

### Functionality Test
1. **Run existing workflow** to verify refactored persistence works
2. **Check logs** to confirm task states are saved
3. **Verify database** contains correct task execution records

**Expected Result**: Identical behavior to before refactoring

### Code Quality Test
1. **Count lines** in both handler files
2. **Verify duplication eliminated**
3. **Check that TaskPersistenceHelper is used** in both handlers

**Expected Result**: ~140 lines removed from each handler file

### Test Data
Use existing workflows - no special test data needed.

## Phase 1 Testing: Type System Updates

### Compilation Test
```bash
cd /Users/garunnvagidov/code/see
cargo build -p s_e_e_core
```

**Expected Result**: Compiles without errors

### Pattern Matching Test
```bash
# Find all match statements on TaskStatus
grep -r "match.*TaskStatus" core/src/

# Find all match statements on WorkflowStatus  
grep -r "match.*WorkflowStatus" core/src/
```

**Expected Result**: All match statements should be exhaustive

### String Conversion Test
```rust
// Test in a simple program or unit test
use s_e_e_core::types::TaskStatus;

let status = TaskStatus::WaitingForInput;
assert_eq!(status.as_str(), "waiting-for-input");
assert_eq!(TaskStatus::from_str("waiting-for-input").unwrap(), TaskStatus::WaitingForInput);
```

**Expected Result**: String conversions work correctly

### Test Data
No special test data needed - test with enum values directly.

## Phase 2 Testing: Execution Context Pause/Resume

### Compilation Test
```bash
cd /Users/garunnvagidov/code/see
cargo build -p s_e_e_core
```

**Expected Result**: Compiles without errors

### Unit Test
Create unit tests for the new methods:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TaskInfo, TaskStatus};

    #[test]
    fn test_pause_resume_task() {
        // Create test context
        let tasks = vec![
            TaskInfo {
                id: "task1".to_string(),
                name: "Test Task".to_string(),
                status: TaskStatus::InProgress,
            }
        ];
        
        let context = ExecutionContext::new(
            tasks,
            None, // no output callback
            None, // no audit store
            "test_execution".to_string(),
            "test_workflow".to_string(),
        );

        // Test pause
        {
            let mut ctx = context.lock().unwrap();
            ctx.start_task("task1");
            let result = ctx.pause_for_input("task1", "Continue?");
            assert!(result.is_ok());
            assert_eq!(ctx.tasks[0].status, TaskStatus::WaitingForInput);
            assert!(ctx.has_waiting_tasks());
            assert_eq!(ctx.get_waiting_tasks().len(), 1);
        }

        // Test resume
        {
            let mut ctx = context.lock().unwrap();
            let result = ctx.resume_task("task1");
            assert!(result.is_ok());
            assert_eq!(ctx.tasks[0].status, TaskStatus::InProgress);
            assert!(!ctx.has_waiting_tasks());
        }
    }

    #[test]
    fn test_pause_nonexistent_task() {
        let tasks = vec![];
        let context = ExecutionContext::new(
            tasks,
            None,
            None,
            "test_execution".to_string(),
            "test_workflow".to_string(),
        );

        let mut ctx = context.lock().unwrap();
        let result = ctx.pause_for_input("nonexistent", "Continue?");
        assert!(result.is_err());
    }

    #[test]
    fn test_resume_non_waiting_task() {
        let tasks = vec![
            TaskInfo {
                id: "task1".to_string(),
                name: "Test Task".to_string(),
                status: TaskStatus::Complete,
            }
        ];
        
        let context = ExecutionContext::new(
            tasks,
            None,
            None,
            "test_execution".to_string(),
            "test_workflow".to_string(),
        );

        let mut ctx = context.lock().unwrap();
        let result = ctx.resume_task("task1");
        assert!(result.is_err());
    }
}
```

**Expected Result**: All unit tests pass

### Test Data
Use in-memory test data - no database needed.

## Phase 3 Testing: GUI Status Indicators

### Compilation Test
```bash
cd /Users/garunnvagidov/code/see
cargo build -p s_e_e_core
cargo build -p s_e_e_gui
```

**Expected Result**: Both crates compile without errors

### Manual Testing
1. **Create test data** with WaitingForInput status in database
2. **Navigate to execution details** page
3. **Verify status indicators** show correctly
4. **Check colors** are appropriate (amber for waiting)

### Test Data Creation
```sql
-- Insert a test workflow with waiting status
INSERT INTO workflow_metadata (id, workflow_name, start_timestamp, end_timestamp, status, task_ids)
VALUES ('test-waiting', 'Test Waiting Workflow', '2024-12-19T10:00:00Z', NULL, 'waiting-for-input', '["task1"]');

-- Insert a test task with waiting status  
INSERT INTO task_executions (execution_id, task_id, task_name, status, logs, start_timestamp, end_timestamp)
VALUES ('test-waiting', 'task1', 'Test Task', 'waiting-for-input', '["Task paused for input"]', '2024-12-19T10:00:00Z', '');
```

**Expected Result**: GUI shows waiting status with correct styling

## Phase 4 Testing: Simple Resume Button

### Compilation Test
```bash
cd /Users/garunnvagidov/code/see
cargo build -p s_e_e_core
cargo build -p s_e_e_gui
```

**Expected Result**: Both crates compile without errors

### Manual Testing
1. **Create test data** with WaitingForInput status in database
2. **Navigate to execution details** page
3. **Verify resume button appears** for waiting tasks
4. **Click resume button** and check logs
5. **Verify no infinite loops** or performance issues

### Log Verification
After clicking the resume button, check the logs for:
```
INFO [gui]: Resume button clicked for execution test-waiting task task1
```

**Expected Result**: Button click logs message correctly

### Test Data
Use same test data as Phase 3.

## Phase 5 Testing: Actual Resume Implementation

### Compilation Test
```bash
cd /Users/garunnvagidov/code/see
cargo build -p s_e_e_core
cargo build -p s_e_e_gui
```

**Expected Result**: Both crates compile without errors

### Functionality Test
1. **Create test data** with WaitingForInput status in database
2. **Navigate to execution details** page
3. **Click resume button**
4. **Check database** to verify task status updated
5. **Refresh page** to verify status changed

### Database Verification
After clicking resume, check the database:

```sql
-- Check task status
SELECT task_id, status, end_timestamp FROM task_executions WHERE execution_id = 'test-waiting';

-- Check workflow status
SELECT status FROM workflow_metadata WHERE id = 'test-waiting';
```

**Expected Result**: 
- Task status should be `in-progress`
- Workflow status should be `running`
- End timestamp should be empty

### Test Data
Use same test data as previous phases.

## Phase 6 Testing: Simple Persistence

### Compilation Test
```bash
cd /Users/garunnvagidov/code/see
cargo build -p s_e_e_core
cargo build -p s_e_e_gui
```

**Expected Result**: Both crates compile without errors

### Database Migration Test
1. **Start the app** to trigger database migration
2. **Check database schema** to verify new fields added
3. **Verify existing data** is not corrupted

### Persistence Test
1. **Create test workflow** with waiting status
2. **Restart the app**
3. **Verify paused workflow** is still there
4. **Resume the workflow**
5. **Verify status** is updated correctly

### Test Data Creation
```sql
-- Insert a test workflow with waiting status
INSERT INTO workflow_metadata (id, workflow_name, start_timestamp, end_timestamp, status, task_ids, is_paused, paused_task_id)
VALUES ('test-waiting', 'Test Waiting Workflow', '2024-12-19T10:00:00Z', NULL, 'waiting-for-input', '["task1"]', true, 'task1');

-- Insert a test task with waiting status  
INSERT INTO task_executions (execution_id, task_id, task_name, status, logs, start_timestamp, end_timestamp)
VALUES ('test-waiting', 'task1', 'Test Task', 'waiting-for-input', '["Task paused for input"]', '2024-12-19T10:00:00Z', '');
```

### Database Verification
After restart, check the database:

```sql
-- Check workflow metadata
SELECT id, status, is_paused, paused_task_id FROM workflow_metadata WHERE id = 'test-waiting';

-- Check task status
SELECT task_id, status FROM task_executions WHERE execution_id = 'test-waiting';
```

**Expected Result**: 
- Workflow should have `is_paused = true` and `paused_task_id = 'task1'`
- Task should have status `waiting-for-input`

## End-to-End Testing

### Complete Workflow Test
1. **Start a workflow** that will pause for input
2. **Verify pause** occurs at correct point
3. **Check GUI** shows waiting status
4. **Click resume** button
5. **Verify workflow** continues and completes
6. **Restart app** and verify persistence

### Error Scenarios Test
1. **Invalid task ID** - should handle gracefully
2. **Wrong task status** - should handle gracefully
3. **Database errors** - should handle gracefully
4. **Network issues** - should handle gracefully

### Performance Test
1. **Multiple paused workflows** - should handle efficiently
2. **Large workflows** - should not cause performance issues
3. **Concurrent operations** - should handle safely

## Test Data Management

### Test Database
- **Use separate test database** for testing
- **Clean up after tests** to avoid interference
- **Use consistent test data** across phases

### Test Workflows
- **Create simple test workflows** for basic testing
- **Create complex test workflows** for edge case testing
- **Document test workflow behavior** for reference

### Test Users
- **Use test user accounts** for GUI testing
- **Verify permissions** work correctly
- **Test error handling** with different user roles

## Continuous Testing

### Pre-commit Testing
- **Compilation test** before every commit
- **Unit tests** before every commit
- **Basic functionality test** before every commit

### Phase Completion Testing
- **Full test suite** before marking phase complete
- **Integration test** before proceeding to next phase
- **Documentation update** after each phase

### Release Testing
- **End-to-end test** before release
- **Performance test** before release
- **User acceptance test** before release

## Test Automation

### Automated Tests
- **Unit tests** - automated in CI/CD
- **Integration tests** - automated in CI/CD
- **Compilation tests** - automated in CI/CD

### Manual Tests
- **GUI testing** - manual verification
- **User experience testing** - manual verification
- **Edge case testing** - manual verification

### Test Reporting
- **Test results** documented for each phase
- **Issues found** tracked and resolved
- **Test coverage** measured and reported

## Troubleshooting

### Common Issues
1. **Compilation errors** - check imports and types
2. **Database errors** - check connection and schema
3. **GUI issues** - check state management and effects
4. **Performance issues** - check for infinite loops

### Debug Tools
- **Logging** - extensive logging for debugging
- **Database inspection** - check data directly
- **GUI inspection** - check component state
- **Network inspection** - check API calls

### Recovery Procedures
- **Rollback plan** for each phase
- **Data recovery** procedures
- **System recovery** procedures
- **User recovery** procedures

## Conclusion

This testing strategy ensures the user input pause feature is thoroughly tested at each phase. The incremental approach catches issues early and prevents them from propagating to later phases. The comprehensive test coverage ensures the feature works correctly and reliably in production.
