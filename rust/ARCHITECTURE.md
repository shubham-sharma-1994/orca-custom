# Orca Next: microkernel + SDK

Everything is a plugin. The kernel (`orca-kernel`) has no product features; it provides only:

| Primitive | Purpose |
|---|---|
| Lifecycle | Dependency-ordered start, reverse stop, rollback on failed start |
| Commands | JSON-in/JSON-out, namespaced `<plugin-id>.<name>`; the public cross-plugin API (CLI, RPC and UI all call these) |
| Services | Typed in-process objects; visible only to the owner and plugins that list it in `requires` |
| Events | Topic-prefix pub/sub (`config.changed`, `worktree.listed`) |

`orca-sdk` is the only crate plugin authors depend on. `orca-cli` (`orca-next`) is a host that composes plugins.

## Product thesis

A VS Code for agents, in Rust, with Orca (the TS app in the repo root) as the feature reference. Mapping:

| VS Code | Orca Next |
|---|---|
| Extension host + API | `orca-kernel` + `orca-sdk` |
| Extensions | Rust crates implementing `Plugin` (built-ins and third-party alike) |
| Commands / command palette | kernel commands (JSON in/out) |
| Contribution points (views, keybindings, settings) | manifest contributions (next kernel work) |
| Activation events | lazy start on first command/event (next kernel work) |
| Workbench UI | a UI plugin (Rust-native) consuming commands/events |

## Plugins are Rust only

No WASM or cross-language loaders. Rust has no stable ABI, so a plugin is a crate linked into the host:

- **Now**: plugins are workspace crates composed in the host binary, selectable by cargo feature.
- **Later (third-party)**: either `cargo install`-style builds of a custom host, or `abi_stable` dylibs behind the same `Plugin` trait. Decide when the first external plugin appears; the trait does not change.

## Orca feature roadmap, one plugin each

- done: `core.config`, `worktree` (list)
- `pty` terminals · `agents` (Codex/Claude/OpenCode/Pi launchers + status) · `git` (status/diff/branches/commit)
- `github`, `gitlab`, `linear`, `jira` · `browser` · `automations` · `mobile-bridge` (RPC for the phone app)
- `rpc-server`: expose commands over a socket so the existing Electron UI can migrate feature by feature
- `ui`: Rust-native workbench (layout, tabs, sidebar, palette)

## Next kernel work

Manifest contributions (commands metadata, settings, keybindings, views) · activation events (lazy start) · capability permissions · versioned command schemas · tracing.
