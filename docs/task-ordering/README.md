# Task Ordering and Workflow Snapshot Documentation

## Overview

This documentation folder contains comprehensive specifications and implementation guides for the task ordering feature that preserves workflow structure in execution records.

## Problem Statement

### Issue 1: Incorrect Task Ordering

Tasks are displayed in the wrong order in the GUI because:
- Workflows are parsed using DFS (depth-first search) which flattens structure
- The order in `exec.tasks` doesn't match actual execution order
- Without original workflow JSON, correct ordering cannot be determined

### Issue 2: Critical Bug - No Workflow Pause

Workflows complete instead of pausing for user input, making interactive workflows unusable.

## Solution

### Workflow Snapshot Storage

Store the complete workflow JSON in each execution via `workflow_snapshot: serde_json::Value` field. This enables:
- Correct task ordering from original structure
- Self-contained execution records
- Complete audit trail
- Historical accuracy

### Bug Fix

Investigate and fix workflow pause mechanism to properly handle user input tasks.

## Documentation Structure

### Architecture and Specifications

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture overview
- **[PERSISTENCE_SPEC.md](PERSISTENCE_SPEC.md)** - Persistence layer changes
- **[CORE_SPEC.md](CORE_SPEC.md)** - Core API changes
- **[GUI_SPEC.md](GUI_SPEC.md)** - GUI implementation

### Bug Investigation

- **[BUG_INVESTIGATION.md](BUG_INVESTIGATION.md)** - User input pause bug analysis and debugging strategy

### Implementation Guides

- **[IMPLEMENTATION_STEPS.md](IMPLEMENTATION_STEPS.md)** - Phase-by-phase implementation guide
- **[TESTING_STRATEGY.md](TESTING_STRATEGY.md)** - Comprehensive testing approach

### Phase Documentation

- **[PHASE_1_PERSISTENCE.md](PHASE_1_PERSISTENCE.md)** - Add workflow_snapshot field
- **[PHASE_2_CORE_INTEGRATION.md](PHASE_2_CORE_INTEGRATION.md)** - Store snapshot on execution
- **[PHASE_3_GUI_ORDERING.md](PHASE_3_GUI_ORDERING.md)** - Implement task ordering
- **[PHASE_4_BUG_DEBUG.md](PHASE_4_BUG_DEBUG.md)** - Debug workflow pause issue
- **[PHASE_5_TESTING.md](PHASE_5_TESTING.md)** - Comprehensive testing
- **[PHASE_6_DOCUMENTATION.md](PHASE_6_DOCUMENTATION.md)** - Final documentation and quality

## Implementation Phases

| Phase | Focus | Duration |
|-------|-------|----------|
| Phase 1 | Persistence Layer | 1 hour |
| Phase 2 | Core Integration | 1.5 hours |
| Phase 3 | GUI Ordering | 1.5 hours |
| Phase 4 | Bug Investigation | 2-4 hours |
| Phase 5 | Testing | 2 hours |
| Phase 6 | Documentation | 1 hour |

**Total Estimated Duration**: 8-10 hours

## Key Features

### 1. Workflow Snapshot

Each execution stores:
- Complete workflow JSON structure
- Original task relationships
- Exact configuration at execution time

### 2. Task Ordering

GUI extracts task IDs from snapshot:
- Preserves execution order
- Handles parallel and nested workflows
- Correct display in details page

### 3. Bug Fix

Fix workflow pause mechanism:
- Investigate using tracing and logs
- Identify root cause
- Implement fix
- Verify user input workflows work

## Code Quality Standards

### Single Responsibility Principle (SRP)

- Small, focused files
- Each file has ONE responsibility
- Clear module boundaries

### Test Organization

- Separate test files for each module
- Tests mirror source structure
- Dedicated test directories

### Quality Checks

Final phase runs `task quality` to ensure:
- No lint warnings
- Code properly formatted
- All tests pass
- SRP compliance

## Getting Started

### For Developers

1. Read [ARCHITECTURE.md](ARCHITECTURE.md) for overview
2. Read [IMPLEMENTATION_STEPS.md](IMPLEMENTATION_STEPS.md) for roadmap
3. Follow phase documentation (Phase 1-6)
4. Run tests after each phase
5. Run `task quality` in Phase 6

### For Bug Investigation

1. Read [BUG_INVESTIGATION.md](BUG_INVESTIGATION.md)
2. Follow Phase 4 steps
3. Enable tracing logs
4. Identify root cause
5. Implement fix

## Success Criteria

✅ Task ordering works correctly  
✅ Workflow pause works correctly  
✅ All tests pass  
✅ Quality checks pass  
✅ Documentation complete  
✅ Ready for production  

## Related Documentation

- [User Input Documentation](../user-input/) - User input feature specs
- [README.md](../../README.md) - Main project documentation

## Status

**Current Status**: ⏳ Pending Implementation

All documentation created. Ready to begin implementation.

