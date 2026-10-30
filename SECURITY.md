# Security policy

## Supported versions

Vault is **pre-alpha**. There are no supported releases yet; security fixes land on `main`.

## Reporting a vulnerability

**Please do not open a public issue.**

Report vulnerabilities privately through GitHub's [private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability) ("Report a vulnerability" on the repository's **Security** tab).

Please include:

- what an attacker can do, and under which conditions;
- steps or a proof of concept to reproduce;
- affected version or commit, and OS.

We aim to acknowledge reports within 7 days and to agree on a disclosure timeline with you.

## Scope

Areas we care most about:

- anything that can delete, overwrite or corrupt files without the user's intent (including bypassing the undo journal or trash);
- privilege escalation through the optional elevated indexing helper on Windows;
- plugins (Lua) escaping the plugin API to access the file system or run code directly;
- the webview reaching the disk or the OS without going through Vault's IPC commands.
