# Maintaining foro Plugins

This document describes the full procedure for updating all foro formatter plugins to track their latest upstream versions and publishing new releases. It is written as a manager guide — the person running this document orchestrates multiple Codex CLI sub-agents, one per plugin, and monitors their progress.

## Plugin Inventory

| Plugin | Language | Upstream | Repo | WASM | Notes |
|--------|----------|----------|------|------|-------|
| foro-rustfmt | Rust | rust-lang/rustfmt | foro-fmt/foro-rustfmt | No | rustc_private, nightly toolchain |
| foro-biome | JS/TS/CSS/HTML | biomejs/biome | foro-fmt/foro-biome | Yes | wasm32-wasip1 target included |
| foro-ruff | Python | astral-sh/ruff | foro-fmt/foro-ruff | Yes | wasm32-wasip1 target included |
| foro-clang-format | C/C++ | llvm/llvm-project | foro-fmt/foro-clang-format | No | C++ only, CMake+FetchContent, no Rust |
| foro-tombi | TOML | nahco314/tombi | foro-fmt/foro-tombi | No | tracks `foro-tombi` branch |

Local clones are in `../foro-<name>` relative to this repo (i.e., `~/RustroverProjects/foro-<name>`).

Each plugin has its own `UPDATE_GUIDE.md` in its local clone that explains the plugin-specific update procedure. Always read it before issuing instructions to a sub-agent.

## Versioning Policy

All plugins share the same versioning rule. Plugin versions are GitHub release tag identifiers, not semver coordinates — semver ordering semantics are intentionally not preserved.

### Rule

**Case 1 — upstream tracking release** (the normal case: upgrading to a new upstream formatter version):

Use the upstream formatter's version as the plugin tag verbatim.

```
biome 2.4.8     → foro-biome tag  2.4.8
ruff 0.11.5     → foro-ruff tag   0.11.5
LLVM 22.1.1     → foro-clang-format tag  22.1.1
rustfmt 1.9.0   → foro-rustfmt tag  1.9.0
tombi 0.5.2     → foro-tombi tag  0.5.2
```

**Case 2 — plugin-only fix** (bug fix, foro ABI change, packaging fix, workflow fix — upstream version unchanged):

Append `-<n>` to the last upstream version. `n` starts at 1 and increments per fix.

```
2.4.8-1, 2.4.8-2, ...
0.11.5-1, ...
```

When the next upstream tracking release happens, the suffix resets — the new bare version is used.

### What goes into Cargo.toml / CMakeLists.txt

The `version` field in `Cargo.toml` (or `project(VERSION ...)` in `CMakeLists.txt`) must always match the tag being released, including the `-<n>` suffix when applicable.

### Why not semver

Strict semver and upstream-version-tracking are incompatible: plugin-only patch bumps would collide with future upstream patch releases. We treat version tags as opaque human-readable identifiers that encode "which upstream, which plugin fix iteration". The `-<n>` suffix looks like a semver pre-release but is not intended as one.

## CI / Release Model (All Plugins)

All plugins share the same two-stage CI model:

1. **Release Verify** — triggered by `pull_request` or `workflow_dispatch`. Builds all platforms and packages, but does not publish a GitHub Release. Used to validate changes before merging.
2. **Release** — triggered by a pushed version tag only. Identical build, but additionally runs the `host` job to publish a GitHub Release with all platform artifacts.

The task is complete only when:
- All `build-local-artifacts` jobs succeed on every platform.
- `build-global-artifacts` succeeds.
- `host` succeeds (Release workflow only).
- A GitHub Release exists for the new tag with the expected assets and proper release notes.

**Release notes must be written before pushing the tag.** Each plugin repo contains a `RELEASE_NOTES.md` file that CI reads when creating the GitHub Release. CI fails if the file still contains the placeholder text. The sub-agent must write proper release notes into `RELEASE_NOTES.md` and commit it before pushing the version tag. See each plugin's UPDATE_GUIDE.md for content guidelines.

## Tools Required

These CLIs must be available:

- `codex` — Codex CLI for launching and resuming AI sub-agent sessions
- `gh` — GitHub CLI for inspecting CI runs and releases
- `git`
- `python3` or `jq` — for parsing JSONL output from Codex

## Step-by-Step Procedure

### Step 1: Decide Which Plugins Need Updating

Check each plugin repo for:
- Whether the upstream formatter has a new release since the last plugin version.
- Whether the release workflow is broken or needs fixing.

```bash
# Quick overview of each plugin's latest release
for plugin in foro-rustfmt foro-biome foro-ruff foro-clang-format foro-tombi; do
    echo "=== $plugin ==="
    gh release list --repo foro-fmt/$plugin --limit 3
done
```

Read the `UPDATE_GUIDE.md` for each plugin that needs updating to understand its specific requirements before issuing instructions.

