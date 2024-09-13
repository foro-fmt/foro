use crate::app_dir::cache_dir_res;
use crate::config::Command;
use crate::handle_plugin::load::{_load_module_base, load_local_module, load_url_module};
use anyhow::{anyhow, Context, Result};
use foro_plugin_utils::data_json_utils::{merge, JsonGetter};
use log::{debug, info, trace};
use minijinja;
use serde_json::{json, Value};
use shell_words;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::format;
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};
use std::thread::sleep;
use std::time::Instant;
use std::{env, fs, thread, time};
use url::Url;
use wasmtime::{
    AsContextMut, Config, Engine, Instance, Linker, Module, Store, StoreContext, StoreContextMut,
    Val,
};
use wasmtime_wasi::preview1::WasiP1Ctx;
use wasmtime_wasi::{preview1, DirPerms, FilePerms, WasiCtxBuilder};

#[derive(Hash, Eq, PartialEq)]
pub(crate) enum FileSource {
    LocalPath(PathBuf),
    Url(Url),
}

pub(crate) struct PluginSetting {
    pub(crate) wasm_source: FileSource,
    pub(crate) cache: bool,
}

static CACHE_WASM_STORE: LazyLock<Mutex<HashMap<FileSource, (Instance, Mutex<Store<WasiP1Ctx>>)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn init_instance(
    setting: &PluginSetting,
    cache_path: &PathBuf,
    use_cache: bool,
) -> Result<(Instance, Store<WasiP1Ctx>)> {
    trace!("start init instance");

    let mut config = Config::default();
    // https://github.com/bytecodealliance/wasmtime/issues/8897
    #[cfg(unix)]
    config.native_unwind_info(false);
    let engine = Engine::new(&config)?;

    trace!("loaded engine");

    let module = match &setting.wasm_source {
        FileSource::LocalPath(path) => load_local_module(&engine, path, cache_path, use_cache)?,
        FileSource::Url(url) => load_url_module(&engine, url, cache_path, use_cache)?,
    };

    trace!("loaded module");

    let mut linker = Linker::new(&engine);

    preview1::add_to_linker_sync(&mut linker, |t| t)?;
    let pre = linker.instantiate_pre(&module)?;

    trace!("loaded linker");

    let wasi_ctx = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_env()
        .preopened_dir("/", "/", DirPerms::all(), FilePerms::all())?
        .build_p1();

    trace!("loaded wasi_ctx");

    let mut store = Store::new(&engine, wasi_ctx);
    let instance = pre.instantiate(&mut store)?;

    trace!("loaded instance");

    Ok((instance, store))
}

pub(crate) fn run_plugin_inner(
    instance: Instance,
    mut store: &mut Store<WasiP1Ctx>,
    cur_map: Value,
) -> Result<Value> {
    let memory = instance
        .get_memory(&mut store, "memory")
        .context("Failed to get memory")?;

    trace!("loaded memory");

    let func = instance.get_typed_func::<(u64, u64), u64>(&mut store, "main")?;
    let malloc = instance.get_typed_func::<(u64, u64), u64>(&mut store, "of_malloc")?;
    let free = instance.get_typed_func::<(u64, u64, u64), ()>(&mut store, "of_free")?;

    trace!("loaded functions");

    let input_data: Vec<u8> = serde_json::to_vec(&cur_map)?;
    let input_len = input_data.len() as u64;

    let data_ptr = malloc.call(&mut store, (input_data.len() as u64, 0))?;

    &memory.write(&mut store, data_ptr as usize, &input_data)?;

    trace!("loaded input data");

    trace!("real start run");

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

    if let Some(formatted) = String::get_value_opt(&output_value, ["formatted-content"]) {
        let target = String::get_value(&cur_map, ["target"])?;
        fs::write(target, formatted)?;

        trace!("wrote");
    }

    // drop of store is so slow, another thread dropping is maybe faster

    Ok(output_value)
}

