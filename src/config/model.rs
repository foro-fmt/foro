use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum OnRule {
    Extension(String),
    Or(Vec<OnRule>),
}

impl OnRule {
    pub fn on_match(&self, target_path: &Path) -> bool {
        match self {
            OnRule::Extension(ext) => target_path
                .extension()
                .is_some_and(|e| &format!(".{}", e.to_string_lossy()) == ext),
            OnRule::Or(rules) => rules.iter().any(|rule| rule.on_match(target_path)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Command {
    PluginUrl(#[serde(with = "url_serde")] Url),
    CommandIO { io: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum CommandWithControlFlow<T> {
    If {
        run: Box<CommandWithControlFlow<T>>,
        cond: String,
        on_true: Box<CommandWithControlFlow<T>>,
        on_false: Box<CommandWithControlFlow<T>>,
    },
    Sequential(Vec<CommandWithControlFlow<T>>),
    Set {
        set: HashMap<String, String>,
    },
    Command(T),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Rule {
    pub on: OnRule,
    pub cmd: CommandWithControlFlow<Command>,
}

impl Rule {
    pub fn on_match(&self, target_path: &Path) -> bool {
        self.on.on_match(target_path)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub rules: Vec<Rule>,
    #[serde(default = "none")]
    pub cache_dir: Option<PathBuf>,
    #[serde(default = "none")]
    pub socket_dir: Option<PathBuf>,
}

fn none<T>() -> Option<T> {
    None
}

fn collect_urls(cmd: &CommandWithControlFlow<Command>, urls: &mut Vec<Url>) {
    match cmd {
        CommandWithControlFlow::Command(Command::PluginUrl(url)) => urls.push(url.clone()),
        CommandWithControlFlow::Command(Command::CommandIO { .. }) => {}
        CommandWithControlFlow::Sequential(cmds) => {
            for c in cmds {
                collect_urls(c, urls);
            }
        }
        CommandWithControlFlow::If {
            run,
            on_true,
            on_false,
            ..
        } => {
            collect_urls(run, urls);
            collect_urls(on_true, urls);
            collect_urls(on_false, urls);
        }
        CommandWithControlFlow::Set { .. } => {}
    }
}

impl Config {
    pub fn find_matched_rule(&self, target_path: &Path) -> Option<Rule> {
        for rule in &self.rules {
            if rule.on_match(target_path) {
                return Some(rule.clone());
            }
        }

        None
    }

    pub fn all_plugin_urls(&self) -> Vec<Url> {
        let mut urls = Vec::new();
        for rule in &self.rules {
            collect_urls(&rule.cmd, &mut urls);
        }
        urls
    }
}

#[allow(unused)]
pub fn load_str(json: &str) -> anyhow::Result<Config> {
    serde_json::from_str(json).map_err(|e| anyhow!(e))
}

pub fn load_file(path: &Path) -> anyhow::Result<Config> {
    // memo: in my measurement, this implementation is faster than serde_json::from_reader, etc
    let mut file = fs::File::open(path).context("Failed to open file")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    serde_json::from_slice(&buffer).map_err(|e| anyhow!(e))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_on_rule_extension_match() {
        let on_rule = OnRule::Extension(".rs".to_string());

        let path = Path::new("hello_world.rs");
        assert!(on_rule.on_match(path), "Should match `.rs` extension");

        let path_ts = Path::new("example.ts");
        assert!(
            !on_rule.on_match(path_ts),
            "Should not match `.ts` extension"
        );

        let path_no_ext = Path::new("Makefile");
        assert!(
            !on_rule.on_match(path_no_ext),
            "Should not match no extension path"
        );
    }

    #[test]
    fn test_on_rule_or_logic() {
        let on_rule = OnRule::Or(vec![
            OnRule::Extension(".rs".to_string()),
            OnRule::Extension(".js".to_string()),
        ]);

        let path_rs = Path::new("main.rs");
        assert!(on_rule.on_match(path_rs), "Should match an `.rs` file");

        let path_js = Path::new("test.js");
        assert!(on_rule.on_match(path_js), "Should match a `.js` file");

        let path_ts = Path::new("hello.ts");
        assert!(
            !on_rule.on_match(path_ts),
            "Should not match `.ts` extension"
        );
    }

    #[test]
    fn test_rule_on_match() {
        let rule = Rule {
            on: OnRule::Extension(".py".to_string()),
            cmd: CommandWithControlFlow::Command(Command::PluginUrl(
                "https://example.com/python_formatter.dllpack"
                    .parse()
                    .unwrap(),
            )),
        };

        let path_py = Path::new("script.py");
        assert!(rule.on_match(path_py), "Should match `.py` extension");

        let path_rs = Path::new("lib.rs");
        assert!(!rule.on_match(path_rs), "Should not match `.rs` extension");
    }

    #[test]
    fn test_config_find_matched_rule() {
        let json = r#"{
            "rules": [
                {
                    "on": ".ts",
                    "cmd": "https://example.com/typescript.dllpack"
                },
                {
                    "on": ".rs",
                    "cmd": "https://example.com/rustfmt.dllpack"
                }
            ],
            "cache_dir": null,
            "socket_dir": null
        }"#;

        let config: Config = serde_json::from_str(json).expect("Should parse valid JSON");

        let path_ts = Path::new("app.ts");
        let matched_ts = config.find_matched_rule(path_ts);
        assert!(
            matched_ts.is_some(),
            "Should find a matching rule for `.ts`"
        );

        let path_rs = Path::new("main.rs");
        let matched_rs = config.find_matched_rule(path_rs);
        assert!(
            matched_rs.is_some(),
            "Should find a matching rule for `.rs`"
        );

        let path_py = Path::new("script.py");
        let matched_py = config.find_matched_rule(path_py);
        assert!(matched_py.is_none(), "No rule should match `.py`");
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let json = r#"{
            "rules": [
                {
                    "on": [".json", ".yaml"],
                    "cmd": "https://example.com/json_plugin.dllpack"
                }
            ],
            "cache_dir": "/custom/cache/foro",
            "socket_dir": null
        }"#;

        let original_config: Config = serde_json::from_str(json).expect("Should parse valid JSON");
        let serialized = serde_json::to_string(&original_config).expect("Should serialize config");
        let deserialized: Config =
            serde_json::from_str(&serialized).expect("Should deserialize config");

        assert_eq!(original_config.rules.len(), deserialized.rules.len());
        assert_eq!(original_config.cache_dir, deserialized.cache_dir);
        assert_eq!(original_config.socket_dir, deserialized.socket_dir);

        if let Some(Rule {
            on: OnRule::Or(rules),
            ..
        }) = deserialized.rules.first()
        {
            assert_eq!(
                rules.len(),
                2,
                "Expected two OnRule::Extension inside OnRule::Or"
            );
        } else {
            panic!("Expected the first rule to be OnRule::Or");
        }
    }

    #[test]
    fn test_command_deserialize() {
        let json_str_cmd = r#"
        {
          "on": ".ts",
          "cmd": "https://example.com/plugin.dllpack"
        }
        "#;

        let rule: Rule = serde_json::from_str(json_str_cmd).unwrap();
        match &rule.cmd {
            CommandWithControlFlow::Command(Command::PluginUrl(url)) => {
                assert_eq!(url.as_str(), "https://example.com/plugin.dllpack");
            }
            _ => panic!("Expected a direct plugin URL"),
        }
    }

    #[test]
    fn test_command_with_control_flow_if() {
        let json = r#"{
            "run": "https://example.com/plugin.dllpack",
            "cond": "test_condition",
            "on_true": "https://example.com/true.dllpack",
            "on_false": "https://example.com/false.dllpack"
        }"#;

        let if_command: CommandWithControlFlow<Command> =
            serde_json::from_str(json).expect("Should parse valid JSON");

        match if_command {
            CommandWithControlFlow::If {
                run,
                cond,
                on_true,
                on_false,
            } => {
                assert_eq!(cond, "test_condition");
                match *run {
                    CommandWithControlFlow::Command(_) => {}
                    _ => panic!("Expected a Command variant"),
                }
                match *on_true {
                    CommandWithControlFlow::Command(_) => {}
                    _ => panic!("Expected a Command variant"),
                }
                match *on_false {
                    CommandWithControlFlow::Command(_) => {}
                    _ => panic!("Expected a Command variant"),
                }
            }
            _ => panic!("Expected an If variant"),
        }
    }

    #[test]
    fn test_command_with_control_flow_sequential() {
        let json = r#"[
            "https://example.com/plugin1.dllpack",
            "https://example.com/plugin2.dllpack"
        ]"#;

        let sequential: CommandWithControlFlow<Command> =
            serde_json::from_str(json).expect("Should parse valid JSON");

        match sequential {
            CommandWithControlFlow::Sequential(commands) => {
                assert_eq!(commands.len(), 2);
            }
            _ => panic!("Expected a Sequential variant"),
        }
    }

    #[test]
    fn test_command_with_control_flow_set() {
        let json = r#"{
            "set": {
                "key1": "value1",
                "key2": "value2"
            }
        }"#;

        let set_command: CommandWithControlFlow<Command> =
            serde_json::from_str(json).expect("Should parse valid JSON");

        match set_command {
            CommandWithControlFlow::Set { set } => {
                assert_eq!(set.len(), 2);
                assert_eq!(set.get("key1"), Some(&"value1".to_string()));
                assert_eq!(set.get("key2"), Some(&"value2".to_string()));
            }
            _ => panic!("Expected a Set variant"),
        }
    }

    #[test]
    fn test_load_str() {
        let json = r#"{"rules":[],"cache_dir":"/cache","socket_dir":"/socket"}"#;
        let config = load_str(json).expect("Should parse valid JSON");

        assert!(config.rules.is_empty());
        assert_eq!(config.cache_dir, Some(PathBuf::from("/cache")));
        assert_eq!(config.socket_dir, Some(PathBuf::from("/socket")));
    }

    #[test]
    fn test_load_str_invalid_json() {
        let invalid_json = r#"{"rules":[],"cache_dir":"/cache","socket_dir":}"#; // Syntax error
        let result = load_str(invalid_json);

        assert!(result.is_err(), "Should fail with invalid JSON");
    }

    #[test]
    fn test_load_file_nonexistent() {
        let nonexistent_path = Path::new("/path/that/does/not/exist.json");
        let result = load_file(nonexistent_path);

        assert!(result.is_err(), "Should fail with nonexistent file");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to open file"),
            "Error message should mention file opening failure"
        );
    }

    #[test]
    fn test_none_helper_function() {
        let result: Option<String> = none();
        assert!(result.is_none(), "none() should return None");

        let result: Option<i32> = none();
        assert!(result.is_none(), "none() should return None for any type");
    }

    #[test]
    fn test_command_io() {
        let json = r#"{
            "io": "cat {{ input }} | grep pattern"
        }"#;

        let command_io: Command = serde_json::from_str(json).expect("Should parse valid JSON");

        match command_io {
            Command::CommandIO { io } => {
                assert_eq!(io, "cat {{ input }} | grep pattern");
            }
            _ => panic!("Expected CommandIO variant"),
        }
    }

    #[test]
    fn test_load_file_invalid_content() {
        use std::io::Write;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("invalid.json");

        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"{invalid json}").unwrap();

        let result = load_file(&file_path);
        assert!(result.is_err(), "Should fail with invalid JSON content");
    }
}