### Step 2: Prepare Codex Session Output Directory

```bash
mkdir -p /tmp/codex-sessions
```

### Step 3: Launch One Codex Sub-Agent Per Plugin

Launch one Codex CLI instance per plugin that needs updating. Use `--dangerously-bypass-approvals-and-sandbox` so the agent does not pause for permission prompts. Use `--json -o <file>` to stream structured output to a JSONL file for monitoring.

The working directory (`-C`) must be the plugin's local clone.

Template command:

```bash
codex exec \
  --dangerously-bypass-approvals-and-sandbox \
  -C /home/nahco314/RustroverProjects/<plugin-name> \
  --json \
  -o /tmp/codex-sessions/<plugin-name>.jsonl \
  "$(cat /home/nahco314/RustroverProjects/<plugin-name>/UPDATE_GUIDE.md)

Follow the UPDATE_GUIDE.md above exactly. Work autonomously without asking for permission. The task is complete only when a GitHub Release exists for the new tag with all expected platform assets, all CI jobs have succeeded, and the GitHub Release body has been updated with end-user release notes." &
```

Example — launching all five agents in parallel:

```bash
mkdir -p /tmp/codex-sessions

for plugin in foro-rustfmt foro-biome foro-ruff foro-clang-format foro-tombi; do
    codex exec \
        --dangerously-bypass-approvals-and-sandbox \
        -C /home/nahco314/RustroverProjects/$plugin \
        --json \
        -o /tmp/codex-sessions/$plugin.jsonl \
        "$(cat /home/nahco314/RustroverProjects/$plugin/UPDATE_GUIDE.md)

Follow the UPDATE_GUIDE.md above exactly. Work autonomously without asking for permission. The task is complete only when a GitHub Release exists for the new tag with all expected platform assets, all CI jobs have succeeded, and the GitHub Release body has been updated with end-user release notes." &
done
```

Each agent runs independently. You do not need to wait for one to finish before starting the next.

### Step 4: Obtain Thread IDs for Session Resumption

Codex emits a `thread.started` event near the beginning of each JSONL file. Extract the thread ID if you need to resume a session later (e.g., to ask for a status update or inject a correction).

```bash
python3 -c "
import json, sys, os

for plugin in ['foro-rustfmt', 'foro-biome', 'foro-ruff', 'foro-clang-format', 'foro-tombi']:
    path = f'/tmp/codex-sessions/{plugin}.jsonl'
    if not os.path.exists(path):
        print(f'{plugin}: not started yet')
        continue
    with open(path) as f:
        for line in f:
            try:
                obj = json.loads(line)
                if obj.get('type') == 'thread.started':
                    print(f'{plugin}: thread_id = {obj[\"thread_id\"]}')
                    break
            except:
                pass
"
```

To resume a session and send a follow-up message:

```bash
codex --dangerously-bypass-approvals-and-sandbox \
    -C /home/nahco314/RustroverProjects/<plugin-name> \
    --json \
    -o /tmp/codex-sessions/<plugin-name>.jsonl \
    --thread-id <thread-id> \
    "Your follow-up message here."
```

### Step 5: Monitor Progress (1-Minute Polling Loop)

Poll both the Codex JSONL output and GitHub CI status every 60 seconds. Do not stop polling until all plugins show a completed GitHub Release.

#### Read latest agent messages from JSONL

```bash
python3 -c "
import json, os

for plugin in ['foro-rustfmt', 'foro-biome', 'foro-ruff', 'foro-clang-format', 'foro-tombi']:
    path = f'/tmp/codex-sessions/{plugin}.jsonl'
    if not os.path.exists(path):
        print(f'=== {plugin}: not started ===')
        continue
    events = []
    with open(path) as f:
        for line in f:
            try: events.append(json.loads(line.strip()))
            except: pass
    completed = [e for e in events if e.get('type') == 'task_complete']
    msgs = [e for e in events if e.get('item', {}).get('type') == 'agent_message' and e.get('type') == 'item.completed']
    last = msgs[-1]['item']['text'][:400] if msgs else '(no messages yet)'
    status = 'DONE' if completed else 'running'
    print(f'=== {plugin} [{status}] ===')
    print(last)
    print()
"
```

#### Check CI run status directly

For plugins in the middle of CI (Release Verify or Release), check GitHub directly:

```bash
# Find the latest Release run for a plugin
gh run list --repo foro-fmt/<plugin-name> --workflow Release --limit 3 --json databaseId,status,conclusion,displayTitle

# Inspect jobs in a specific run
gh run view <run_id> --repo foro-fmt/<plugin-name> --json status,conclusion,jobs | python3 -c "
import json, sys
d = json.load(sys.stdin)
print('status:', d['status'], 'conclusion:', d['conclusion'])
for j in d['jobs']:
    print(f'  {j[\"name\"]}: {j[\"status\"]} / {j[\"conclusion\"]}')
"
```

