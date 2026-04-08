#![allow(dead_code)]

use assert_cmd::prelude::*;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub enum Cache {
    TempCache(ChildPath),
    GlobalCache,
}

impl Cache {
    pub fn path(&self) -> PathBuf {
        match self {
            Cache::TempCache(path) => path.path().to_path_buf(),
            Cache::GlobalCache => PathBuf::from(env!("TARGET_DIR"))
                .join("foro-test-cache")
                .join(worktree_cache_suffix()),
        }
    }

    pub fn ensure_dir(&self) {
        if !self.path().exists() {
            fs::create_dir_all(self.path()).unwrap();
        }
    }
}

fn worktree_cache_suffix() -> String {
    let mut hasher = DefaultHasher::new();
    env!("CARGO_MANIFEST_DIR").hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

enum CacheKind {
    TempCache(PathBuf),
    GlobalCache,
}

impl CacheKind {
    fn build(self, temp_dir: &assert_fs::TempDir) -> Cache {
        match self {
            CacheKind::TempCache(path) => Cache::TempCache(temp_dir.child(path)),
            CacheKind::GlobalCache => Cache::GlobalCache,
        }
    }
}

pub struct TestEnv {
    temp_dir: assert_fs::TempDir,
    pub work_dir: ChildPath,
    pub config_file: ChildPath,
    pub cache: Cache,
    pub socket_dir: ChildPath,
}

impl TestEnv {
    pub fn new_fixture<P: AsRef<Path>>(fixture_path: P) -> Self {
        TestEnvBuilder::new().fixture_path(fixture_path).build()
    }

    pub fn new() -> Self {
        TestEnvBuilder::new().build()
    }

    fn construct(&self) {
        if !self.config_file.exists() {
            let default_config = self.raw_foro(&["config", "default"]).unwrap().stdout;

            self.config_file.write_binary(&default_config).unwrap();
        }

        self.cache.ensure_dir();

        if !self.socket_dir.exists() {
            self.socket_dir.create_dir_all().unwrap();
        }

        self.foro(&["install"]);
    }

    fn construct_bare(&self) {
        if !self.config_file.exists() {
            let default_config = self.raw_foro(&["config", "default"]).unwrap().stdout;

            self.config_file.write_binary(&default_config).unwrap();
        }

        self.cache.ensure_dir();

        if !self.socket_dir.exists() {
            self.socket_dir.create_dir_all().unwrap();
        }
    }

    pub fn child<P: AsRef<Path>>(&self, path: P) -> ChildPath {
        self.temp_dir.child(path)
    }

    pub fn path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.child(path).path().to_path_buf()
    }

    pub fn assert_eq<P: AsRef<Path>>(&self, actual: P, expected: P) {
        let actual = String::from_utf8(fs::read(self.child(actual)).unwrap())
            .unwrap()
            .replace("\r\n", "\n");
        let expected = String::from_utf8(fs::read(self.child(expected)).unwrap())
            .unwrap()
            .replace("\r\n", "\n");

        assert_eq!(
            actual, expected,
            "\nactual:\n{actual}\n\n-------------\nexpected:\n{expected}"
        );
    }

    pub fn build_option(&self) -> Vec<String> {
        vec![
            "--config-file".to_string(),
            self.config_file.path().to_str().unwrap().to_string(),
            "--cache-dir".to_string(),
            self.cache.path().to_str().unwrap().to_string(),
            "--socket-dir".to_string(),
            self.socket_dir.path().to_str().unwrap().to_string(),
        ]
    }

    pub fn raw_foro(&self, args: &[&str]) -> Command {
        let mut cmd = Command::cargo_bin("foro").unwrap();
        cmd.current_dir(self.work_dir.path());
        cmd.args(args);

        cmd
    }

    pub fn foro_cmd(&self, args: &[&str]) -> Command {
        let mut args: Vec<_> = args.to_vec();
        let options = self.build_option();
        args.extend(options.iter().map(String::as_str));

        self.raw_foro(&args)
    }

    pub fn foro(&self, args: &[&str]) -> Output {
        let mut cmd = self.foro_cmd(args);

        let res = cmd.output().unwrap();
        assert!(res.status.success(), "Command failed: {res:?}");

        res
    }

    pub fn foro_stdout(&self, args: &[&str]) -> String {
        let res = self.foro(args);

        String::from_utf8(res.stdout).unwrap()
    }

    pub fn foro_stderr(&self, args: &[&str]) -> String {
        let res = self.foro(args);

        String::from_utf8(res.stderr).unwrap()
    }
}

pub fn uv_available() -> bool {
    Command::new("uv").arg("--version").output().is_ok()
}

impl Default for TestEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        self.foro(&["daemon", "stop"]);
    }
}

pub struct TestEnvBuilder {
    fixture_path: Option<PathBuf>,
    work_dir: Option<PathBuf>,
    config_file: Option<PathBuf>,
    cache: Option<CacheKind>,
    socket_dir: Option<PathBuf>,
}

impl Default for TestEnvBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TestEnvBuilder {
    pub fn new() -> Self {
        TestEnvBuilder {
            fixture_path: None,
            work_dir: None,
            config_file: None,
            cache: None,
            socket_dir: None,
        }
    }

    pub fn fixture_path<P: AsRef<Path>>(mut self, fixture_path: P) -> Self {
        self.fixture_path = Some(fixture_path.as_ref().to_path_buf());
        self
    }

    pub fn work_dir<P: AsRef<Path>>(mut self, work_dir: P) -> Self {
        self.work_dir = Some(work_dir.as_ref().to_path_buf());
        self
    }

    pub fn config_file<P: AsRef<Path>>(mut self, config_file: P) -> Self {
        self.config_file = Some(config_file.as_ref().to_path_buf());
        self
    }

    pub fn cache_dir<P: AsRef<Path>>(mut self, cache_dir: P) -> Self {
        self.cache = Some(CacheKind::TempCache(cache_dir.as_ref().to_path_buf()));
        self
    }

    pub fn global_cache(mut self) -> Self {
        self.cache = Some(CacheKind::GlobalCache);
        self
    }

    pub fn socket_dir<P: AsRef<Path>>(mut self, socket_dir: P) -> Self {
        self.socket_dir = Some(socket_dir.as_ref().to_path_buf());
        self
    }

    pub fn build(self) -> TestEnv {
        let env = self.build_inner();
        env.construct();
        env
    }

    /// Build without running `foro install`. Used for testing install behavior.
    pub fn build_without_install(self) -> TestEnv {
        let env = self.build_inner();
        env.construct_bare();
        env
    }

    fn build_inner(self) -> TestEnv {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        if let Some(path) = &self.fixture_path {
            temp_dir.copy_from(path, &["**"]).unwrap();
        }

        let work_dir = temp_dir.child(self.work_dir.unwrap_or(PathBuf::from(".")));
        let config_file = temp_dir.child(self.config_file.unwrap_or(PathBuf::from("foro.json")));
        let cache_kind = match self.cache {
            Some(cache) => cache,
            None => {
                if cfg!(windows) {
                    CacheKind::TempCache(PathBuf::from("cache"))
                } else {
                    CacheKind::GlobalCache
                }
            }
        };
        let cache = cache_kind.build(&temp_dir);
        let socket_dir = temp_dir.child(self.socket_dir.unwrap_or(PathBuf::from("socket")));

        TestEnv {
            temp_dir,
            work_dir,
            config_file,
            cache,
            socket_dir,
        }
    }
}
