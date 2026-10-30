# 0012. Adopt Contributor Covenant 2.1 and GitHub private vulnerability reporting

- **Status:** Accepted
- **Date:** 2026-10-29
- **Deciders:** project maintainer (made during repo scaffolding)

## Context

The plan asks for a CODE_OF_CONDUCT, SECURITY policy and issue/PR templates, but doesn't say which code of conduct, how to report vulnerabilities, or which CI building blocks to use.

## Decision

- **Code of conduct:** Contributor Covenant 2.1, unmodified apart from the enforcement contact. The contact is a placeholder until a project address exists.
- **Security reports:** GitHub private vulnerability reporting; no email address needed. Must be enabled in the repository settings once the repo is on GitHub.
- **Issue forms** (YAML) for bugs and features. Bug reports ask explicitly about data loss, since data-loss bugs block every release.
- **PR template** checklist mirrors the non-negotiable rules in `CLAUDE.md`.
- **CI building blocks:** `dtolnay/rust-toolchain`, `Swatinem/rust-cache`, `pnpm/action-setup` (reads the pinned `packageManager`), `actions/setup-node` with Node 24.
- **Commit messages:** imperative summary with an optional area prefix (`feat:`, `fix:`, `docs:`, …). No enforced Conventional Commits tooling.

## Consequences

- Before going public: set the code-of-conduct contact, enable private vulnerability reporting, and replace `OWNER` in `.github/ISSUE_TEMPLATE/config.yml`.
