# 0005. Dual-license under MIT and Apache-2.0

- **Status:** Accepted (final confirmation required before the first public release)
- **Date:** 2026-10-29
- **Deciders:** project maintainer

## Context

Vault is open source and built on the Rust ecosystem, where MIT OR Apache-2.0 is the norm. We want maximal reuse of the core crates and an explicit patent grant for contributors and users.

## Decision

All Vault code is available under **MIT OR Apache-2.0**, at the user's option. Every crate and package declares `license = "MIT OR Apache-2.0"`. The repo carries `LICENSE-MIT` and `LICENSE-APACHE`. Contributions are accepted under the same terms (see [CONTRIBUTING.md](../../CONTRIBUTING.md)).

## Alternatives considered

- **GPL-3.0** — protects against proprietary forks, but limits reuse of the core crates and is unusual in the Rust ecosystem.
- **MIT only** — simpler, but no explicit patent grant.
- **Apache-2.0 only** — patent grant, but incompatible with GPLv2-only projects.

## Consequences

- Dependencies must be compatible with dual MIT/Apache distribution. A license check (e.g. `cargo-deny`) should be added to CI before the first release.
- Confirming the license is an open question tracked in [STATUS.md](../STATUS.md).
