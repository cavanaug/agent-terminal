# Crate Directory Rename Design

## Summary

Rename the workspace crate directories from `crates/pilotty-core` and
`crates/pilotty-cli` to `crates/agent-terminal-core` and
`crates/agent-terminal-cli`.

This is an internal repository cleanup, not a public Cargo package rename.
The published package identities remain `agent-terminal-core` and
`agent-terminal-cli`, and Rust import spelling remains `agent_terminal_core`.

## Goals

- remove the remaining legacy `pilotty-*` crate directory names from the repo
- keep the public Cargo package names unchanged as `agent-terminal-*`
- keep runtime behavior, CLI behavior, protocol semantics, and release artifact
  naming unchanged
- update repository checks so the old crate paths are no longer treated as an
  allowed exception

## Non-Goals

- changing `[package].name` for any crate
- changing Rust crate import spelling such as `agent_terminal_core`
- changing binary names, CLI commands, runtime paths, or release artifact names
- adding a long-lived compatibility layer for old repository paths
- designing a crates.io migration or external consumer communication plan

## Current State

The workspace already publishes the intended package identities:

- `crates/pilotty-core/Cargo.toml` uses `name = "agent-terminal-core"`
- `crates/pilotty-cli/Cargo.toml` uses `name = "agent-terminal-cli"`

The remaining legacy surface is mostly on-disk structure:

- crate directories are still named `pilotty-*`
- `crates/pilotty-cli/Cargo.toml` still depends on `../pilotty-core`
- repository audits currently allow one legacy Cargo path as a documented
  exception

## Target State

After the rename:

- crate directories live at `crates/agent-terminal-core` and
  `crates/agent-terminal-cli`
- all workspace path references point at the new directories
- repository tests and audit scripts no longer allow legacy crate paths as a
  stable exception
- any remaining `pilotty` mentions are limited to historical origin notes or
  negative assertions that verify the old brand does not leak into active
  surfaces

## Design

### 1. Filesystem and Workspace Layout

Rename the crate directories directly on disk:

- `crates/pilotty-core` -> `crates/agent-terminal-core`
- `crates/pilotty-cli` -> `crates/agent-terminal-cli`

The workspace `members = ["crates/*"]` entry does not need structural changes;
the renamed directories will still be discovered automatically.

### 2. Cargo Manifest Updates

Update internal path dependencies to use the renamed directories.

Expected change:

- in the CLI crate manifest, change the core dependency path from
  `../pilotty-core` to `../agent-terminal-core`

Because package names are already correct, no `[package].name` changes may be
made.

### 3. Repository Reference Cleanup

Update all in-repo references that are sensitive to the old crate directory
names. This includes:

- test file paths under the renamed crate directories
- audit scripts and hardcoded allowlists that currently mention
  `crates/pilotty-*`
- any docs or helper scripts that embed the old crate directory paths

The cleanup is intentionally a clean repository break. The design does not
preserve old in-repo paths.

### 4. Repository Policy Update

`tests/verify-repo-branding-audit.sh` currently documents the old Cargo path as
an allowed legacy exception. After this rename, that exception must be
removed.

The audit policy must instead enforce that:

- legacy `pilotty` mentions are bounded to true historical references
- negative assertions in tests are still allowed where they prove the active
  product surfaces do not leak the old brand
- legacy crate directory paths are no longer present or allowlisted

### 5. Verification Strategy

The rename is complete only if the repository passes both Rust verification and
repository-specific audit checks.

Required verification:

- `cargo test --all`
- `cargo build --release`
- `tests/verify-repo-branding-audit.sh`
- `tests/verify-docs-release-identity.sh`
- final repository search for `pilotty-` and `crates/pilotty-`

## Implementation Sequence

1. rename crate directories on disk
2. update Cargo path references
3. update renamed-path references in tests, scripts, and docs
4. tighten branding-audit expectations to remove the legacy Cargo path
   exception
5. run verification commands and fix any stale references exposed by the checks

This sequence keeps the work focused on mechanical repository cleanup while
avoiding unrelated behavior changes.

## Risks

### Stale Path Assumptions

The largest risk is that repository tests or scripts still assume the old crate
directory names. This is more likely than a Cargo resolution failure because the
workspace package names already match the target branding.

Mitigation:

- update path-sensitive references immediately after the directory rename
- use repository-wide searches to catch any hardcoded old paths
- run the branding audit after the mechanical rename so policy drift is caught
  explicitly

### Over-Renaming Public Identity

There is a risk of accidentally changing public package identifiers or runtime
surfaces while cleaning up the repo.

Mitigation:

- treat `[package].name`, binary names, CLI surface, and runtime paths as frozen
  for this work
- review changes specifically for unexpected edits outside directory/path
  references

## Success Criteria

The design is successful when all of the following are true:

- the workspace uses `crates/agent-terminal-core` and
  `crates/agent-terminal-cli`
- no Cargo path dependency points at `pilotty-*`
- branding/repository audits do not allow legacy crate paths as an exception
- build and test verification pass without behavior changes
- remaining `pilotty` references are only historical provenance notes or
  negative assertions used for identity protection tests
