# Orca Next: microkernel + SDK

Everything is a plugin. The kernel (`orca-kernel`) has no product features; it provides only:

| Primitive | Purpose |
|---|---|
| Lifecycle | Dependency-ordered start, reverse stop, rollback on failed start |
| Commands | JSON-in/JSON-out, namespaced `<plugin-id>.<name>`; the public cross-plugin API (CLI, RPC and UI all call these) |
| Services | Typed in-process objects; visible only to the owner and plugins that list it in `requires` |
| Events | Topic-prefix pub/sub (`config.changed`, `worktree.listed`) |

`orca-sdk` is the only crate plugin authors depend on. `orca-cli` (`orca-next`) is a host that composes plugins.

## Plugin loading (deliberately undecided)

`Plugin` is transport-agnostic. Ways a plugin can be loaded, all of which can sit behind that trait:

- **Static**: compiled into the host binary (today; all built-ins).
- **WASM component** (wasmtime + WIT): sandboxed, any language, permissions enforced by the host. Best fit for third-party plugins.
- **Out-of-process** (JSON-RPC over stdio, like LSP/MCP): simplest, language-agnostic, higher call overhead. Good for agent/PTY-heavy plugins.

Because commands and events are JSON, the WASM and process loaders are themselves plugins that wrap foreign code as a `Plugin`.

## Roadmap: each line is an independent plugin

- `core.config` ✅ · `worktree` ✅ (list)
- `pty` (terminals) · `agents` (Codex/Claude/OpenCode launchers) · `git` (status/diff/branches)
- `github` / `gitlab` / `linear` / `jira` (providers) · `browser` · `mobile-bridge`
- `rpc-server` (exposes commands to the existing Electron UI, enabling strangler migration) · `loader-wasm` · `loader-process`
- `manifest` loader to replace `src/shared/plugins` semantics

## Next kernel work

Capability permissions in manifests (fs/net/spawn) · versioned command schemas · config-driven plugin discovery · tracing.
