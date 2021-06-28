#![warn(rust_2018_idioms, clippy::all)]
#![feature(
    array_chunks,
    array_windows,
    bool_to_option,
    destructuring_assignment,
    is_sorted,
    map_first_last,
    option_result_contains,
    stmt_expr_attributes,
    trait_alias
)]

use std::fmt::Debug;
use std::path::PathBuf;

use eyre::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "memeroute", about = "Memeroute GUI")]
struct Args {
    /// Path to data
    #[structopt(short, long, parse(from_os_str))]
    data_path: PathBuf,
}

pub async fn run() -> Result<()> {
    let args = Args::from_args();
    Ok(())
}
