use crate::config::{CommandWithControlFlow, PureCommand, SomeCommand, WriteCommand};
use crate::{debug_long, trace_long};
use anyhow::{anyhow, Context, Result};
use dll_pack::load::{NativeLibrary, WasmLibrary};
use dll_pack::{load, run_multi_cached, Library};
use foro_plugin_utils::data_json_utils::{merge, JsonGetter};
use log::{debug, trace};
use minijinja;
use serde_json::{json, to_value, Value};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use url::Url;
use wasmtime::{Instance, Store};
use wasmtime_wasi::preview1::WasiP1Ctx;

struct PluginSetting {
    pub source: Url,
    pub cache: bool,
}

fn run_plugin_inner_wasm(
    instance: Instance,
    mut store: &mut Store<WasiP1Ctx>,
    cur_map: Value,
) -> Result<Value> {
    let memory = instance
        .get_memory(&mut store, "memory")
        .context("Failed to get memory")?;

    trace!("loaded memory");

    // to handle memory within wasm, we will use `foro_malloc` and `foro_free` as malloc/free,
    // which are exported from the wasm binary
    let malloc = instance.get_typed_func::<(u64, u64), u64>(&mut store, "foro_malloc")?;
    let free = instance.get_typed_func::<(u64, u64, u64), ()>(&mut store, "foro_free")?;

    let func = instance.get_typed_func::<(u64, u64), u64>(&mut store, "foro_main")?;

    trace!("loaded functions");

    // the input json is written to memory on wasm, and the pointer is obtained
    let input_data: Vec<u8> = serde_json::to_vec(&cur_map)?;
    let input_len = input_data.len() as u64;

    let data_ptr = malloc.call(&mut store, (input_data.len() as u64, 0))?;

    memory.write(&mut store, data_ptr as usize, &input_data)?;

    trace!("loaded input data");

    trace!("real start run");

    // The pointer to the input json and its length are given to `foro_main`.
    // The plugin has the length information, so it can easily read the contents of the pointer.
    //
    // However, due to the restrictions of the wasm abi,`foro_main` only returns a single integer value.
    //
    // Therefore, it creates the output in the format “the length of the output json is placed
    // in the first 8 bytes (in little endian) and the output json is placed after that”,
    // and returns the pointer to it.

    let result_ptr = func.call(&mut store, (data_ptr, input_len))?;

    trace!("real end run");

    let wasm_memory = memory.data(&store);
    let len_part = &wasm_memory[result_ptr as usize..(result_ptr as usize + 8)];
    let len = u64::from_le_bytes(len_part.try_into()?) as usize;

    let output_data = &wasm_memory[result_ptr as usize + 8..(result_ptr as usize + 8 + len)];

    let output_value: Value = serde_json::from_slice(output_data)?;

    trace!("loaded output data");

    free.call(&mut store, (data_ptr, input_data.len() as u64, 0))?;
    free.call(&mut store, (result_ptr, (len + 8) as u64, 0))?;

    trace!("free memory");

    if let Some(err_s) = String::get_value_opt(&output_value, ["plugin-panic"]) {
        return Err(anyhow!("Plugin panicked: {}", err_s));
    }

    // if let Some(formatted) = String::get_value_opt(&output_value, ["formatted-content"]) {
    //     let target = String::get_value(&cur_map, ["target"])?;
    //     fs::write(target, formatted)?;
    //
    //     trace!("wrote");
    // }

    // drop of store is so slow, another thread dropping is maybe faster

    Ok(output_value)
}

fn run_plugin_inner_native(library: &mut Library, cur_map: Value) -> Result<Value> {
    let input_data: Vec<u8> = serde_json::to_vec(&cur_map)?;
    let input_len = input_data.len();

    trace!("real run started");

    let func = library.get_function::<(u64, u64), u64>("foro_main")?;

    // In order to provide the same interface as the wasm plugin,
    // there are the same restrictions on the `foro_main` abi as the wasm plugin.
    //
    // For details, please check the comments in `run_plugin_inner_wasm`.

    let result_ptr_u64 = func.call(library, (input_data.as_ptr() as u64, input_len as u64));
    let result_ptr = result_ptr_u64 as *mut u8;

    trace!("real run ended");

    let len_part = unsafe { std::slice::from_raw_parts(result_ptr, 8) };
    let len = u64::from_le_bytes(len_part.try_into()?) as usize;

    let output_data = unsafe { std::slice::from_raw_parts(result_ptr.add(8), len) };
    let output_value: Value = serde_json::from_slice(output_data)?;

    if let Some(err_s) = String::get_value_opt(&output_value, ["plugin-panic"]) {
        return Err(anyhow!("Plugin panicked: {}", err_s));
    }

    // if let Some(formatted) = String::get_value_opt(&output_value, ["formatted-content"]) {
    //     let target = String::get_value(&cur_map, ["target"])?;
    //     fs::write(target, formatted)?;
    //
    //     trace!("wrote");
    // }

    Ok(output_value)
}

