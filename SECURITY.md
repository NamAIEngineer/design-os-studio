# Security Policy (`design-os-studio`)

## Supported Versions

| Version | Supported |
| :--- | :--- |
| `0.2.x` / `main` | Yes |

## Security Model

- `design-os-studio` executes locally with **zero external telemetry** (see [`PRIVACY.md`](PRIVACY.md)).
- The Model Context Protocol server (`designos-mcp`) runs over local `stdio` scoped to your explicit `--workspace` directory.
- The embedded Agentic PTY Chat Runtime (`crates/designos-app/src/terminal.rs`) runs your own local CLI (`claude`, `codex`, `agy`) inside a user-controlled local pseudo-terminal without storing or transmitting API keys.
- Diagnostic commands (`/doctor` and `python3 scripts/self_check.py`) are strictly read-only verification checks.

## Reporting a Vulnerability

Please report suspected vulnerabilities via [GitHub Security Advisories](https://github.com/jangtrinh/design-os-studio/security/advisories/new) rather than public issues. We acknowledge reports within 48 hours.