#### Verify a completed release

```bash
gh release view <tag> --repo foro-fmt/<plugin-name> --json tagName,url,assets | python3 -c "
import json, sys
d = json.load(sys.stdin)
print('Tag:', d['tagName'])
print('URL:', d['url'])
print('Assets:', [a['name'] for a in d['assets']])
"
```

#### Automated 1-minute polling shell script

Save this as `/tmp/codex-sessions/monitor.sh` and run it in the background, or invoke it manually each minute:

```bash
#!/bin/bash
set -euo pipefail

PLUGINS=(foro-rustfmt foro-biome foro-ruff foro-clang-format foro-tombi)

while true; do
    echo "===== $(date) ====="
    for plugin in "${PLUGINS[@]}"; do
        path="/tmp/codex-sessions/$plugin.jsonl"
        if [ ! -f "$path" ]; then
            echo "[$plugin] not started"
            continue
        fi
        # Latest agent message
        python3 -c "
import json, sys
events = []
with open('$path') as f:
    for line in f:
        try: events.append(json.loads(line.strip()))
        except: pass
completed = any(e.get('type') == 'task_complete' for e in events)
msgs = [e for e in events if e.get('type') == 'item.completed' and e.get('item',{}).get('type') == 'agent_message']
last = msgs[-1]['item']['text'][:300] if msgs else '(no messages yet)'
print(f'[$plugin] {\"DONE\" if completed else \"running\"}: {last}')
"
    done
    echo ""
    sleep 60
done
```

### Step 6: Handle Failures

If a Codex agent stops or gets stuck, check its JSONL for an error event, then resume via `--thread-id`.