fn run_plugin_inner(library: &mut Library, cur_map: Value) -> Result<Value> {
    match library {
        Library::WasmLibrary(WasmLibrary { instance, store }) => {
            run_plugin_inner_wasm(instance.clone(), store, cur_map)
        }
        Library::NativeLibrary(NativeLibrary { .. }) => run_plugin_inner_native(library, cur_map),
    }
}

fn run_plugin(
    setting: PluginSetting,
    cur_json: Value,
    cache_path: &PathBuf,
    use_cache: bool,
) -> Result<Value> {
    let use_cache = use_cache && setting.cache;

    if use_cache {
        return run_multi_cached(&setting.source, cache_path, |lib| {
            run_plugin_inner(lib, cur_json.clone())
        });
    }

    let mut lib = load(&setting.source, cache_path)?;

    run_plugin_inner(&mut lib, cur_json)
}

fn run_inner_pure_command(
    command: &PureCommand,
    mut cur_json: Value,
    cache_path: &PathBuf,
    use_cache: bool,
) -> Result<Value> {
    match command {
        PureCommand::PluginUrl(url) => {
            let setting = PluginSetting {
                source: url.clone(),
                cache: true,
            };

            let res = run_plugin(setting, cur_json.clone(), cache_path, use_cache)?;

            merge(&mut cur_json, &res);

            Ok(cur_json)
        }
        PureCommand::CommandIO { io: cmd } => {
            let env = minijinja::Environment::new();
            let rendered_cmd = env.render_str(cmd, &cur_json)?;

            if cfg!(target_os = "windows") {
                // memo: we can use https://github.com/chipsenkbeil/winsplit-rs
                todo!("Windows is not supported yet")
            } else {
                #[cfg(unix)]
                let words = shell_words::split(&rendered_cmd)?;
                #[cfg(windows)]
                let words = winsplit::split(&rendered_cmd);

                let (exec, args) = words.split_first().context("Empty command")?;
                let _target_path = String::get_value(&cur_json, ["os-target"])?;
                let target_content = String::get_value(&cur_json, ["target-content"])?;
                let current_dir = String::get_value(&cur_json, ["current-dir"])?;

                debug_long!(
                    "exec: {:?}, args: {:?}, current_dir: {:?}",
                    exec,
                    args,
                    current_dir
                );

                let mut child = std::process::Command::new(exec)
                    .args(args)
                    .current_dir(current_dir)
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .spawn()
                    .context("Failed to execute command")?;

                trace!("spawned",);

                let mut stdin = child.stdin.take().unwrap();

                stdin.write_all(target_content.as_bytes())?;

                drop(stdin);

                trace!("writed");

                let exit_status = child.wait()?;

                if exit_status.success() {
                    let stdout = child.stdout.as_mut().unwrap();

                    let mut buf = String::new();
                    stdout.read_to_string(&mut buf)?;

                    let cur_json_m = cur_json.as_object_mut().unwrap();
                    cur_json_m.insert("format-status".to_string(), json!("success"));
                    cur_json_m.insert("formatted-content".to_string(), json!(buf));
                } else {
                    let stderr = child.stderr.as_mut().unwrap();

                    let mut buf = String::new();
                    stderr.read_to_string(&mut buf)?;

                    let cur_json_m = cur_json.as_object_mut().unwrap();
                    cur_json_m.insert("format-status".to_string(), json!("error"));
                    cur_json_m.insert("format-error".to_string(), json!(buf));
                }
            };

            Ok(cur_json)
        }
    }
}

fn run_inner_write_command(
    command: &WriteCommand,
    cur_json: Value,
    cache_path: &PathBuf,
    use_cache: bool,
) -> Result<Value> {
    match command {
        WriteCommand::SimpleCommand(cmd) => {
            let env = minijinja::Environment::new();
            let rendered_cmd = env.render_str(cmd, &cur_json)?;

            #[cfg(unix)]
            let words = shell_words::split(&rendered_cmd)?;
            #[cfg(windows)]
            let words = winsplit::split(&rendered_cmd);

            let (exec, args) = words.split_first().context("Empty command")?;
            let current_dir = String::get_value(&cur_json, ["current-dir"])?;

            debug_long!(
                "exec: {:?}, args: {:?}, current_dir: {:?}",
                exec,
                args,
                current_dir
            );

            let mut output = std::process::Command::new(exec)
                .args(args)
                .current_dir(current_dir)
                .spawn()
                .context("Failed to execute command")?;

            trace!("spawned");

            output.wait()?;

            trace_long!("output: {:?}", output);

            Ok(json!({}))
        }
        WriteCommand::Pure(pure) => {
            let target_path = String::get_value(&cur_json, ["os-target"])?;

            let res = run_inner_pure_command(pure, cur_json, cache_path, use_cache)?;

            if let Some(formatted) = String::get_value_opt(&res, ["formatted-content"]) {
                fs::write(target_path, formatted)?;
            }

            Ok(res)
        }
    }
}

