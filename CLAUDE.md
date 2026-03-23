Related projects (plugins such as foro-biome, dllpack, and other projects under foro-fmt on GitHub) are located in ../.
For example, foro-biome is in ../foro-biome. You are free to read, edit, commit, and push them.

These related repos (dll-pack, foro-biome, foro-rustfmt, foro-ruff, foro-clang-format, foro-tombi, etc.) are all part of the foro project — they just live in separate repositories. Treat them as first-class foro code: edit directly, no workarounds needed. Never propose a workaround in foro when the correct fix is in one of these repos.

About plugin management, read ./MAINTAIN_PLUGINS.md .

About releasing foro, read ./RELEASE.md .

## Client vs Daemon responsibilities

foro has two access paths: the CLI (`foro format`) and direct daemon communication (used by IDE plugins, editor integrations, etc.). Any logic placed only in the CLI layer is invisible to non-CLI callers.

**Rule: all substantial processing must live in the daemon, not the client.**

The client (CLI) is responsible only for:
- Parsing arguments and user-facing flags
- Launching/connecting to the daemon
- Forwarding requests and printing results

The daemon is responsible for:
- Loading plugins
- Checking whether plugins are installed
- Running formatters

**Plugin installation checks must be in the daemon.** Putting `check_ready` in `format.rs` (the CLI handler) means IDE plugins that talk directly to the daemon bypass the check entirely, causing freezes or silent failures when plugins are not downloaded.

**`foro install` is the only legitimate download path.** dll-pack's `resolve()` silently downloads plugins on demand as a side effect. Any code path that reaches `resolve()` outside of `foro install` is a bug — the daemon must panic or return a hard error if it encounters a plugin that is not already present on disk. Never silently trigger a download from the daemon or from format logic.
