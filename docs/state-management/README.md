# State Management Refactor

## Overview

This documentation describes the refactoring of the Dioxus GUI application's state management to use the `dioxus-query` crate. The refactor addresses issues with excessive cloning, naive polling, poor reactivity, and compilation errors.

## Problem Statement

The current state management has several critical issues:

- **Excessive cloning**: Heavy use of `.clone()` causing compilation errors and performance issues
- **Naive polling**: Execution details page polls every 2 seconds unconditionally with no caching
- **Poor reactivity**: State changes don't trigger UI updates properly
- **Manual reload flags**: `needs_reload` flags that don't work correctly
- **Borrow checker fights**: Compilation errors from `Signal<T>` with nested reads

## Solution

Use [dioxus-query](https://docs.rs/dioxus-query/latest/dioxus_query/) - a production-ready, fully-typed async state management system inspired by [TanStack Query](https://tanstack.com/query/v5/docs/framework/react/overview).

### Key Features

- **QueryCapability trait**: Define type-safe data fetching with automatic caching
- **Smart polling**: Optional background re-execution that stops when not needed
- **Cache invalidation**: Automatic or manual cache invalidation for data consistency
- **No cloning**: Proper ownership management eliminates cloning issues
- **Focus-based refetch**: Auto-refresh when window gains focus
- **Loading/error states**: Built-in handling for async operations

## Documentation Structure

1. **[ARCHITECTURE.md](./ARCHITECTURE.md)** - Current issues and proposed architecture
2. **[CORE_SPEC.md](./CORE_SPEC.md)** - Query capabilities, mutations, and cache invalidation
3. **[GUI_SPEC.md](./GUI_SPEC.md)** - Component changes and UI patterns
4. **[IMPLEMENTATION_STEPS.md](./IMPLEMENTATION_STEPS.md)** - Phased rollout plan
5. **[TESTING_STRATEGY.md](./TESTING_STRATEGY.md)** - Validation criteria and testing approach
6. **[EXAMPLES.md](./EXAMPLES.md)** - Before/after code examples for each pattern

## Current Status

**Phase**: Documentation  
**Last Updated**: Current date  
**Blockers**: None

## Quick Links

- [dioxus-query Documentation](https://docs.rs/dioxus-query/)
- [TanStack Query Inspiration](https://tanstack.com/query/v5/docs/framework/react/overview)
- [Dioxus State Management Guide](https://dioxuslabs.com/learn/0.6/essentials/state/)

## Implementation Phases

1. **Phase 1**: Add dioxus-query dependency, create query capabilities
2. **Phase 2**: Create query hooks module
3. **Phase 3**: Replace prompts page (simplest case)
4. **Phase 4**: Replace history page (polling case)
5. **Phase 5**: Replace execution details (smart polling)
6. **Phase 6**: Update remaining components
7. **Phase 7**: Clean up old code and remove anti-patterns
8. **Phase 8**: Testing and validation

## Benefits

✅ **No compilation errors** - Proper ownership eliminates borrowing issues  
✅ **Smart polling** - Auto-stops when not needed  
✅ **Automatic caching** - No redundant API calls  
✅ **Cache invalidation** - Mutations auto-refresh related data  
✅ **Better reactivity** - UI updates when query state changes  
✅ **Cleaner code** - No manual state management boilerplate  
✅ **Production-ready** - Using battle-tested library  

