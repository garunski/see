# Phase 2: Replace Prompts Page

**Estimated Time**: 2-3 hours (including human UI testing)

## Goal

Replace the prompts list page to use dioxus-query. Create ONLY the files needed for this one page.

## Strategy

Create files as they are needed, implement the prompts page fully, test it, then move on.

## Steps

### Step 2.1: Create GetPrompts Query (ONLY THIS ONE)

**File**: `gui/src/queries/mod.rs` (create when needed)

```rust
pub mod prompt_queries;
pub use prompt_queries::*;
```

**File**: `gui/src/queries/prompt_queries.rs` (create this file)

```rust
use dioxus_query::QueryCapability;
use s_e_e_core::{Prompt, get_global_store};
use crate::services::prompt::UserPromptService;

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetPrompts;

impl QueryCapability for GetPrompts {
    type Ok = Vec<Prompt>;
    type Err = String;
    type Keys = ();

    async fn run(&self, _: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        UserPromptService::fetch_prompts()
            .await
            .map_err(|e| e.to_string())
    }
}
```

**Validation**: `task quality`

### Step 2.2: Update Prompts Page to Use Query

**File**: `gui/src/pages/prompts/list/page.rs`

Replace the component to use queries instead of old state management.

**Validation**: 
- Code compiles
- Page loads
- Run `task quality`

### Step 2.3: Human UI Testing (YOU TEST THIS)

**Test Checklist**:
1. Navigate to /prompts
2. Verify prompts list displays correctly
3. Create a new prompt - verify it appears
4. Edit existing prompt - verify changes save
5. Delete a prompt - verify it disappears
6. Navigate away and back - verify state persists
7. Check network tab - verify single request on load
8. Verify no console errors

**Validation**: All tests pass

### Step 2.4: Quality Checks

```bash
task quality
```

**Expected**: All pass, no warnings about unused code

## Success Criteria

✅ Prompts page works with query system  
✅ Human testing confirms no regressions  
✅ No console errors  
✅ Network requests are efficient  
✅ All CRUD operations work  
✅ No unused code warnings from clippy  

**Important**: Do NOT create any other query files yet. Wait until Phase 3.

