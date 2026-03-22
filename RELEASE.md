# Releasing foro

foro uses [cargo-dist](https://github.com/nahco314/cargo-dist) for releases.
Pushing a version tag triggers the Release CI, which builds binaries for all
target platforms and creates a GitHub Release automatically.

## Checklist

1. **依存クレートの確認**
   - `dll-pack` を local path dep で開発していた場合は、git rev に戻す:
     ```toml
     dll-pack = { version = "0.3.0", git = "https://github.com/foro-fmt/dll-pack", rev = "<commit-sha>" }
     ```
   - `cargo build` でエラーがないことを確認する。

2. **テストを通す**
   ```bash
   cargo test --test cli_install
   cargo test --test cli_format -- --skip test_cli_format_cpp   # cpp は GLIBC 依存のため除外
   cargo test --test cli_daemon
   ```

3. **バージョンバンプ**
   `Cargo.toml` の `version` を更新する（SemVer）:
   - 後方互換な新機能 → minor バンプ（例: `0.2.x` → `0.3.0`）
   - バグ修正のみ → patch バンプ（例: `0.3.0` → `0.3.1`）

   ```bash
   # Cargo.toml の version を編集後:
   cargo build           # Cargo.lock を更新
   git add Cargo.toml Cargo.lock
   git commit -m "chore: bump version to X.Y.Z"
   git push origin main
   ```

4. **タグをプッシュ**
   ```bash
   git tag X.Y.Z
   git push origin X.Y.Z
   ```
   これで Release CI（`.github/workflows/release.yml`）が起動する。

5. **CI を確認**
   ```bash
   gh run list --repo foro-fmt/foro --limit 5
   ```
   全ジョブが green になると GitHub Release が自動作成される。

## Notes

- タグ名は `v` プレフィックスなし（例: `0.3.0`）。`v0.3.0` でも動作するが、
  過去リリースに合わせて `v` なしを使う。
- CHANGELOG は cargo-dist が git log から自動生成する。手書き不要。
- `cargo-dist` の設定は `dist.toml` を参照。
- リリース後に default_config.json のプラグイン URL を更新する必要がある場合は
  `./MAINTAIN_PLUGINS.md` を参照。
