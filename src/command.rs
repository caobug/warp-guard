use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, default_value = "1080")]
    pub listen_port: u16,

    #[arg(long, default_value = "10")]
    pub healthcheck_interval: u64,
    #[arg(long, default_value = "15")]
    pub healthcheck_start_period: u64,
    #[arg(long, default_value = "3")]
    pub healthcheck_retries: usize,
    #[arg(long, default_value = "10")]
    pub healthcheck_timeout: u64,

    #[arg(long, default_value = "2")]
    pub warp_cli_delay: u64,

    /// allow gost to display logs
    #[arg(long, default_value = "false")]
    pub display_gost_logs: u8,
}
