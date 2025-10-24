# Phase 1: Core Type System Updates

## Overview

Add `WaitingForInput` status to the core type system. This phase is testable in isolation and has minimal risk since it only involves adding enum variants and updating match statements.

## Changes Required

### 1. Add WaitingForInput to TaskStatus

**File**: `core/src/types.rs`

**Current**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
}
```

**Updated**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
    WaitingForInput,  // Add this line
}
```

### 2. Update TaskStatus Implementation

**File**: `core/src/types.rs`

Update the `as_str()` method:
```rust
impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::InProgress => "in-progress",
            TaskStatus::Complete => "complete",
            TaskStatus::Failed => "failed",
            TaskStatus::WaitingForInput => "waiting-for-input",  // Add this line
        }
    }
}
```

Update the `from_str()` method:
```rust
impl FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(TaskStatus::Pending),
            "in-progress" => Ok(TaskStatus::InProgress),
            "complete" => Ok(TaskStatus::Complete),
            "failed" => Ok(TaskStatus::Failed),
            "waiting-for-input" => Ok(TaskStatus::WaitingForInput),  // Add this line
            _ => Err(format!("Invalid task status: {}", s)),
        }
    }
}
```

### 3. Add WaitingForInput to WorkflowStatus

**File**: `core/src/persistence/models.rs`

**Current**:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Running,
    Complete,
    Failed,
}
```

**Updated**:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Running,
    Complete,
    Failed,
    WaitingForInput,  // Add this line
}
```

## Testing Phase 1

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

**Expected Result**: All match statements should be exhaustive (no compilation errors)

### String Conversion Test
```rust
// Test in a simple program or unit test
use s_e_e_core::types::TaskStatus;

let status = TaskStatus::WaitingForInput;
assert_eq!(status.as_str(), "waiting-for-input");
assert_eq!(TaskStatus::from_str("waiting-for-input").unwrap(), TaskStatus::WaitingForInput);
```

## Potential Failures

### 1. Non-Exhaustive Pattern Matching
**Error**: `error[E0004]: non-exhaustive patterns: TaskStatus::WaitingForInput not covered`
**Solution**: Use grep to find all match statements first, then update each one

**Files to check**:
- `core/src/engine/handlers/cli_command.rs`
- `core/src/engine/handlers/cursor_agent.rs`
- `core/src/execution/context.rs`
- Any GUI files that match on TaskStatus

### 2. String Conversion Mismatches
**Error**: Serialization/deserialization fails
**Solution**: Ensure `as_str()` and `from_str()` handle the new variant

### 3. Missing Imports
**Error**: `TaskStatus` or `WorkflowStatus` not found in scope
**Solution**: Add proper imports where needed

### 4. Serialization Issues
**Error**: JSON serialization fails for new variant
**Solution**: The `Serialize` and `Deserialize` derives should handle this automatically

## Files That May Need Updates

### Core Files
- `core/src/types.rs` - Main changes
- `core/src/persistence/models.rs` - WorkflowStatus update

### Files That May Have Match Statements
- `core/src/execution/context.rs` - May match on TaskStatus
- `core/src/engine/handlers/cli_command.rs` - May match on TaskStatus
- `core/src/engine/handlers/cursor_agent.rs` - May match on TaskStatus
- `core/src/engine/execute.rs` - May match on TaskStatus

### GUI Files (Future Phases)
- `gui/src/pages/executions/details/components/task_details_panel.rs`
- `gui/src/pages/executions/details/components/execution_overview.rs`
- `gui/src/pages/executions/details/components/workflow_flow.rs`

## Step-by-Step Implementation

### Step 1: Update TaskStatus Enum
1. Open `core/src/types.rs`
2. Add `WaitingForInput,` to the enum
3. Update `as_str()` method
4. Update `from_str()` method

### Step 2: Update WorkflowStatus Enum
1. Open `core/src/persistence/models.rs`
2. Add `WaitingForInput,` to the enum

### Step 3: Find and Update Match Statements
1. Run grep to find all match statements
2. Update each match statement to handle `WaitingForInput`
3. Compile after each change

### Step 4: Test Compilation
1. Run `cargo build -p s_e_e_core`
2. Fix any compilation errors
3. Verify all match statements are exhaustive

## Success Criteria

### ✅ Compilation Success
- Core crate compiles without errors
- No non-exhaustive pattern matching errors
- All imports resolved correctly

### ✅ String Conversions Work
- `as_str()` returns correct string for WaitingForInput
- `from_str()` parses "waiting-for-input" correctly
- Serialization/deserialization works

### ✅ All Match Statements Updated
- No compilation errors about missing patterns
- All existing functionality preserved
- New variant handled consistently

### ✅ No Functional Changes
- Existing workflows execute identically
- No performance impact
- No behavior changes

## Verification Checklist

- [ ] TaskStatus enum has WaitingForInput variant
- [ ] WorkflowStatus enum has WaitingForInput variant
- [ ] as_str() method handles WaitingForInput
- [ ] from_str() method handles "waiting-for-input"
- [ ] All match statements on TaskStatus are exhaustive
- [ ] All match statements on WorkflowStatus are exhaustive
- [ ] Core crate compiles without errors
- [ ] No functional changes to existing code

## Common Match Statement Patterns

### TaskStatus Match Statements
```rust
// Pattern 1: Status display
match task.status {
    TaskStatus::Pending => "Pending",
    TaskStatus::InProgress => "In Progress",
    TaskStatus::Complete => "Complete",
    TaskStatus::Failed => "Failed",
    TaskStatus::WaitingForInput => "Waiting for Input",  // Add this
}

// Pattern 2: Status colors
match task.status {
    TaskStatus::Pending => "#6b7280",
    TaskStatus::InProgress => "#3b82f6",
    TaskStatus::Complete => "#10b981",
    TaskStatus::Failed => "#ef4444",
    TaskStatus::WaitingForInput => "#f59e0b",  // Add this (amber)
}

// Pattern 3: Status checks
if task.status == TaskStatus::Complete {
    // handle completion
} else if task.status == TaskStatus::WaitingForInput {  // Add this
    // handle waiting for input
}
```

### WorkflowStatus Match Statements
```rust
// Pattern 1: Status display
match workflow.status {
    WorkflowStatus::Running => "Running",
    WorkflowStatus::Complete => "Complete",
    WorkflowStatus::Failed => "Failed",
    WorkflowStatus::WaitingForInput => "Waiting for Input",  // Add this
}
```

## Next Steps

After Phase 1 is complete and verified:
1. Proceed to Phase 2 (Execution Context Pause/Resume)
2. Ensure all success criteria are met
3. Do not proceed if there are any compilation errors

## Notes

- This phase should have ZERO functional impact
- The goal is to add the new status without breaking anything
- Take time to find all match statements - missing one will cause compilation errors
- Test thoroughly before proceeding to Phase 2