pub(crate) fn run_plugin(
    setting: PluginSetting,
    cur_map: Value,
    cache_path: &PathBuf,
    use_cache: bool,
) -> Result<Value> {
    let use_cache = use_cache && setting.cache;

    if use_cache {
        if let Some((instance, store)) = CACHE_WASM_STORE.lock().unwrap().get(&setting.wasm_source)
        {
            debug!("loaded from in-memory cache");
            return run_plugin_inner(instance.clone(), store.lock().unwrap().deref_mut(), cur_map);
        }
    }

    let (instance, mut store) = init_instance(&setting, cache_path, use_cache)?;

    if use_cache {
        let res = run_plugin_inner(instance.clone(), &mut store, cur_map);

        CACHE_WASM_STORE
            .lock()
            .unwrap()
            .insert(setting.wasm_source, (instance.clone(), Mutex::new(store)));

        return res;
    }

    run_plugin_inner(instance.clone(), &mut store, cur_map)
}

static CACHE_NATIVE_LIB: LazyLock<Mutex<HashMap<String, (libloading::Library)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) unsafe fn run_plugin_native_inner(
    func: libloading::Symbol<unsafe extern "C" fn(ptr: u64, len: u64) -> u64>,
    cur_map: Value,
) -> Result<Value> {
    let input_data: Vec<u8> = serde_json::to_vec(&cur_map)?;
    let input_len = input_data.len();

    let result_ptr_u64 = func(input_data.as_ptr() as u64, input_len as u64);
    let result_ptr = result_ptr_u64 as *mut u8;

    let len_part = std::slice::from_raw_parts(result_ptr, 8);
    let len = u64::from_le_bytes(len_part.try_into()?) as usize;

    let output_data = std::slice::from_raw_parts(result_ptr.add(8), len);
    let output_value: Value = serde_json::from_slice(output_data)?;

    if let Some(err_s) = String::get_value_opt(&output_value, ["plugin-panic"]) {
        return Err(anyhow!("Plugin panicked: {}", err_s));
    }

    if let Some(formatted) = String::get_value_opt(&output_value, ["formatted-content"]) {
        let target = String::get_value(&cur_map, ["target"])?;
        fs::write(target, formatted)?;

        trace!("wrote");
    }

    Ok(output_value)
}

pub(crate) fn run_plugin_native(
    dll_path: &str,
    cur_map: Value,
    cache_path: &PathBuf,
    use_cache: bool,
) -> Result<Value> {
    unsafe {
        trace!("0");

        if let Some(lib) = CACHE_NATIVE_LIB.lock().unwrap().get(&dll_path.to_string()) {
            debug!("loaded from in-memory cache");

            let func: libloading::Symbol<unsafe extern "C" fn(ptr: u64, len: u64) -> u64> =
                lib.get(b"main")?;

            return run_plugin_native_inner(func, cur_map);
        }

        trace!("1");

        let lib = libloading::Library::new(dll_path)?;
        trace!("2");

        let func: libloading::Symbol<unsafe extern "C" fn(ptr: u64, len: u64) -> u64> =
            lib.get(b"main")?;

        trace!("loaded function");

        let input_data: Vec<u8> = serde_json::to_vec(&cur_map)?;
        let input_len = input_data.len();

        let result_ptr_u64 = func(input_data.as_ptr() as u64, input_len as u64);
        let result_ptr = result_ptr_u64 as *mut u8;

        let len_part = std::slice::from_raw_parts(result_ptr, 8);
        let len = u64::from_le_bytes(len_part.try_into()?) as usize;

        let output_data = std::slice::from_raw_parts(result_ptr.add(8), len);
        let output_value: Value = serde_json::from_slice(output_data)?;

        CACHE_NATIVE_LIB
            .lock()
            .unwrap()
            .insert(dll_path.to_string(), lib);

        Ok(output_value)
    }
}

