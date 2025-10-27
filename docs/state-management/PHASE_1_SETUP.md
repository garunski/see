# Phase 1: Setup

**Estimated Time**: 15 minutes

## Goal

Add the dioxus-query dependency only. Do NOT create any empty files yet.

## Steps

### Step 1.1: Add dioxus-query Dependency

**File**: `gui/Cargo.toml`

```toml
[dependencies]
dioxus-query = "0.8.1"  # Latest version - check https://docs.rs/dioxus-query/latest/dioxus_query/
```

**Validation**: 
```bash
task quality
```

## Success Criteria

✅ Dependency added  
✅ `task quality` passes  
✅ Nothing else created yet

**Important**: That's it. No files, no structure, no helpers. Just the dependency.
