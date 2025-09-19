Consolidating he-* and he-helix-* Crates
========================================

Goal
----
Reduce drift and duplication between legacy `he-*` and newer `he-helix-*` crates by introducing a single canonical set, well-defined re-exports, and a staged migration path that avoids breaking changes.

Strategy (Phased)
-----------------
1. Canonical Core
   - Designate `he-helix-core` as canonical core implementation.
   - Keep `crates/he-core` as the only stable facade re-exporting `he-helix-core` (as already done), and route all other internal crates to depend on `he-core` only.

2. Re-export Facades for Feature Areas
   - For duplicated feature crates (e.g., entity, account, process), create small facade crates in `crates/` that depend on the canonical `he-helix-*` implementation and re-export its public API.
   - Example: `he-core-entity` re-exports `he-helix-entity`.

3. Workspace Members
   - In `Cargo.toml` workspace, mark only facades and canonical `he-helix-*` crates as members. Exclude legacy duplicates from the workspace to avoid build/test bloat.
   - If external paths refer to legacy crates, keep them compiled temporarily but not used by primary binaries.

4. Code Moves (N-to-1)
   - Incrementally move duplicated modules (e.g., models, actors, genserver) from `he-core-*` into their corresponding `he-helix-*` crates.
   - Update facade crates to re-export moved items; update call sites to import from facades.

5. CI/Tests
   - Run tests per feature area after each move. Ensure no cross-feature or cyclical deps.
   - Stop building legacy duplicates in CI once all consumers are migrated.

6. Deprecate and Remove
   - Once all consumers import via the `he-core` facade or the canonical `he-helix-*`, remove the legacy crate directories.

Guidelines
----------
- Do not rename runtime crates published externally; use re-exporting facades to preserve public API.
- Avoid breaking versions; keep semantic versioning and add deprecation notices in README of deprecated crates.
- Prefer `he-core` facade as the single import point for application crates; only feature crates should import from `he-helix-*` directly.

Immediate Low-Risk Steps
------------------------
- Audit all `Cargo.toml` to ensure they depend on `he-core` where possible instead of both `he-core` and `he-helix-core`.
- Remove duplicate workspace members from root workspace if not used by binaries/tests.
- Add this plan to the repository to coordinate PRs.

