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
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use eyre::Result;
use memeroute::dsn::design_to_pcb::DesignToPcb;
use memeroute::dsn::lexer::Lexer;
use memeroute::dsn::parser::Parser;
use memeroute::model::pcb::Pcb;
use structopt::StructOpt;

use crate::gui::MemerouteGui;

pub mod gui;
pub mod pcb;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[derive(Debug, StructOpt)]
#[structopt(name = "memeroute", about = "Memeroute GUI")]
struct Args {
    /// Path to data
    #[structopt(short, long, parse(from_os_str))]
    data_path: PathBuf,
}

fn load_pcb<P: AsRef<Path>>(path: P) -> Result<Pcb> {
    let data = read_to_string(path)?;
    let lexer = Lexer::new(&data)?;
    let parser = Parser::new(&lexer.lex()?);
    let pcb = parser.parse()?;
    let pcb = DesignToPcb::new(pcb).convert()?;
    Ok(pcb)
}

pub async fn run() -> Result<()> {
    let args = Args::from_args();
    let pcb = load_pcb(&args.data_path)?;
    let app = MemerouteGui::new(pcb, args.data_path);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
    Ok(())
}
