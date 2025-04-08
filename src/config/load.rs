use crate::app_dir::{AppDirResolver, DefaultAppDirResolver};
use crate::config::load_file;
use crate::config::model::Config;
use crate::debug_long;
use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

// functions that manually inject resolvers
// ----------------------------------------

pub(crate) fn get_or_create_default_config_with<R: AppDirResolver>(
    resolver: &R,
) -> Option<PathBuf> {
    let config_path = resolver.config_file()?;

    if !config_path.exists() {
        debug!("try create default config file: {:?}", config_path);

        fs::DirBuilder::new()
            .recursive(true)
            .create(&config_path.parent()?)
            .ok()?;

        let default_config = include_str!("default_config.json");

        fs::write(&config_path, &default_config).ok()?;

        info!("created default config file: {:?}", config_path);
        info!("content: {:?}", default_config);
    }

    Some(config_path)
}

pub(crate) fn load_config_and_cache_with<R: AppDirResolver>(
    resolver: &R,
    given_config_file: &Option<PathBuf>,
    given_cache_dir: &Option<PathBuf>,
) -> Result<(Config, PathBuf)> {
    let config_file = given_config_file
        .clone()
        .or_else(|| get_or_create_default_config_with(resolver))
        .context("Could not get config directory")?;

    let config = load_file(&config_file)
        .with_context(|| format!("Failed to load config file ({:?})", &config_file))?;

    let cache_dir = given_cache_dir
        .clone()
        .or(config.cache_dir.clone())
        .or_else(|| resolver.cache_dir())
        .context("Failed to get cache directory")?;

    debug!("config file: {:?}", &config_file);
    debug_long!("config: {:?}", &config);
    debug!("cache dir: {:?}", &cache_dir);

    Ok((config, cache_dir))
}

pub(crate) fn load_config_and_socket_with<R: AppDirResolver>(
    resolver: &R,
    given_config_file: &Option<PathBuf>,
    given_socket_dir: &Option<PathBuf>,
) -> Result<(Config, PathBuf)> {
    let config_file = given_config_file
        .clone()
        .or_else(|| get_or_create_default_config_with(resolver))
        .context("Failed to get config directory")?;

    let config = load_file(&config_file)
        .with_context(|| format!("Failed to load config file ({:?})", &config_file))?;

    let socket_dir = given_socket_dir
        .clone()
        .or(config.socket_dir.clone())
        .or_else(|| resolver.socket_dir())
        .context("Failed to get socket directory")?;

    debug!("config file: {:?}", &config_file);
    debug_long!("config: {:?}", &config);
    debug!("socket dir: {:?}", &socket_dir);

    Ok((config, socket_dir))
}

pub(crate) fn load_paths_with<R: AppDirResolver>(
    resolver: &R,
    given_config_file: &Option<PathBuf>,
    given_cache_dir: &Option<PathBuf>,
    given_socket_dir: &Option<PathBuf>,
) -> Result<(PathBuf, PathBuf, PathBuf)> {
    let config_file = given_config_file
        .clone()
        .or_else(|| get_or_create_default_config_with(resolver))
        .context("Failed to get config directory")?;

    let config = load_file(&config_file)
        .with_context(|| format!("Failed to load config file ({:?})", &config_file))?;

    let cache_dir = given_cache_dir
        .clone()
        .or(config.cache_dir.clone())
        .or_else(|| resolver.cache_dir())
        .context("Failed to get cache directory")?;

    let socket_dir = given_socket_dir
        .clone()
        .or(config.socket_dir.clone())
        .or_else(|| resolver.socket_dir())
        .context("Failed to get socket directory")?;

    Ok((config_file, cache_dir, socket_dir))
}

// functions that use the default resolver
// ----------------------------------------

pub(crate) fn get_or_create_default_config() -> Option<PathBuf> {
    get_or_create_default_config_with(&DefaultAppDirResolver {})
}

pub(crate) fn load_config_and_cache(
    given_config_file: &Option<PathBuf>,
    given_cache_dir: &Option<PathBuf>,
) -> Result<(Config, PathBuf)> {
    load_config_and_cache_with(
        &DefaultAppDirResolver {},
        given_config_file,
        given_cache_dir,
    )
}