/// Execute the pure command or write command CommandWithControlFlow.
///
/// There are two types of commands in foro:
/// - pure commands, which return the code received as a string without actually writing it to a file.
/// - write commands, which write directly to a file at runtime.
///
/// Although write commands can contain pure commands, the reverse is not possible,
/// and since the two share most of the control flow, etc.,
/// the implementation is a little abstract and difficult to understand,
/// via [run_flow], [CommandWithControlFlow] etc.
fn run_flow<T>(
    command_with_control_flow: &CommandWithControlFlow<T>,
    run_inner: fn(&T, Value, &PathBuf, bool) -> Result<Value>,
    mut cur_json: Value,
    cache_path: &PathBuf,
    use_cache: bool,
) -> Result<Value> {
    match command_with_control_flow {
        CommandWithControlFlow::If {
            run,
            cond,
            on_true,
            on_false,
        } => {
            trace!("if");

            let run_res = run_flow(run, run_inner, cur_json, cache_path, use_cache)?;

            let env = minijinja::Environment::new();
            let cond_expr = env.compile_expression(cond)?;
            let cond_res = cond_expr.eval(&run_res)?;
            let cond_bool = cond_res.is_true();

            trace!("cond: {:?}", cond);
            trace!("cond_bool: {:?}", cond_bool);

            let res = if cond_bool {
                run_flow(on_true, run_inner, run_res, cache_path, use_cache)
            } else {
                run_flow(on_false, run_inner, run_res, cache_path, use_cache)
            }?;

            trace!("if done");

            Ok(res)
        }
        CommandWithControlFlow::Sequential(seq) => {
            trace!("seq");

            for command in seq {
                cur_json = run_flow(command, run_inner, cur_json, cache_path, use_cache)?;
            }

            trace!("seq done");

            Ok(cur_json)
        }
        CommandWithControlFlow::Set { set } => {
            trace!("set");

            let env = minijinja::Environment::new();

            for (key, value) in set {
                let value_expr = env.compile_expression(value)?;
                let value_res = value_expr.eval(&cur_json)?;

                trace!("set key: {:?}, value: {:?}", key, value_res);

                cur_json[key] = to_value(value_res)?;
            }

            trace!("set done");

            Ok(cur_json)
        }
        CommandWithControlFlow::Command(cmd) => {
            trace!("cmd");

            let res = run_inner(cmd, cur_json, cache_path, use_cache)?;

            trace!("cmd done");

            Ok(res)
        }
    }
}

pub fn run(
    some_command: &SomeCommand,
    cur_json: Value,
    cache_path: &PathBuf,
    use_cache: bool,
) -> Result<Value> {
    debug!("run command: {:?}", some_command);
    debug_long!("data-json: {:?}", &cur_json);

    let res = match some_command {
        SomeCommand::Pure { cmd } => {
            let target_path = String::get_value(&cur_json, ["os-target"])?;
            let original_content = String::get_value(&cur_json, ["target-content"])?;

            let res = run_flow(cmd, run_inner_pure_command, cur_json, cache_path, use_cache)?;

            if let Some(formatted) = String::get_value_opt(&res, ["formatted-content"]) {
                if formatted != original_content {
                    fs::write(target_path, formatted)?;
                }
            }
            res
        }
        SomeCommand::Write { write_cmd } => {
            let res = run_flow(
                write_cmd,
                run_inner_write_command,
                cur_json,
                cache_path,
                use_cache,
            )?;

            res
        }
    };

    debug!("done command: {:?}", some_command);
    debug_long!("data-json: {:?}", &res);

    Ok(res)
}

pub fn run_pure(
    command: &CommandWithControlFlow<PureCommand>,
    cur_json: Value,
    cache_path: &PathBuf,
    use_cache: bool,
) -> Result<Value> {
    debug!("run pure command: {:?}", command);
    debug_long!("data-json: {:?}", &cur_json);

    let res = run_flow(
        command,
        run_inner_pure_command,
        cur_json,
        cache_path,
        use_cache,
    )?;

    debug!("done pure command: {:?}", command);
    debug_long!("data-json: {:?}", &res);

    Ok(res)
}
