# 0006. Name the companion CLI `vlt`

- **Status:** Accepted
- **Date:** 2026-10-29
- **Deciders:** project maintainer

## Context

Vault ships a companion command-line tool: `vlt .` opens Vault from a terminal, and a file-picker mode lets other tools use it ([FEATURES.md](../FEATURES.md)). The obvious name, `vault`, is already HashiCorp Vault's CLI and is installed on many developer machines.

## Decision

The CLI binary is **`vlt`**. It lives in `crates/cli` (package `vault-cli`, binary `vlt`). Like plugins, it goes through the same core APIs, operations queue and undo log.

## Alternatives considered

- **`vault`** — clashes with HashiCorp Vault on `PATH`.
- **`vaultx`, `vx`** — `vx` is too short and generic; `vaultx` is longer to type for no gain.

## Consequences

- Docs, help text and package manifests (Winget, Homebrew, AUR) use `vlt`.
- Package-name availability for "Vault" and `vlt` in each registry is still an open question ([STATUS.md](../STATUS.md)).
