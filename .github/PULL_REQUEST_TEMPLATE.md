## What and why

<!-- What does this change, and why? Link the issue, backlog item, ADR or RFC. -->

## How it was tested

<!-- Commands run, OS (Windows 10 / 11), manual steps. -->

## Checklist

- [ ] `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` pass
- [ ] `pnpm lint`, `pnpm format:check`, `pnpm typecheck` pass (if the UI changed)
- [ ] No performance budget regresses (docs/PERFORMANCE.md) — benchmark numbers attached if relevant
- [ ] The UI does not touch the disk; ops that can exceed 100 ms are cancellable background tasks
- [ ] Destructive operations are journaled and undoable
- [ ] Platform-specific code is only in `crates/platform`, behind its traits
- [ ] Docs updated (docs/STATUS.md and the relevant doc; ADR/RFC if a decision or public surface changed)
