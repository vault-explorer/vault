# 0013. Use `io.github.vault-explorer.vault` as the bundle identifier

- **Status:** Accepted
- **Date:** 2026-10-30
- **Deciders:** project maintainer
- **Supersedes:** the bundle-identifier part of [0008](0008-workspace-layout-and-naming.md)

## Context

Tauri needs a reverse-DNS bundle identifier (`identifier` in `app/src-tauri/tauri.conf.json`). It names the app's data and config folders (e.g. `%APPDATA%\<identifier>` on Windows) and identifies the app to the OS on macOS and Linux. Changing it after users have installed Vault strands their settings, so it has to be fixed before the first public build. ADR 0008 used `dev.vaultexplorer.vault` as a placeholder because the project had no domain or GitHub org.

The project now lives at `github.com/vault-explorer/vault`. It still has no domain of its own.

## Decision

The bundle identifier is **`io.github.vault-explorer.vault`**.

It follows the usual reverse-DNS convention for projects hosted on GitHub (`io.github.<owner>.<repo>`). The project controls that namespace through the GitHub org, so no domain purchase is needed.

## Alternatives considered

- **`dev.vaultexplorer.vault`** (the placeholder): implies a domain the project doesn't own.
- **Buy a domain first**: blocks on an open question (STATUS.md) for no technical gain. A later domain doesn't require changing the identifier.

## Consequences

- Windows app data will live under `%APPDATA%\io.github.vault-explorer.vault` and `%LOCALAPPDATA%\io.github.vault-explorer.vault`. Nobody has a public build yet, so nothing needs migrating.
- The hyphen is valid for Windows, macOS and Linux desktop identifiers. Android package names don't allow hyphens, but mobile is a non-goal.
- Treat the identifier as permanent from the first public release; changing it later needs a migration plan and a new ADR.
- `repository` is now set in `[workspace.package]`, which resolves the other org-dependent placeholder ADR 0008 listed.
