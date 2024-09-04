use crate::app_dir::cache_dir_res;
use crate::config::Command;
use crate::handle_plugin::load::{_load_module_base, load_local_module, load_url_module};
use anyhow::{Context, Result};
use log::{debug, info, trace};
use minijinja;
use onefmt_plugin_utils::data_json_utils::{merge, JsonGetter};
use serde_json::{json, Value};
use shell_words;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use url::Url;
use wasmtime::{Config, Engine, Instance, Linker, Module, Store, Val};
use wasmtime_wasi::preview1::WasiP1Ctx;
use wasmtime_wasi::{preview1, DirPerms, FilePerms, WasiCtxBuilder};

pub(crate) enum WasmSource {
    LocalPath(PathBuf),
    Url(Url),
}

pub(crate) struct PluginSetting {
    pub(crate) wasm_source: WasmSource,
    pub(crate) cache: bool,
}

fn init_instance(
    setting: PluginSetting,
    cache_path: &PathBuf,
    use_cache: bool,
) -> Result<(Instance, Store<WasiP1Ctx>)> {
    trace!("start init instance");

    let mut config = Config::default();
    // https://github.com/bytecodealliance/wasmtime/issues/8897
    config.native_unwind_info(false);
    let engine = Engine::new(&config)?;

    trace!("loaded engine");

    let module = match setting.wasm_source {
        WasmSource::LocalPath(path) => {
            load_local_module(&engine, path, cache_path, use_cache && setting.cache)?
        }
        WasmSource::Url(url) => {
            load_url_module(&engine, url, cache_path, use_cache && setting.cache)?
        }
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

pub(crate) fn run_plugin(
    setting: PluginSetting,
    cur_map: Value,
    cache_path: &PathBuf,
    use_cache: bool,
) -> Result<Value> {
    let (instance, mut store) = init_instance(setting, cache_path, use_cache)?;

    trace!("loaded instance");

    let memory = instance
        .get_memory(&mut store, "memory")
        .context("Failed to get memory")?;

    trace!("loaded memory");

    let func = instance.get_typed_func::<(i32, i32), i32>(&mut store, "main")?;
    let malloc = instance.get_typed_func::<(i32, i32), i32>(&mut store, "of_malloc")?;
    let free = instance.get_typed_func::<(i32, i32, i32), ()>(&mut store, "of_free")?;

    trace!("loaded functions");

    let input_data: Vec<u8> = serde_json::to_vec(&cur_map)?;
    let input_len = input_data.len() as i32;

    let data_ptr = malloc.call(&mut store, (input_data.len() as i32, 0))?;

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

    free.call(&mut store, (data_ptr, input_data.len() as i32, 0))?;
    free.call(&mut store, (result_ptr, (len + 8) as i32, 0))?;

    trace!("free memory");

    if let Some(err_s) = String::get_value_opt(&output_value, ["plugin-panic"]) {
        return Err(anyhow::anyhow!("Plugin panicked: {}", err_s));
    }

    if let Some(formatted) = String::get_value_opt(&output_value, ["formatted-content"]) {
        let target = String::get_value(&cur_map, ["target"])?;
        fs::write(target, formatted)?;

        trace!("wrote");
    }

    // drop of store is so slow, another thread dropping is maybe faster

    Ok(output_value)
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
                wasm_source: WasmSource::Url(url.clone().into_inner()),
                cache: true,
            };
            run_plugin(setting, cur_json, cache_path, use_cache)
        }
        Command::SimpleCommand(cmd) => {
            let env = minijinja::Environment::new();
            let rendered_cmd = env.render_str(cmd, &cur_json)?;

            let output = if cfg!(target_os = "windows") {
                // memo: we can use https://github.com/chipsenkbeil/winsplit-rs
                todo!("Windows is not supported yet")
            } else {
                let words = shell_words::split(&rendered_cmd)?;
                let (exec, args) = words.split_first().context("Empty command")?;

                debug!("exec: {:?}, args: {:?}", exec, args);

                let mut output = std::process::Command::new(exec)
                    .args(args)
                    .spawn()
                    .context("Failed to execute command")?;

                output.wait()?;

                debug!("output: {:?}", output);
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
    };

    debug!("done command: {:?}", command);
    // debug!("data-json: {:?}", &res);

    res
}
