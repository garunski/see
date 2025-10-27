# State Management Refactor with dioxus-query 0.8.1

## Overview

Refactor the Dioxus GUI application's state management to use dioxus-query 0.8.1, addressing:
- Excessive `.clone()` calls causing compilation errors
- Naive polling with no caching
- Manual `needs_reload` flags
- Poor reactivity

## Solution

Use [dioxus-query 0.8.1](https://docs.rs/dioxus-query/latest/dioxus_query/) for:
- Type-safe data fetching with automatic caching
- Smart polling that stops when not needed
- Cache invalidation on mutations
- Proper ownership (no cloning)

## Implementation

Follow phases in order:

1. **[Phase 1: Setup](PHASE_1_SETUP.md)** - Add dependency (15 min)
2. **[Phase 2: Prompts Page](PHASE_2_PROMPTS_PAGE.md)** - Replace prompts page (2-3 hours)
3. **[Phase 3: History Page](PHASE_3_HISTORY_PAGE.md)** - Add polling for history (3-4 hours)
4. **[Phase 4: Execution Details](PHASE_4_EXECUTION_DETAILS.md)** - Smart polling (2-3 hours)
5. **[Phase 5: Cleanup](PHASE_5_CLEANUP.md)** - Remove old code (1-2 hours)

**See [IMPLEMENTATION_STEPS.md](IMPLEMENTATION_STEPS.md) for overview.**

## Principles

- **NO FRONT LOADING**: Create files only when needed for current page
- **ONE PAGE AT A TIME**: Complete and test each before moving on
- **HUMAN TESTING**: Manual UI testing after each phase
- **QUALITY CHECKS**: Run `task quality` after each phase

## Quick Links

- [dioxus-query 0.8.1 Docs](https://docs.rs/dioxus-query/latest/dioxus_query/)
- [TanStack Query Inspiration](https://tanstack.com/query/v5/docs/framework/react/overview)

## Success Criteria

✅ No compilation errors  
✅ Smart polling stops automatically  
✅ Cache invalidation works  
✅ UI updates reactively  
✅ No redundant API calls  
✅ Human tests pass  