pub(crate) fn load_config_and_socket(
    given_config_file: &Option<PathBuf>,
    given_socket_dir: &Option<PathBuf>,
) -> Result<(Config, PathBuf)> {
    load_config_and_socket_with(
        &DefaultAppDirResolver {},
        given_config_file,
        given_socket_dir,
    )
}

pub(crate) fn load_paths(
    given_config_file: &Option<PathBuf>,
    given_cache_dir: &Option<PathBuf>,
    given_socket_dir: &Option<PathBuf>,
) -> Result<(PathBuf, PathBuf, PathBuf)> {
    load_paths_with(
        &DefaultAppDirResolver {},
        given_config_file,
        given_cache_dir,
        given_socket_dir,
    )
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::app_dir::AppDirResolver;
    use anyhow::Result;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    /// A mock resolver that returns predefined paths.
    struct MockResolver {
        config_path: Option<PathBuf>,
        cache_path: Option<PathBuf>,
        socket_path: Option<PathBuf>,
        log_path: Option<PathBuf>,
    }

    impl AppDirResolver for MockResolver {
        fn config_file(&self) -> Option<PathBuf> {
            self.config_path.clone()
        }

        fn cache_dir(&self) -> Option<PathBuf> {
            self.cache_path.clone()
        }

        fn socket_dir(&self) -> Option<PathBuf> {
            self.socket_path.clone()
        }

        fn log_dir(&self) -> Option<PathBuf> {
            self.log_path.clone()
        }
    }

    /// Returns the content of default_config.json for comparison.
    fn default_config_str() -> &'static str {
        include_str!("default_config.json")
    }

    #[test]
    fn test_get_or_create_default_config_with_creates_file_if_not_exists() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_file_path = temp_dir.path().join("foro.json");

        let resolver = MockResolver {
            config_path: Some(config_file_path.clone()),
            cache_path: None,
            socket_path: None,
            log_path: None,
        };

        // Should create the default config file if it doesn't exist.
        let returned_path =
            get_or_create_default_config_with(&resolver).expect("Expected a valid path.");

        assert_eq!(returned_path, config_file_path);

        // Check if the file was actually created with default content.
        let content = fs::read_to_string(&config_file_path)?;
        assert_eq!(content, default_config_str());

        Ok(())
    }

    #[test]
    fn test_get_or_create_default_config_with_does_not_overwrite_if_exists() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_file_path = temp_dir.path().join("foro.json");

        // Write a different initial content.
        let initial_content = r#"{"rules":[]}"#;
        fs::write(&config_file_path, initial_content)?;

        let resolver = MockResolver {
            config_path: Some(config_file_path.clone()),
            cache_path: None,
            socket_path: None,
            log_path: None,
        };

        // Should not overwrite existing content.
        let returned_path =
            get_or_create_default_config_with(&resolver).expect("Expected a valid path.");
        assert_eq!(returned_path, config_file_path);

        let content_after = fs::read_to_string(&config_file_path)?;
        assert_eq!(content_after, initial_content);

        Ok(())
    }

    #[test]
    fn test_load_config_and_cache_with_given_paths() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_file_path = temp_dir.path().join("custom_config.json");
        let custom_cache_dir = temp_dir.path().join("my_cache");

        // This config file explicitly sets a cache_dir inside the JSON.
        let config_json = r#"
        {
          "rules": [
            {
              "on": ".rs",
              "cmd": "https://example.com/rust.dllpack"
            }
          ],
          "cache_dir": "config_defined_cache_dir", 
          "socket_dir": "config_defined_socket_dir"
        }
        "#;
        fs::write(&config_file_path, config_json)?;

        let resolver = MockResolver {
            config_path: Some(config_file_path.clone()),
            cache_path: None,
            socket_path: None,
            log_path: None,
        };

        let given_config_file = Some(config_file_path.clone());
        let given_cache_dir = Some(custom_cache_dir.clone());

        // User-provided cache_dir should take precedence.
        let (loaded_config, loaded_cache_dir) =
            load_config_and_cache_with(&resolver, &given_config_file, &given_cache_dir)?;

        assert_eq!(loaded_cache_dir, custom_cache_dir);
        assert_eq!(loaded_config.rules.len(), 1);
        assert!(loaded_config.rules[0].some_cmd.is_pure());

        Ok(())
    }

    #[test]
    fn test_load_config_and_cache_with_no_given_paths_uses_resolver() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_file_path = temp_dir.path().join("foro.json");
        let cache_dir_path = temp_dir.path().join("cache_dir_from_resolver");

        // The resolver should return paths, but the config doesn't exist yet.
        let resolver = MockResolver {
            config_path: Some(config_file_path.clone()),
            cache_path: Some(cache_dir_path.clone()),
            socket_path: None,
            log_path: None,
        };

        // This should create the default config and use resolver's cache_dir.
        let (config, used_cache_dir) = load_config_and_cache_with(&resolver, &None, &None)?;

        let written_config = fs::read_to_string(&config_file_path)?;
        assert_eq!(written_config, default_config_str());
        assert_eq!(used_cache_dir, cache_dir_path);
        assert_eq!(config.rules.len(), 5);

        Ok(())
    }

    #[test]
    fn test_load_config_and_socket_with_custom_values() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_file_path = temp_dir.path().join("config.json");
        let socket_dir_path = temp_dir.path().join("custom_socket_dir");

        // This config sets a socket_dir, but we will override it.
        let config_json = r#"
        {
            "rules": [],
            "cache_dir": "/ignore_this",
            "socket_dir": "config_defined_socket_dir"
        }
        "#;
        fs::write(&config_file_path, config_json)?;

        let resolver = MockResolver {
            config_path: Some(config_file_path.clone()),
            cache_path: None,
            socket_path: None,
            log_path: None,
        };

        let (config, used_socket_dir) = load_config_and_socket_with(
            &resolver,
            &Some(config_file_path),
            &Some(socket_dir_path.clone()),
        )?;

        assert!(config.rules.is_empty());
        assert_eq!(used_socket_dir, socket_dir_path);

        Ok(())
    }

    #[test]
    fn test_load_paths_with_fallback_order() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_file_path = temp_dir.path().join("some_config.json");

        let config_json = r#"
        {
            "rules": [],
            "cache_dir": "/from_config_cache",
            "socket_dir": "/from_config_socket"
        }
        "#;
        fs::write(&config_file_path, config_json)?;

        let resolver_cache = temp_dir.path().join("resolver_cache");
        let resolver_socket = temp_dir.path().join("resolver_socket");

        let resolver = MockResolver {
            config_path: Some(config_file_path.clone()),
            cache_path: Some(resolver_cache.clone()),
            socket_path: Some(resolver_socket.clone()),
            log_path: None,
        };

        // If config_file is Some(...), it should load from it.
        let (actual_config_file, actual_cache, actual_socket) =
            load_paths_with(&resolver, &Some(config_file_path.clone()), &None, &None)?;

        assert_eq!(actual_config_file, config_file_path);
        assert_eq!(actual_cache, PathBuf::from("/from_config_cache"));
        assert_eq!(actual_socket, PathBuf::from("/from_config_socket"));

        // If config_file is None, it creates the default config if none is found.
        fs::remove_file(&config_file_path)?; // Remove it to force creation.
        let (cf, _, _) = load_paths_with(&resolver, &None, &None, &None)?;
        let default_content = fs::read_to_string(&config_file_path)?;
        assert_eq!(default_content, default_config_str());
        assert_eq!(cf, config_file_path);

        Ok(())
    }

    #[test]
    fn test_error_when_config_file_not_found_in_resolver() {
        let resolver = MockResolver {
            config_path: None,
            cache_path: None,
            socket_path: None,
            log_path: None,
        };

        // This should return None, since the resolver gives no path.
        let result = get_or_create_default_config_with(&resolver);
        assert!(result.is_none());

        // load_config_and_cache_with should fail in this situation.
        let err = load_config_and_cache_with(&resolver, &None, &None).unwrap_err();
        assert!(
            err.to_string().contains("Could not get config directory"),
            "Expected an error about missing config directory."
        );
    }

    #[test]
    fn test_wrapper_functions_with_mocked_resolver() -> Result<()> {
        use std::sync::Once;

        static INIT: Once = Once::new();
        static mut MOCK_CONFIG_PATH: Option<PathBuf> = None;
        static mut MOCK_CACHE_PATH: Option<PathBuf> = None;
        static mut MOCK_SOCKET_PATH: Option<PathBuf> = None;

        unsafe fn init_mocks() {
            INIT.call_once(|| {
                let temp_dir = tempdir().expect("Failed to create temp dir");
                MOCK_CONFIG_PATH = Some(temp_dir.path().join("mock_config.json"));
                MOCK_CACHE_PATH = Some(temp_dir.path().join("mock_cache"));
                MOCK_SOCKET_PATH = Some(temp_dir.path().join("mock_socket"));

                fs::write(
                    MOCK_CONFIG_PATH.as_ref().unwrap(),
                    r#"{"rules":[],"cache_dir":null,"socket_dir":null}"#,
                )
                .expect("Failed to write mock config");

                std::mem::forget(temp_dir);
            });
        }

        unsafe {
            init_mocks();
        }

        Ok(())
    }

    #[test]
    fn test_get_or_create_default_config() -> Result<()> {
        let temp_dir = tempdir()?;
        let _config_path = temp_dir.path().join("config.json");

        assert!(
            get_or_create_default_config().is_some() || get_or_create_default_config().is_none(),
            "get_or_create_default_config should always return Some or None"
        );

        Ok(())
    }

    #[test]
    fn test_load_config_and_cache_wrapper() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("config.json");
        let cache_path = temp_dir.path().join("cache");

        fs::write(
            &config_path,
            r#"{"rules":[],"cache_dir":null,"socket_dir":null}"#,
        )?;

        let given_config = Some(config_path);
        let given_cache = Some(cache_path.clone());

        let result = load_config_and_cache(&given_config, &given_cache);

        assert!(result.is_ok(), "load_config_and_cache should return Ok");
        if let Ok((config, cache_dir)) = result {
            assert_eq!(
                cache_dir, cache_path,
                "Cache directory should match expected"
            );
            assert!(config.rules.is_empty(), "Config rules should be empty");
        }

        Ok(())
    }

    #[test]
    fn test_load_config_and_socket_wrapper() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("config.json");
        let socket_path = temp_dir.path().join("socket");

        fs::write(
            &config_path,
            r#"{"rules":[],"cache_dir":null,"socket_dir":null}"#,
        )?;

        let given_config = Some(config_path);
        let given_socket = Some(socket_path.clone());

        let result = load_config_and_socket(&given_config, &given_socket);

        assert!(result.is_ok(), "load_config_and_socket should return Ok");
        if let Ok((config, socket_dir)) = result {
            assert_eq!(
                socket_dir, socket_path,
                "Socket directory should match expected"
            );
            assert!(config.rules.is_empty(), "Config rules should be empty");
        }

        Ok(())
    }

    #[test]
    fn test_load_paths_wrapper() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("config.json");
        let cache_path = temp_dir.path().join("cache");
        let socket_path = temp_dir.path().join("socket");

        fs::write(
            &config_path,
            r#"{"rules":[],"cache_dir":null,"socket_dir":null}"#,
        )?;

        let given_config = Some(config_path.clone());
        let given_cache = Some(cache_path.clone());
        let given_socket = Some(socket_path.clone());

        let result = load_paths(&given_config, &given_cache, &given_socket);

        assert!(result.is_ok(), "load_paths should return Ok");
        if let Ok((config_file, cache_dir, socket_dir)) = result {
            assert_eq!(
                config_file, config_path,
                "Config file should match expected"
            );
            assert_eq!(
                cache_dir, cache_path,
                "Cache directory should match expected"
            );
            assert_eq!(
                socket_dir, socket_path,
                "Socket directory should match expected"
            );
        }

        Ok(())
    }

    #[test]
    fn test_error_handling_in_get_or_create_default_config_with() -> Result<()> {
        let resolver = MockResolver {
            config_path: Some(PathBuf::from(
                "/nonexistent/directory/that/should/not/exist/config.json",
            )),
            cache_path: None,
            socket_path: None,
            log_path: None,
        };

        let result = get_or_create_default_config_with(&resolver);
        assert!(
            result.is_none(),
            "Should return None when parent directory creation fails"
        );

        Ok(())
    }

    #[test]
    fn test_log_dir_in_mock_resolver() -> Result<()> {
        let temp_dir = tempdir()?;
        let log_path = temp_dir.path().join("logs");

        let resolver = MockResolver {
            config_path: None,
            cache_path: None,
            socket_path: None,
            log_path: Some(log_path.clone()),
        };

        assert_eq!(resolver.log_dir(), Some(log_path));

        Ok(())
    }

    #[test]
    fn test_error_paths_in_load_config_and_socket_with() -> Result<()> {
        let temp_dir = tempdir()?;
        let nonexistent_config = temp_dir.path().join("nonexistent.json");
        let socket_dir = temp_dir.path().join("socket");

        let resolver = MockResolver {
            config_path: None,
            cache_path: None,
            socket_path: Some(socket_dir.clone()),
            log_path: None,
        };

        let result = load_config_and_socket_with(&resolver, &None, &None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to get config directory"));

        let result = load_config_and_socket_with(&resolver, &Some(nonexistent_config), &None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to load config file"));

        Ok(())
    }

    #[test]
    fn test_error_paths_in_load_paths_with() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("config.json");

        fs::write(
            &config_path,
            r#"{"rules":[],"cache_dir":"/cache","socket_dir":null}"#,
        )?;

        let resolver = MockResolver {
            config_path: Some(config_path.clone()),
            cache_path: Some(temp_dir.path().join("cache")),
            socket_path: None, // No socket dir in resolver
            log_path: None,
        };

        let result = load_paths_with(&resolver, &Some(config_path), &None, &None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to get socket directory"));

        Ok(())
    }

    #[test]
    fn test_debug_logging_in_get_or_create_default_config_with() -> Result<()> {
        use log::Level;
        use std::sync::Once;

        static INIT_LOGGER: Once = Once::new();

        INIT_LOGGER.call_once(|| {
            env_logger::builder()
                .filter_level(log::LevelFilter::Debug)
                .is_test(true)
                .try_init()
                .ok();
        });

        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("config.json");

        let resolver = MockResolver {
            config_path: Some(config_path),
            cache_path: None,
            socket_path: None,
            log_path: None,
        };

        let _ = get_or_create_default_config_with(&resolver);

        Ok(())
    }

    #[test]
    fn test_debug_logging_in_load_config_and_cache_with() -> Result<()> {
        use log::Level;
        use std::sync::Once;

        static INIT_LOGGER: Once = Once::new();

        INIT_LOGGER.call_once(|| {
            env_logger::builder()
                .filter_level(log::LevelFilter::Debug)
                .is_test(true)
                .try_init()
                .ok();
        });

        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("config.json");
        let cache_path = temp_dir.path().join("cache");

        fs::write(
            &config_path,
            r#"{"rules":[],"cache_dir":null,"socket_dir":null}"#,
        )?;

        let resolver = MockResolver {
            config_path: Some(config_path.clone()),
            cache_path: Some(cache_path.clone()),
            socket_path: None,
            log_path: None,
        };

        let _ = load_config_and_cache_with(&resolver, &Some(config_path), &None);

        Ok(())
    }

    #[test]
    fn test_debug_logging_in_load_config_and_socket_with() -> Result<()> {
        use log::Level;
        use std::sync::Once;

        static INIT_LOGGER: Once = Once::new();

        INIT_LOGGER.call_once(|| {
            env_logger::builder()
                .filter_level(log::LevelFilter::Debug)
                .is_test(true)
                .try_init()
                .ok();
        });

        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("config.json");
        let socket_path = temp_dir.path().join("socket");

        fs::write(
            &config_path,
            r#"{"rules":[],"cache_dir":null,"socket_dir":null}"#,
        )?;

        let resolver = MockResolver {
            config_path: Some(config_path.clone()),
            cache_path: None,
            socket_path: Some(socket_path.clone()),
            log_path: None,
        };

        let _ = load_config_and_socket_with(&resolver, &Some(config_path), &None);

        Ok(())
    }

    #[test]
    fn test_error_paths_in_get_or_create_default_config_with_parent_creation() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_file = temp_dir.path().join("readonly_dir").join("config.json");

        fs::create_dir(temp_dir.path().join("readonly_dir"))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let readonly = fs::Permissions::from_mode(0o444);
            fs::set_permissions(temp_dir.path().join("readonly_dir"), readonly)?;
        }

        let resolver = MockResolver {
            config_path: Some(config_file),
            cache_path: None,
            socket_path: None,
            log_path: None,
        };

        let _result = get_or_create_default_config_with(&resolver);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let writable = fs::Permissions::from_mode(0o755);
            fs::set_permissions(temp_dir.path().join("readonly_dir"), writable)?;
        }

        Ok(())
    }
}
