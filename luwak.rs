use clap::Parser;
use deno_core::error::AnyError;
use luwaklib::deno_broadcast_channel::InMemoryBroadcastChannel;
use luwaklib::deno_web::BlobStore;
use luwaklib::module::LuwakModule;
use luwaklib::permissions::Permissions;
use luwaklib::worker::MainWorker;
use luwaklib::worker::WorkerOptions;
use luwaklib::BootstrapOptions;
use num_cpus;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

fn get_error_class_name(e: &AnyError) -> &'static str {
    luwaklib::errors::get_error_class_name(e).unwrap_or("Error")
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Javascript file location
    #[clap(value_parser)]
    js_script: String,

    /// Number of cpu
    #[clap(short, long, value_parser, default_value_t = num_cpus::get())]
    cpu: usize,

    /// Enable tty
    #[clap(short, long, value_parser)]
    tty: bool,

    /// Enable debuging flags
    #[clap(short, long, value_parser)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    let args = Args::parse();

    let module_loader = Rc::new(LuwakModule);
    let create_web_worker_cb = Arc::new(|_| {
        todo!("Web workers are not supported");
    });
    let web_worker_event_cb = Arc::new(|_| {
        todo!("Web workers are not supported");
    });

    let options = WorkerOptions {
        bootstrap: BootstrapOptions {
            args: vec![],
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

    let js_path = Path::new(&args.js_script);

    let main_module = deno_core::resolve_url_or_path(&js_path.to_string_lossy())?;
    let permissions = Permissions::allow_all();

    let mut worker = MainWorker::bootstrap_from_options(main_module.clone(), permissions, options);
    worker.execute_main_module(&main_module).await?;
    worker.run_event_loop(false).await?;
    Ok(())
}
