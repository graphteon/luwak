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

    /// Dump libraries to luwaklibs.lock
    #[clap(short, long, value_parser)]
    pub libdump: bool,

    /// Download javascript to bin
    #[clap(short, long, value_parser, default_value = "")]
    pub download: String,

    /// Dump libraries to luwaklibs.lock
    #[clap(short, long, value_parser)]
    pub compile: bool,
}

pub fn args() -> Args {
    Args::parse()
}
