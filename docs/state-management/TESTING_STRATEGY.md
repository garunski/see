# Testing Strategy

## Overview

This document outlines the testing approach for validating the state management refactor using dioxus-query.

## Compilation Tests

### No Cloning Errors

**Objective**: Verify no cloning-related compilation errors exist

**Test**:
```bash
cargo check
```

**Expected**: Compiles without errors

**If fails**: Check for `.clone()` calls in query result usage

### No Borrowing Errors

**Objective**: Verify borrow checker is happy

**Test**:
```bash
cargo clippy --all-targets --all-features
```

**Expected**: No borrowing-related warnings

**Common errors**:
- `E0382: use of moved value` - Fix by removing unnecessary moves
- `E0502: cannot borrow as mutable` - Fix by using query state directly
- `E0277: cannot iterate` - Fix by accessing query data correctly

### Type Safety

**Objective**: Verify QueryCapability implementations are type-safe

**Test**: 
```bash
cargo build --all-targets
```

**Expected**: All types match correctly

**Check**: Ensure each query returns correct types as specified in `CORE_SPEC.md`

## Functional Tests

### Test 1: Prompts Page Loading

**Description**: Test that prompts page loads and displays data correctly

**Steps**:
1. Navigate to /prompts
2. Verify loading spinner shows initially
3. Verify prompts list appears after load
4. Check network tab - should see single request to fetch prompts

**Expected**:
- Loading state shows
- Prompts render correctly
- Only one API call made
- No console errors

**Files to check**:
- `gui/src/pages/prompts/list/page.rs`

### Test 2: History Page - Polling Behavior

**Description**: Test that running workflows auto-update via polling

**Steps**:
1. Start a workflow execution
2. Navigate to /executions/history
3. Verify running workflows section shows the execution
4. Wait 5 seconds
5. Verify page updates automatically (may see status changes)
6. Complete the execution
7. Verify it moves to completed section
8. Verify polling stops (check network tab)

**Expected**:
- Running workflows appear
- Page polls every 5 seconds
- Updates visible without manual refresh
- Polling stops when no running workflows
- Execution moves to completed

**Files to check**:
- `gui/src/pages/executions/history/page.rs`
- `gui/src/queries/history_queries.rs`

### Test 3: Execution Details - Smart Polling

**Description**: Test that execution details polls but stops when complete

**Steps**:
1. Navigate to running execution details page
2. Open network tab
3. Verify requests happen every 2 seconds
4. Complete the execution
5. Verify requests stop after status becomes Complete
6. Refresh page
7. Verify only one request made (no polling for completed execution)

**Expected**:
- Polls every 2 seconds while running
- Stops polling when status is Complete or Failed
- Single request on page load for completed executions

**Files to check**:
- `gui/src/pages/executions/details/page.rs`
- `gui/src/pages/executions/details/hooks.rs`
- `gui/src/queries/history_queries.rs`

### Test 4: Cache Invalidation on Mutations

**Description**: Test that creating/updating items invalidates cache

**Subtest 4.1: Create Workflow**
1. Go to workflows list
2. Create new workflow
3. Verify workflows list auto-updates with new workflow
4. Check network tab - should see new fetch after mutation

**Subtest 4.2: Update Workflow**
1. Edit an existing workflow
2. Save changes
3. Verify workflow list shows updated data
4. Check cache invalidation

**Subtest 4.3: Create Prompt**
1. Go to prompts list
2. Create new prompt
3. Verify prompts list auto-updates

**Expected**:
- All mutations trigger cache invalidation
- UI updates automatically
- No manual refresh needed

**Files to check**:
- `gui/src/queries/mutations.rs`

### Test 5: Multiple Components Using Same Query

**Description**: Test that multiple components using same query deduplicate requests

**Steps**:
1. Open prompts list in two tabs
2. Both should show prompts
3. Check network tab - should see only one request total
4. Create a prompt
5. Both tabs should update

**Expected**:
- Request deduplication works
- Cache shared across components
- Updates propagate to all components

### Test 6: Error Handling

**Description**: Test error state handling

**Steps**:
1. Disconnect database or cause error
2. Navigate to any page using queries
3. Verify error message shows
4. Verify graceful degradation

**Expected**:
- Error state renders correctly
- No crashes
- User sees helpful error message