If CI fails and the agent is not reacting:
1. Read the failing job log: `gh run view <run_id> --repo foro-fmt/<plugin-name> --job <job_id> --log`
2. Identify the failure category (see each plugin's `UPDATE_GUIDE.md` — Failure Triage section).
3. Resume the agent with a message explaining what failed and what it should fix.

Common categories:
- **Upstream API changed** — update plugin code or dependency version.
- **Workflow/runner issue** — fix `.github/workflows/release*.yml`.
- **dll-pack-builder broke** — fix in `foro-fmt/dll-pack-builder`, push, pin the new commit hash in both workflow files.

### Step 7: Confirm All Releases Published

When all agents report task completion, verify each release exists:

```bash
for plugin in foro-rustfmt foro-biome foro-ruff foro-clang-format foro-tombi; do
    echo "=== $plugin ==="
    gh release list --repo foro-fmt/$plugin --limit 1
done
```

## Plugin-Specific Notes

### foro-rustfmt
- Native-only (no WASM). Uses `rustc_private`, requires a specific nightly toolchain.
- Toolchain pin is in `rust-toolchain.toml`. Must match upstream `rustfmt`'s required nightly.
- Reference workflow: already the gold standard. Other plugins are modeled after it.

### foro-biome
- Supports both native (4 platforms) and WASM (`wasm32-wasip1`). 5 `build-local-artifacts` jobs total.
- Dependency is pinned via `git tag` in `Cargo.toml` (e.g., `tag = "@biomejs/biome@2.4.8"`).
- Update `Cargo.toml` tag and bump the crate version. Run `cargo update` to refresh `Cargo.lock`.

### foro-ruff
- Supports both native (4 platforms) and WASM (`wasm32-wasip1`). 5 `build-local-artifacts` jobs total.
- Dependency is pinned via git `rev` in `Cargo.toml`.
- Get the latest commit from `astral-sh/ruff` and update the `rev`.

### foro-clang-format
- **Pure C++. No Rust code.** Uses CMake + FetchContent to download and compile LLVM/clang.
- Build is slow (LLVM compilation takes 30+ minutes per platform in CI).
- Update `LLVM_VERSION` and `URL_HASH SHA256=...` in `CMakeLists.txt`.
- Get the correct SHA256 by downloading the tarball: `curl -sL <url> | sha256sum`.
- `BUILD_OUT_DIR` is `./build` (not Rust `target/`).
- **GLIBC compatibility**: The Linux build must use `container: 'buildpack-deps:focal'` (Ubuntu 20.04) on the `ubuntu-24.04` runner, same as foro-rustfmt. Without this, the `.so` will be linked against GLIBC 2.38+ (Ubuntu 24.04's default) and will fail to load on older systems. The container runs as root, so `apt-get` is used without `sudo`.

### foro-tombi
- Tracks the `foro-tombi` branch of `nahco314/tombi` (not the main branch).
- Dependencies are pinned via git `rev` in `Cargo.toml`.
- Get the latest commit from that branch and update all `tombi-*` crate revs together.
- Native-only (no WASM).

## Packaging Helper: dll-pack-builder

All Rust-based plugins use `foro-fmt/dll-pack-builder` (a Python tool). It is installed via:

```yaml
run: uv tool install git+https://github.com/foro-fmt/dll-pack-builder@<commit>
# or on Windows/older runners:
run: python3 -m pip install git+https://github.com/foro-fmt/dll-pack-builder@<commit>
```

Always pin an exact commit hash, never a floating branch ref.

Current pinned commit (as of 2026-03-21): `1e90b63`

If `dll-pack-builder` itself needs fixing:
1. Clone `https://github.com/foro-fmt/dll-pack-builder`.
2. Fix the smallest thing that is wrong.
3. Push to that repo.
4. Update the pinned commit in both `release-verify.yml` and `release.yml` of the affected plugin.

## Step 8: Update the foro Repository Itself

After all plugin releases are published, the foro repo must be updated to reference the new versions. This step is easy to forget — do it before closing the session.

### 8a. Update `src/config/default_config.json`

Each plugin entry has a hardcoded `.dllpack` URL containing the version tag. Bump each URL that was updated.

```json
{
  "rules": [
    {
      "on": [".ts", ".tsx", ".js", ...],
      "cmd": "https://github.com/foro-fmt/foro-biome/releases/download/<NEW_TAG>/foro-biome.dllpack"
    },
    {
      "on": ".rs",
      "cmd": "https://github.com/foro-fmt/foro-rustfmt/releases/download/<NEW_TAG>/foro-rustfmt.dllpack"
    },
    ...
  ]
}
```

Edit `src/config/default_config.json` and replace the old tag in each URL with the new tag.

### 8b. Update Test Fixtures

If the new plugin version produces different output for the same input, the expected output files in `tests/fixtures/` must be updated to match.

Run the format tests locally after updating `default_config.json` to see which fixtures (if any) differ:

```bash
cargo test --test cli_format 2>&1 | grep -A 20 "thread .* panicked"
```

For each fixture that differs, run foro on the input file and capture the new expected output:

```bash
# Example: regenerate a fixture
cargo run -- format tests/fixtures/<plugin>/input.<ext>
cp tests/fixtures/<plugin>/input.<ext> tests/fixtures/<plugin>/output.<ext>
```

Review the diff carefully before committing — a formatter behavior change should be intentional.

### 8c. Update Benchmarks

Update the benchmark comparison tools to match the newly released plugin versions, then re-run.

**biome** — update the npm package in `benchmark/biome-test/`:

```bash
cd benchmark/biome-test
npm install @biomejs/biome@<NEW_BIOME_VERSION>
```

**ruff** — upgrade the system tool:

```bash
uv tool upgrade ruff
# Verify: ruff --version
```

**clang-format** — the benchmark uses the Ubuntu system package (`apt install clang-format`). Updating to a specific LLVM version requires manual steps; do so when the Ubuntu package lags significantly behind the plugin version.

Re-run and capture results:

```bash
cd benchmark
uv run ./run.py 2>&1 | tee ./run_result.txt
```

Review the output and confirm foro is still faster than the baseline tools.

### 8d. Commit and Push

```bash
git add src/config/default_config.json tests/fixtures/ benchmark/
git commit -m "chore(plugins): bump plugin versions to <summary>"
git push
```

Verify CI passes on the foro repo before considering the update complete.

---

## Example: Updating All Plugins (Full Session)

```bash
# 1. Prepare output directory
mkdir -p /tmp/codex-sessions

# 2. Launch sub-agents (example: only plugins that need updating)
for plugin in foro-biome foro-ruff foro-clang-format foro-tombi; do
    codex exec \
        --dangerously-bypass-approvals-and-sandbox \
        -C /home/nahco314/RustroverProjects/$plugin \
        --json \
        -o /tmp/codex-sessions/$plugin.jsonl \
        "$(cat /home/nahco314/RustroverProjects/$plugin/UPDATE_GUIDE.md)

Follow the UPDATE_GUIDE.md above exactly. Work autonomously. The task is complete only when a GitHub Release exists for the new tag with all expected assets, all CI jobs have succeeded, and the GitHub Release body has been updated with end-user release notes." &
done

# 3. Monitor (run in a loop; Ctrl-C when all done)
while true; do
    echo "===== $(date) ====="
    for plugin in foro-biome foro-ruff foro-clang-format foro-tombi; do
        latest_run=$(gh run list --repo foro-fmt/$plugin --workflow Release --limit 1 --json databaseId,status,conclusion --jq '.[0] | "\(.status)/\(.conclusion) id=\(.databaseId)"' 2>/dev/null || echo "no run")
        echo "[$plugin] CI: $latest_run"
    done
    sleep 60
done
```