pub fn run(
    command: &Command,
    mut cur_json: Value,
    cache_path: &PathBuf,
    use_cache: bool,
) -> Result<Value> {
    debug!("run command: {:?}", command);
    // debug!("data-json: {:?}", &cur_json);

    let res = match command {
        Command::PluginUrl(url) => {
            let setting = PluginSetting {
                wasm_source: FileSource::Url(url.clone().into_inner()),
                cache: true,
            };
            run_plugin(setting, cur_json, cache_path, use_cache)
        }
        Command::SimpleCommand(cmd) => {
            let env = minijinja::Environment::new();
            let rendered_cmd = env.render_str(cmd, &cur_json)?;

            let output = if cfg!(target_os = "windows") {
                let words = winsplit::split(&rendered_cmd);
                let (exec, args) = words.split_first().context("Empty command")?;
                let current_dir = String::get_value(&cur_json, ["current-dir"])?;

                debug!(
                    "exec: {:?}, args: {:?}, current_dir: {:?}",
                    exec, args, current_dir
                );

                let mut output = std::process::Command::new(exec)
                    .args(args)
                    .current_dir(current_dir)
                    .spawn()
                    .context("Failed to execute command")?;

                output.wait()?;

                trace!("output: {:?}", output);
            } else {
                let words = shell_words::split(&rendered_cmd)?;
                let (exec, args) = words.split_first().context("Empty command")?;
                let current_dir = String::get_value(&cur_json, ["current-dir"])?;

                debug!(
                    "exec: {:?}, args: {:?}, current_dir: {:?}",
                    exec, args, current_dir
                );

                let mut output = std::process::Command::new(exec)
                    .args(args)
                    .current_dir(current_dir)
                    .spawn()
                    .context("Failed to execute command")?;

                output.wait()?;

                trace!("output: {:?}", output);
            };

            Ok(json!({}))
        }
        Command::CommandIO { io: cmd } => {
            let env = minijinja::Environment::new();
            let rendered_cmd = env.render_str(cmd, &cur_json)?;

            let output = if cfg!(target_os = "windows") {
                // memo: we can use https://github.com/chipsenkbeil/winsplit-rs
                todo!("Windows is not supported yet")
            } else {
                let words = shell_words::split(&rendered_cmd)?;
                let (exec, args) = words.split_first().context("Empty command")?;
                let target_path = String::get_value(&cur_json, ["target"])?;
                let target_content = String::get_value(&cur_json, ["target-content"])?;
                let current_dir = String::get_value(&cur_json, ["current-dir"])?;

                debug!(
                    "exec: {:?}, args: {:?}, current_dir: {:?}",
                    exec, args, current_dir
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

                trace!("writed",);

                let mut stdout = child.stdout.as_mut().unwrap();

                let mut buf = String::new();
                stdout.read_to_string(&mut buf)?;

                debug!("write to {}", target_path);

                fs::write(target_path, buf.clone())?;
            };

            Ok(json!({}))
        }
        Command::Finding {
            finding,
            if_found,
            else_,
        } => {
            let finding_res = run(finding, cur_json.clone(), cache_path, use_cache)?;

            merge(&mut cur_json, &finding_res);

            if bool::get_value(&finding_res, ["found"]).unwrap_or(false) {
                run(if_found, cur_json, cache_path, use_cache)
            } else {
                let res = run(else_, cur_json.clone(), cache_path, use_cache)?;

                Ok(res)
            }
        }
        Command::Sequential(commands) => {
            for command in commands {
                let res = run(command, cur_json.clone(), cache_path, use_cache)?;

                merge(&mut cur_json, &res);
            }

            Ok(cur_json)
        }
        Command::NativeDll {
            native_dll: dll_path,
        } => {
            let res = run_plugin_native(dll_path, cur_json.clone(), cache_path, use_cache)?;

            // debug!("done native: {:?}", res);

            Ok(cur_json)
        }
    };

    debug!("done command: {:?}", command);
    // debug!("data-json: {:?}", &res);

    res
}
