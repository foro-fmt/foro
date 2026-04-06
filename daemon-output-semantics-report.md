# Daemon Output Semantics Report

## 現状整理

- `foro format` は現在、常に daemon 経由で実行される。単一ファイルは `DaemonCommands::Format`、ディレクトリや複数パスは `DaemonCommands::BulkFormat` に分岐する。
- そのため `bulk_format.rs` の `info!` は CLI 直下の出力ではなく、daemon 内部の実行ログである。
- `eprintln!` を daemon 側で使うのは責務境界として不自然であり、daemon は `log` と構造化レスポンスだけを持つ方が自然である。

## 推奨する責務分離

- `stdout`: パイプ可能な成果物だけを出す。
- `stderr` の素の文言 (`eprintln!`): CLI がユーザーに返す正式な実行結果だけを出す。
- `info!/warn!/debug!`: daemon lifecycle、startup lock、config 自動生成、build ID mismatch、接続状態などの補助情報と診断を出す。

## この方針での整理

- `foro daemon start` の成功通知は `eprintln!` ではなく `info!` でよい。
- `foro format` が内部で daemon を自動起動した場合の通知も `info!` でよい。
- startup lock の待機は、単なる内部診断ではなく前景コマンドの待機理由なので、`debug!` より `info!` が自然である。

## さらなる改善案

- daemon server 層では `println!` / `eprintln!` を禁止し、構造化レスポンスと `log` のみに揃える。
- CLI 層だけがユーザー向け文言を生成する。
- `start_daemon()` や `ensure_daemon_running()` は文字列を直接出す代わりに、起きたことを型で返す。

```rust
enum DaemonStartupOutcome {
    Started,
    AlreadyRunning,
    Restarted,
    WaitedForLock,
}
```

- CLI はその結果を見て、明示的な `foro daemon start` と暗黙の auto-start で文言を分ける。
- 例:
  - explicit start: `Daemon started`
  - implicit start: `Started daemon automatically`

## 最小差分でやるなら

- まずは daemon 側の human-facing text を増やさない。
- 次に `ensure_daemon_running()` の戻り値を `Result<()>` から「何が起きたか分かる型」に変える。
- その型を `src/cli/format.rs` と `src/cli/daemon.rs` で受けて、必要な `info!` を呼び分ける。

## テスト観点

- `foro format file.rs` の初回実行で auto-start が起きたとき、正式結果と補助ログがどう並ぶかを確認するテストを追加する。
- `foro format .` のとき、bulk path の daemon ログと CLI の最終結果が混ざっても意味論が崩れないことを確認する。
- `startup-lock held, waiting...` がデフォルトログで見えることを確認する。
