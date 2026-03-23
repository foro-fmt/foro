Related projects (plugins such as foro-biome, dllpack, and other projects under foro-fmt on GitHub) are located in ../.
For example, foro-biome is in ../foro-biome. You are free to read, edit, commit, and push them.

These related repos (dll-pack, foro-biome, foro-rustfmt, foro-ruff, foro-clang-format, foro-tombi, etc.) are all part of the foro project — they just live in separate repositories. Treat them as first-class foro code: edit directly, no workarounds needed. Never propose a workaround in foro when the correct fix is in one of these repos.

About plugin management, read ./MAINTAIN_PLUGINS.md .

About releasing foro, read ./RELEASE.md .

## Architectural principles

**Client vs daemon responsibilities**: foro is accessed via the CLI and also directly via daemon communication (e.g. IDE plugins). Any logic placed only in the CLI layer is invisible to non-CLI callers. Substantial processing — plugin loading, validation, formatting — must live in the daemon. The client is responsible only for argument parsing, launching/connecting to the daemon, and forwarding results. When deciding where logic belongs, always ask: "does this need to work when called outside the CLI?"

**Single responsibility per operation**: Operations that are conceptually distinct (e.g. downloading vs. loading vs. resolving dependencies) must be separate code paths. Coupling them causes unintended side effects — for example, a "load" that silently downloads on cache miss will trigger network I/O from contexts where that is never expected. Each operation should do exactly one thing and fail loudly if its preconditions aren't met.

## Notes on this file

CLAUDE.md is for general architectural principles and project-wide conventions that are not derivable from reading the code. Do not record individual bugs, specific code locations, or current implementation details here — those belong in code comments, commit messages, or issue trackers, and writing them here wastes context window. Concrete examples to illustrate a principle are fine, but keep them brief.
