use luwaklib::deno_web::BlobStore;
use std::fs::{create_dir_all, write, File};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use luwaklib::cli_parser;
use luwaklib::compile;
use luwaklib::deno_broadcast_channel::InMemoryBroadcastChannel;
use luwaklib::deno_core::anyhow::Result;
use luwaklib::luwak_util::{info, init, dump_luwak_module_path};
use luwaklib::module::LuwakModule;
use luwaklib::permissions::PermissionsContainer;
use luwaklib::worker::{MainWorker, WorkerOptions};
use luwaklib::{deno_core, BootstrapOptions};
use tokio::runtime::Builder;

use crate::deno_core::error::AnyError;

fn get_error_class_name(e: &AnyError) -> &'static str {
    luwaklib::errors::get_error_class_name(e).unwrap_or("Error")
}

deno_core::extension!(
    luwak,
    esm_entry_point = "ext:luwak/init.js",
    esm = [dir "luwak", "init.js"]
);

fn main() -> Result<()> {
    let args = cli_parser::args();

    if args.info {
        println!("{}", info().unwrap());
        std::process::exit(0);
    }

    if args.init {
        init();
        std::process::exit(0);
    }

    if args.libdump {
        println!("🚀 All dependencies will be stored in the luwak_module directory...");
        dump_luwak_module_path().unwrap();
    }

    let module_loader = Rc::new(LuwakModule);
    let create_web_worker_cb = Arc::new(|_| {
        todo!("Web workers are not supported");
    });

    let options = WorkerOptions {
        bootstrap: BootstrapOptions {
            args: args.js_option,
            cpu_count: args.cpu,
            inspect: args.debug,
            enable_testing_features: false,
            location: None,
            no_color: false,
            is_tty: args.tty,
            runtime_version: Default::default(),
            ts_version: Default::default(),
            unstable: true,
            user_agent: Default::default(),
            has_node_modules_dir: true,
            locale: Default::default(),
            log_level: Default::default(),
            maybe_binary_npm_command_name: Default::default(),
        },
        extensions: vec![luwak::init_ops_and_esm()],
        unsafely_ignore_certificate_errors: None,
        seed: None,
        source_map_getter: None,
        format_js_error_fn: None,
        create_web_worker_cb,
        maybe_inspector_server: None,
        should_break_on_first_statement: false,
        module_loader,
        npm_resolver: None,
        get_error_class_fn: Some(&get_error_class_name),
        origin_storage_dir: None,
        blob_store: BlobStore::default().into(),
        broadcast_channel: InMemoryBroadcastChannel::default(),
        shared_array_buffer_store: None,
        compiled_wasm_module_store: None,
        stdio: Default::default(),
        cache_storage_dir: Default::default(),
        create_params: Default::default(),
        fs: Arc::new(deno_fs::RealFs),
        root_cert_store_provider: Default::default(),
        should_wait_for_inspector_session: Default::default(),
        startup_snapshot: Default::default(),
    };

    if args.download != "" {
        let download_script = format!("#!/bin/bash\nluwak {} $@", &args.js_script);
        let download_bin = format!("{}/.luwak/bin", env!("HOME"));
        let download_path = format!("{}/{}", download_bin, args.download);
        if !Path::new(&download_bin).exists() {
            create_dir_all(&download_bin)
                .expect("Error encountered while creating luwak bin directory!");
        }
        if !Path::new(&download_path).exists() {
            File::create(&download_path).expect("Unable to create luwak file script");
            let metadata = std::fs::metadata(&download_path)?;
            let mut perm = metadata.permissions();
            perm.set_mode(0o755);
            std::fs::set_permissions(&download_path, perm)?;
        }
        write(download_path, download_script).expect("Unable to write luwak file");
    }

    let js_path = Path::new(&args.js_script);
    let cwd = std::env::current_dir().unwrap();

    let main_module = deno_core::resolve_url_or_path(&js_path.to_string_lossy(), &cwd)?;
    let permissions = PermissionsContainer::allow_all();

    let rt = Builder::new_current_thread().enable_all().build()?;

    let fut = async move {
        if args.compile {
            let out = if args.out != "" {
                PathBuf::from(args.out)
            } else {
                std::env::current_dir().unwrap().join("out.bin")
            };
            let _ = compile::do_pkg(&js_path.to_path_buf(), &out).await;
            std::process::exit(0);
        }

        let mut worker =
            MainWorker::bootstrap_from_options(main_module.clone(), permissions, options);

        worker.execute_main_module(&main_module).await?;
        worker.run_event_loop(false).await?;
        Ok::<_, AnyError>(())
    };

    let local = tokio::task::LocalSet::new();
    local.block_on(&rt, fut)?;

    Ok(())
}
