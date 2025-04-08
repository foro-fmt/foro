use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum OnRule {
    Extension(String),
    Or(Vec<OnRule>),
}

impl OnRule {
    pub fn on_match(&self, target_path: &PathBuf) -> bool {
        match self {
            OnRule::Extension(ext) => target_path
                .extension()
                .map_or(false, |e| &format!(".{}", e.to_string_lossy()) == ext),
            OnRule::Or(rules) => rules.iter().any(|rule| rule.on_match(target_path)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PureCommand {
    PluginUrl(#[serde(with = "url_serde")] Url),
    CommandIO { io: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum WriteCommand {
    Pure(PureCommand),
    SimpleCommand(String),
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SomeCommand {
    Pure {
        cmd: CommandWithControlFlow<PureCommand>,
    },
    Write {
        write_cmd: CommandWithControlFlow<WriteCommand>,
    },
}

impl SomeCommand {
    pub fn is_pure(&self) -> bool {
        match self {
            SomeCommand::Pure { .. } => true,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Rule {
    pub on: OnRule,
    #[serde(flatten)]
    pub some_cmd: SomeCommand,
}

impl Rule {
    pub fn on_match(&self, target_path: &PathBuf, force_pure: bool) -> bool {
        if force_pure && !self.some_cmd.is_pure() {
            return false;
        }

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

impl Config {
    pub fn find_matched_rule(&self, target_path: &PathBuf, force_pure: bool) -> Option<Rule> {
        for rule in &self.rules {
            if rule.on_match(target_path, force_pure) {
                return Some(rule.clone());
            }
        }

        None
    }
}

#[allow(unused)]
pub fn load_str(json: &str) -> anyhow::Result<Config> {
    serde_json::from_str(json).map_err(|e| anyhow!(e))
}

pub fn load_file(path: &PathBuf) -> anyhow::Result<Config> {
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
    use std::path::PathBuf;

    #[test]
    fn test_on_rule_extension_match() {
        let on_rule = OnRule::Extension(".rs".to_string());

        let path = PathBuf::from("hello_world.rs");
        assert!(on_rule.on_match(&path), "Should match `.rs` extension");

        let path_ts = PathBuf::from("example.ts");
        assert!(
            !on_rule.on_match(&path_ts),
            "Should not match `.ts` extension"
        );

        let path_no_ext = PathBuf::from("Makefile");
        assert!(
            !on_rule.on_match(&path_no_ext),
            "Should not match no extension path"
        );
    }

    #[test]
    fn test_on_rule_or_logic() {
        let on_rule = OnRule::Or(vec![
            OnRule::Extension(".rs".to_string()),
            OnRule::Extension(".js".to_string()),
        ]);

        let path_rs = PathBuf::from("main.rs");
        assert!(on_rule.on_match(&path_rs), "Should match an `.rs` file");

        let path_js = PathBuf::from("test.js");
        assert!(on_rule.on_match(&path_js), "Should match a `.js` file");

        let path_ts = PathBuf::from("hello.ts");
        assert!(
            !on_rule.on_match(&path_ts),
            "Should not match `.ts` extension"
        );
    }

    #[test]
    fn test_some_command_is_pure() {
        let pure_command = SomeCommand::Pure {
            cmd: CommandWithControlFlow::Command(PureCommand::PluginUrl(
                "https://example.com/plugin.dllpack".parse().unwrap(),
            )),
        };
        assert!(
            pure_command.is_pure(),
            "Expected a pure command to be recognized as pure"
        );

        let write_command = SomeCommand::Write {
            write_cmd: CommandWithControlFlow::Command(WriteCommand::SimpleCommand(
                "echo Hello".to_string(),
            )),
        };
        assert!(
            !write_command.is_pure(),
            "Expected a write command not to be recognized as pure"
        );
    }

    #[test]
    fn test_rule_on_match() {
        let pure_rule = Rule {
            on: OnRule::Extension(".py".to_string()),
            some_cmd: SomeCommand::Pure {
                cmd: CommandWithControlFlow::Command(PureCommand::PluginUrl(
                    "https://example.com/python_formatter.dllpack"
                        .parse()
                        .unwrap(),
                )),
            },
        };

        let path_py = PathBuf::from("script.py");
        assert!(
            pure_rule.on_match(&path_py, false),
            "Should match `.py` extension in non-pure mode"
        );
        assert!(
            pure_rule.on_match(&path_py, true),
            "Should match `.py` extension in pure mode"
        );

        let write_rule = Rule {
            on: OnRule::Extension(".rs".to_string()),
            some_cmd: SomeCommand::Write {
                write_cmd: CommandWithControlFlow::Command(WriteCommand::SimpleCommand(
                    "rustfmt {{ os-target }}".to_string(),
                )),
            },
        };

        let path_rs = PathBuf::from("lib.rs");
        assert!(
            write_rule.on_match(&path_rs, false),
            "Should match `.rs` extension in non-pure mode"
        );
        assert!(
            !write_rule.on_match(&path_rs, true),
            "Should not match if forced pure, but is a write command"
        );
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
                    "write_cmd": "rustfmt {{ os-target }}"
                }
            ],
            "cache_dir": null,
            "socket_dir": null
        }"#;

        let config: Config = serde_json::from_str(json).expect("Should parse valid JSON");

        let path_ts = PathBuf::from("app.ts");
        let matched_ts = config.find_matched_rule(&path_ts, false);
        assert!(
            matched_ts.is_some(),
            "Should find a matching rule for `.ts`"
        );
        assert!(
            matched_ts.unwrap().some_cmd.is_pure(),
            "Expected `.ts` rule to be a pure command"
        );

        let path_rs = PathBuf::from("main.rs");
        let matched_rs = config.find_matched_rule(&path_rs, false);
        assert!(
            matched_rs.is_some(),
            "Should find a matching rule for `.rs`"
        );
        assert!(
            !matched_rs.unwrap().some_cmd.is_pure(),
            "Expected `.rs` rule to be a write command"
        );

        let path_py = PathBuf::from("script.py");
        let matched_py = config.find_matched_rule(&path_py, false);
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
            some_cmd,
            ..
        }) = deserialized.rules.get(0)
        {
            assert_eq!(
                rules.len(),
                2,
                "Expected two OnRule::Extension inside OnRule::Or"
            );
            assert!(some_cmd.is_pure(), "Expected a pure command");
        } else {
            panic!("Expected the first rule to be OnRule::Or with a pure command");
        }
    }

    #[test]
    fn test_some_command_deserialize() {
        let json_str_cmd = r#"
        {
          "on": ".ts",
          "cmd": "https://example.com/plugin.dllpack"
        }
        "#;

        let rule_cmd: Rule = serde_json::from_str(json_str_cmd).unwrap();
        match &rule_cmd.some_cmd {
            SomeCommand::Pure { cmd } => match cmd {
                CommandWithControlFlow::Command(PureCommand::PluginUrl(url)) => {
                    assert_eq!(url.as_str(), "https://example.com/plugin.dllpack");
                }
                _ => panic!("Expected a direct plugin URL in pure command"),
            },
            _ => panic!("Expected a pure command for `.ts`"),
        }

        let json_str_write_cmd = r#"
        {
          "on": ".rs",
          "write_cmd": "rustfmt {{ os-target }}"
        }
        "#;

        let rule_write_cmd: Rule = serde_json::from_str(json_str_write_cmd).unwrap();
        match &rule_write_cmd.some_cmd {
            SomeCommand::Write { write_cmd } => match write_cmd {
                CommandWithControlFlow::Command(WriteCommand::SimpleCommand(s)) => {
                    assert!(s.contains("rustfmt"));
                }
                _ => panic!("Expected a simple command for `.rs`"),
            },
            _ => panic!("Expected a write command for `.rs`"),
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

        let if_command: CommandWithControlFlow<PureCommand> =
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

        let sequential: CommandWithControlFlow<PureCommand> =
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

        let set_command: CommandWithControlFlow<PureCommand> =
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
        let nonexistent_path = PathBuf::from("/path/that/does/not/exist.json");
        let result = load_file(&nonexistent_path);

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
    fn test_pure_command_command_io() {
        let json = r#"{
            "io": "cat {{ input }} | grep pattern"
        }"#;

        let command_io: PureCommand = serde_json::from_str(json).expect("Should parse valid JSON");

        match command_io {
            PureCommand::CommandIO { io } => {
                assert_eq!(io, "cat {{ input }} | grep pattern");
            }
            _ => panic!("Expected CommandIO variant"),
        }
    }

    #[test]
    fn test_write_command_variants() {
        let simple_cmd_json = r#""echo 'Hello World'""#;

        let simple_cmd: WriteCommand =
            serde_json::from_str(simple_cmd_json).expect("Should parse valid JSON");
        match simple_cmd {
            WriteCommand::SimpleCommand(cmd) => {
                assert_eq!(cmd, "echo 'Hello World'");
            }
            _ => panic!("Expected SimpleCommand variant"),
        }

        let pure_cmd_json = r#""https://example.com/plugin.dllpack""#;

        let pure_cmd: WriteCommand =
            serde_json::from_str(pure_cmd_json).expect("Should parse valid JSON");
        match pure_cmd {
            WriteCommand::Pure(_) => {}
            _ => panic!("Expected Pure variant"),
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
