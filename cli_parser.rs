use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Javascript file location
    #[clap(value_parser)]
    pub js_script: String,

    /// Number of cpu
    #[clap(short, long, value_parser, default_value_t = std::thread::available_parallelism().map(|p| p.get()).unwrap_or(1))]
    pub cpu: usize,

    /// Enable tty
    #[clap(short, long, value_parser)]
    pub tty: bool,

    /// Enable debuging flags
    #[clap(short, long, value_parser)]
    pub debug: bool,

    /// Dump libraries to luwak_deps.js
    #[clap(short, long, value_parser)]
    pub libdump: bool,
}

pub fn args() -> Args {
    Args::parse()
}