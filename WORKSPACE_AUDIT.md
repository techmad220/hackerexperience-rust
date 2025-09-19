Workspace Audit (Phase 1 Consolidation)
=======================================

Objective
---------
Reduce duplication and drift by limiting the active workspace to canonical crates and facades. Avoid building legacy duplicates and conflicting implementations (e.g., Postgres vs MySQL DB layers) during normal development.

Changes Applied
---------------
- Cargo workspace narrowed to prefer Postgres database crate:
  - Excluded `crates/he-db` (legacy MySQL) from `[workspace].members`.
  - Kept `crates/he-database` (Postgres) active.

Findings
--------
- Canonical core path:
  - `crates/he-core` (facade) re-exports `he-helix-core`. This is the preferred import surface for downstream crates.
- Duplicates outside `crates/`:
  - Multiple `he-core-*` crates exist at the repository root alongside `he-helix-*` variants.
  - The workspace currently includes `he-helix-*` crates as active members; `he-core-*` at repo root are not members (safe to leave for now).
- Backups and legacy copies:
  - `crates.backup/` contains copies; not part of the workspace (leave untouched).

Recommended Next Steps
----------------------
1. Dependency Normalization
   - Where feasible, point feature crates to depend on `he-core` (facade) instead of `he-helix-core` directly.
   - This allows swapping internals with minimal churn.

2. Facade Re-exports
   - For each feature area with a `he-helix-*` canonical crate, maintain a small `he-core-*` facade in `crates/` that re-exports it.
   - Update consumers to import from the `he-core-*` facade.

3. Decommission Legacy Members
   - After consumers are migrated, remove references to root-level legacy `he-core-*` crates from CI and builds.

4. CI Scope
   - Ensure CI builds only the active set of workspace members (postgreSQL + canonical helix modules).

