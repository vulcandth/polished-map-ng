//! Experimental master-format CLI: arguments → commands → JSON envelope. LGPL-3.0-only.
//! The library backend also supports attributed/unified workflows; this CLI does not yet expose them.
mod args;
mod commands;
mod output;
mod printing;
use args::Cli;
use clap::Parser;
use commands::run;
use serde_json::json;
use std::process::ExitCode;

fn main() -> ExitCode {
    let (output, status) = match run(Cli::parse()) {
        Ok(result) => (
            json!({"schema_version":1, "ok":true, "result":result}),
            ExitCode::SUCCESS,
        ),
        Err(error) => (
            json!({"schema_version":1, "ok":false, "error":{"code":error.code,"message":error.message}}),
            ExitCode::FAILURE,
        ),
    };
    println!("{output}");
    status
}