**Files to check**:
- All pages using queries

### Test 7: Loading State During Refetch

**Description**: Test loading states during cache invalidation refetch

**Steps**:
1. Create a workflow
2. Verify workflows list shows loading briefly
3. Verify new workflow appears

**Expected**:
- Loading state shown during refetch
- Smooth transition from loading to success
- No flicker or white screen

## Performance Tests

### Test 1: Request Deduplication

**Objective**: Verify same query key doesn't make duplicate requests

**Manual Test**:
1. Open browser DevTools network tab
2. Navigate to prompts page
3. Navigate away and back
4. Count requests to fetch prompts

**Expected**: Only one request total

### Test 2: Cache Reuse

**Objective**: Verify cached data is reused

**Manual Test**:
1. Navigate to prompts page
2. Navigate to workflows page
3. Navigate back to prompts
4. Check network tab

**Expected**: No new request when returning to prompts (uses cache)

### Test 3: Polling Efficiency

**Objective**: Verify polling doesn't continue unnecessarily

**Manual Test**:
1. Open execution details for completed workflow
2. Check network tab for 10 seconds
3. Count requests

**Expected**: Only 1-2 requests (should stop polling)

### Test 4: Memory Usage

**Objective**: Verify no memory leaks from queries

**Manual Test**:
1. Navigate between pages multiple times
2. Check browser memory usage
3. Trigger garbage collection
4. Check memory again

**Expected**: No memory growth over time

## Integration Tests

### Test 1: End-to-End Workflow Execution

**Steps**:
1. Start workflow execution
2. Navigate to history page
3. Verify execution appears in running section
4. Verify auto-updates as it progresses
5. Navigate to execution details
6. Verify details page polls and updates
7. Wait for completion
8. Verify stops polling
9. Verify shows in completed section

**Expected**: Entire flow works smoothly with correct polling behavior

### Test 2: User Interaction During Polling

**Steps**:
1. Navigate to history page
2. Start polling (running workflows visible)
3. Create a new workflow while on history page
4. Verify new workflow appears
5. Verify polling continues for running workflows

**Expected**: Interactions don't disrupt polling

## Regression Tests

### Existing Features Still Work

**Checklist**:
- [ ] Workflows execute correctly
- [ ] CLI commands still work
- [ ] Database operations succeed
- [ ] User input still works
- [ ] All existing UI features work
- [ ] Settings save and load
- [ ] Theme switching works
- [ ] File picker works

**Validation**: Run full manual test suite of all app features

## Test Results

### Success Criteria

✅ All compilation tests pass  
✅ All functional tests pass  
✅ All performance tests acceptable  
✅ No regressions in existing features  
✅ Cache invalidation works correctly  
✅ Polling stops appropriately  
✅ No redundant API calls  
✅ UI updates reactively  

### Failure Cases

If tests fail:
1. Check error messages
2. Review query implementations
3. Verify cache invalidation calls
4. Check for clone/borrow issues
5. Fix and re-test

## Continuous Testing

### During Development

Run these after each phase:
```bash
cargo check
cargo clippy --all-targets --all-features
cargo test
```

### Before Committing

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
cargo build --release
```

## Test Coverage

**Target**: Maintain or improve existing test coverage

**Areas to test**:
- Query capabilities work correctly
- Cache invalidation happens
- Polling starts and stops appropriately
- Components render with query data
- Error states display correctly
- Loading states show

## Performance Benchmarks

### Baseline (Before Refactor)

- Prompts page: ~500ms initial load
- History page: ~800ms initial load
- Execution details: ~2s polling interval

### Target (After Refactor)

- Prompts page: <500ms (with caching)
- History page: <600ms (with caching)
- Execution details: <2s polling, stops when complete
- Zero redundant requests
- Reduced memory usage (no excessive clones)

### Measurement

Use browser DevTools:
- Network tab for request counts
- Performance tab for load times
- Memory tab for memory usage

## Documentation

After successful tests, document:
- Any new patterns discovered
- Performance improvements achieved
- Known limitations
- Future improvements

## Rollback Testing

### If Issues Found

1. Revert to previous commit
2. Run regression tests
3. Verify working state restored
4. Investigate issues in isolated branch
5. Fix and re-test before merging

