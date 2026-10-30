# RFCs

An RFC ("request for comments") is a written proposal for a change that other people will build on and that is expensive to undo. In Vault that means:

- the **config format** — settings, keymaps, themes (TOML + JSON Schema);
- the **plugin API** — anything Lua plugins can call or implement;
- the **`vlt` CLI** interface, once other tools depend on it.

Everything else — bug fixes, internal refactors, new features that don't change those surfaces — goes straight to a PR. Architecture decisions that aren't public-API changes go in an [ADR](../adr/README.md).

## Process

1. **Discuss first (optional).** Open a GitHub Discussion to test the idea before writing it up.
2. **Write it.** Copy the outline below into `docs/rfcs/NNNN-short-title.md` (next free number) and open a PR titled `RFC: <title>`.
3. **Comment period.** The RFC stays open for at least **7 days** of review. Revise it in the PR as feedback comes in.
4. **Decide.** A maintainer marks it *Accepted*, *Rejected* or *Postponed* in the status line and merges the PR either way, so the reasoning is kept.
5. **Implement.** Accepted RFCs are implemented in separate PRs that link to the RFC. Record the outcome in an ADR and update the affected docs (e.g. [ARCHITECTURE.md](../ARCHITECTURE.md), plugin API reference).

After the Phase 3 gate the plugin API is **frozen**: breaking changes need an RFC and a migration plan, and land only in a major version.

## Outline

```markdown
# NNNN. Title

- **Status:** Draft | Accepted | Rejected | Postponed
- **Author:** @handle
- **Discussion:** link

## Summary
One paragraph.

## Motivation
What problem this solves and for whom.

## Design
The proposed config keys / API / CLI, with examples.

## Compatibility
What breaks, and the migration path.

## Performance
Effect on the budgets in docs/PERFORMANCE.md.

## Alternatives
What else was considered and why not.

## Unresolved questions
```

## Index

No RFCs yet.
