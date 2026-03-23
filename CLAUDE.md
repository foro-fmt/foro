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

## dll-pack download/load separation

dll-pack's current `resolve()` does download + path resolution in one shot via `cached_download_lib` / `cached_download_manifest`. This means any caller of `resolve()` can silently trigger a network download. This is the root cause of the "waits a while then auto-downloads" behavior.

The correct design requires two distinct operations in dll-pack:

- **`install(url, work_dir, platform)`** — downloads all manifests and libraries for a plugin. Called only from `foro install`. This is the only legitimate download path.
- **`load(url, work_dir, platform)`** — assumes all files are already on disk. If any file is missing, it must return a hard error (or panic) immediately. Never downloads. Called by the daemon when it needs to use a plugin.

Until dll-pack is redesigned this way, any call to `resolve()` from the daemon or format logic is a latent bug. When fixing this, change dll-pack first, then update foro's daemon to use `load()` instead of `resolve()`.
