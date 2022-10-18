use luwaklib::deno_web::BlobStore;
use std::fs::{create_dir_all, write, File};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use luwaklib::cli_parser;
use luwaklib::deno_broadcast_channel::InMemoryBroadcastChannel;
use luwaklib::deno_core::anyhow::Result;
use luwaklib::module::LuwakModule;
use luwaklib::permissions::Permissions;
use luwaklib::worker::{MainWorker, WorkerOptions};
use luwaklib::{deno_core, BootstrapOptions};
use tokio::runtime::Builder;

use crate::deno_core::error::AnyError;

fn get_error_class_name(e: &AnyError) -> &'static str {
    luwaklib::errors::get_error_class_name(e).unwrap_or("Error")
}
fn main() -> Result<()> {
    let args = cli_parser::args();

    let module_loader = Rc::new(LuwakModule);
    let create_web_worker_cb = Arc::new(|_| {
        todo!("Web workers are not supported");
    });
    let web_worker_event_cb = Arc::new(|_| {
        todo!("Web workers are not supported");
    });

    let options = WorkerOptions {
        bootstrap: BootstrapOptions {
            args: args.js_option,
            cpu_count: args.cpu,
            debug_flag: args.debug,
            enable_testing_features: false,
            location: None,
            no_color: false,
            is_tty: args.tty,
            runtime_version: "1.0.0".to_string(),
            ts_version: "x".to_string(),
            unstable: false,
            user_agent: "luwak".to_string(),
        },
        extensions: vec![],
        unsafely_ignore_certificate_errors: None,
        root_cert_store: None,
        seed: None,
        source_map_getter: None,
        format_js_error_fn: None,
        web_worker_preload_module_cb: web_worker_event_cb.clone(),
        web_worker_pre_execute_module_cb: web_worker_event_cb,
        create_web_worker_cb,
        maybe_inspector_server: None,
        should_break_on_first_statement: false,
        module_loader,
        npm_resolver: None,
        get_error_class_fn: Some(&get_error_class_name),
        origin_storage_dir: None,
        blob_store: BlobStore::default(),
        broadcast_channel: InMemoryBroadcastChannel::default(),
        shared_array_buffer_store: None,
        compiled_wasm_module_store: None,
        stdio: Default::default(),
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

    let main_module = deno_core::resolve_url_or_path(&js_path.to_string_lossy())?;
    let permissions = Permissions::allow_all();

    let rt = Builder::new_current_thread().enable_all().build()?;

    let fut = async move {
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
