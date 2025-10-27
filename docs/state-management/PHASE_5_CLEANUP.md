# Phase 5: Cleanup Old Code

**Estimated Time**: 1-2 hours

## Goal

Remove old state management code, reload flags, and manual polling.

## Steps

### Step 5.1: Remove Old Hooks

**File**: `gui/src/hooks/use_app_state.rs`

Remove unused hook implementations.

### Step 5.2: Remove Reload Flags

Remove `needs_reload` flags from state files.

### Step 5.3: Remove Manual Polling

Remove any remaining manual polling loops.

### Step 5.4: Quality Checks

```bash
task quality
```

**Expected**: All pass, no warnings

## Success Criteria

✅ Old code removed  
✅ No unused warnings  
✅ All tests pass  

