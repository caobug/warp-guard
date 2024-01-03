use std::io;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use clap::Parser;
use log::{error, info, warn, LevelFilter};
use tokio::process::{Child, Command};
use tokio::{select, signal};

use crate::command::Args;

mod command;

/// failed to start warp-svc program.
const EXIT_CODE_WARP_ERR: i32 = 1;
/// failed to start gost program.
const EXIT_CODE_GOST_ERR: i32 = 2;
/// failed to stop a specific program, we have to exit the process.
const EXIT_CODE_STOP_ERR: i32 = 3;
/// stop process from external request.
const EXTERNAL_STOP_TIMEOUT: Duration = Duration::from_secs(3);
/// stop process from internal request.
const INTERNAL_STOP_TIMEOUT: Duration = Duration::from_secs(5);

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .format_target(false)
        .init();
    let args = Args::parse();
    let stop_programs = |warp: Child, proxy: Child| async {
        stop_program(proxy, INTERNAL_STOP_TIMEOUT).await;
        stop_program(warp, INTERNAL_STOP_TIMEOUT).await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    };
    loop {
        let mut warp = start_warp();
        let mut proxy = start_proxy(&args);
        // we have to delay calling warp-cli
        select! {
            _ = tokio::time::sleep(Duration::from_secs(args.warp_cli_delay)) => {
                connect_warp().await;
            },
            _ = signal::ctrl_c() => {
                stop_program(proxy, EXTERNAL_STOP_TIMEOUT).await;
                stop_program(warp, EXTERNAL_STOP_TIMEOUT).await;
                break;
            },
        }
        // wait for all child processes to exit
        let result: io::Result<ExitStatus> = select! {
            r = warp.wait() => r,
            r = proxy.wait() => r,
            _ = signal::ctrl_c() => {
                stop_program(proxy, EXTERNAL_STOP_TIMEOUT).await;
                stop_program(warp, EXTERNAL_STOP_TIMEOUT).await;
                break;
            },
            _ = start_healthcheck(&args) => {
                stop_programs(warp, proxy).await;
                continue;
            },
        };
        match result {
            Ok(code) => info!("exit early: {}", code.code().unwrap_or(-1)),
            Err(e) => warn!("execute error: {}", e),
        };
        stop_programs(warp, proxy).await;
    }
}

/// start a warp-svc program
fn start_warp() -> Child {
    match Command::new("warp-svc").stdin(Stdio::null()).spawn() {
        Ok(val) => val,
        Err(e) => {
            error!(
                "failed to call warp-svc (https://pkg.cloudflareclient.com/), reason = {}",
                e
            );
            std::process::exit(EXIT_CODE_WARP_ERR);
        }
    }
}

/// call: warp-cli --accept-tos connect
async fn connect_warp() {
    match Command::new("warp-cli")
        .args(["--accept-tos", "connect"])
        .output()
        .await
    {
        Ok(_) => info!("successfully connected"),
        Err(e) => {
            error!("failed to connect, reason = {}", e);
            std::process::exit(EXIT_CODE_WARP_ERR);
        }
    }
}

/// start a gost proxy program
fn start_proxy(args: &Args) -> Child {
    let stdio_cfg = || {
        if args.display_gost_logs == 1 {
            Stdio::inherit()
        } else {
            Stdio::null()
        }
    };
    match Command::new("gost")
        .args(["-L", &format!(":{}", args.listen_port)])
        .stdin(Stdio::null())
        .stdout(stdio_cfg())
        .stderr(stdio_cfg())
        .spawn()
    {
        Ok(val) => val,
        Err(e) => {
            error!(
                "failed to call gost (https://github.com/ginuerzh/gost), reason = {}",
                e
            );
            std::process::exit(EXIT_CODE_GOST_ERR);
        }
    }
}

/// check the warp service availability.
async fn start_healthcheck(args: &Args) {
    let endpoint = "https://cloudflare.com/cdn-cgi/trace";
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(args.healthcheck_timeout))
        .build()
        .unwrap();
    let mut total_errors = 0;
    tokio::time::sleep(Duration::from_secs(args.healthcheck_start_period)).await;
    loop {
        if total_errors >= args.healthcheck_retries {
            error!(
                "check failed, retries = {}/{}, peer timeout = {}s",
                total_errors, args.healthcheck_retries, args.healthcheck_timeout
            );
            break;
        }
        tokio::time::sleep(Duration::from_secs(args.healthcheck_interval)).await;
        let url = reqwest::Url::parse(endpoint).expect("invalid endpoint");
        let resp = match client.get(url).send().await {
            Ok(val) => val,
            Err(e) => {
                total_errors += 1;
                warn!("request failed, reason = {}", e);
                continue;
            }
        };
        let text = match resp.text().await {
            Ok(val) => val,
            Err(e) => {
                total_errors += 1;
                warn!("invalid response, reason = {}", e);
                continue;
            }
        };
        match text.lines().find(|x| *x == "warp=plus" || *x == "warp=on") {
            None => {
                warn!("warp isn't enabled\n{}", text);
                break;
            }
            Some(line) => {
                total_errors = 0;
                info!("warp is normal, {}", line);
            }
        }
    }
}

/// stop a program.
async fn stop_program(mut child: Child, timeout: Duration) {
    let id = child.id().unwrap_or(0);
    match tokio::time::timeout(timeout, child.kill()).await {
        Ok(val) => match val {
            Ok(_) => info!("stop success, pid = {}", id),
            Err(e) => error!("stop error, pid = {}, reason = {}", id, e),
        },
        Err(_) => std::process::exit(EXIT_CODE_STOP_ERR),
    }
}
