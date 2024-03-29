use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Javascript file location
    #[clap(value_parser)]
    pub js_script: String,

    #[clap(multiple = true, allow_hyphen_values = true)]
    pub js_option: Vec<String>,

    /// Number of cpu
    #[clap(long, value_parser, default_value_t = std::thread::available_parallelism().map(|p| p.get()).unwrap_or(1))]
    pub cpu: usize,

    /// Enable tty
    #[clap(long, value_parser)]
    pub tty: bool,

    /// Enable debuging flags
    #[clap(long, value_parser)]
    pub debug: bool,

    /// All dependencies will be stored in the luwak_module directory
    #[clap(short, long, value_parser)]
    pub install: bool,

    /// Download javascript to bin
    #[clap(short, long, value_parser, default_value = "")]
    pub download: String,

    /// Standalone binary output
    #[clap(short, long, value_parser, default_value = "")]
    pub out: String,

    /// Package your library to standalone binary
    #[clap(short, long, value_parser)]
    pub compile: bool,

    /// Enable debuging flags
    #[clap(long, value_parser)]
    pub info: bool,

    /// Create and initilize new luwak apps
    #[clap(long, value_parser)]
    pub init: bool,
}

pub fn args() -> Args {
    Args::parse()
}
