#![warn(
    clippy::all,
    clippy::pedantic,
    future_incompatible,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    nonstandard_style,
    noop_method_call,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    trivial_casts,
    unreachable_pub,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused
)]
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::items_after_statements,
    clippy::many_single_char_names,
    clippy::match_on_vec_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::unreadable_literal
)]
#![feature(array_windows, once_cell)]

use std::fmt::Debug;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use clap::StructOpt;
use eyre::Result;
use memedsn::lexer::Lexer;
use memedsn::parser::Parser;
use memeroute::dsn::design_to_pcb::DesignToPcb;
use memeroute::model::pcb::Pcb;

use crate::gui::MemerouteGui;

pub mod gui;
pub mod pcb;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[must_use]
#[derive(Debug, clap::Parser)]
#[clap(name = "memeroute", about = "Memeroute GUI")]
struct Args {
    /// Path to data
    #[clap(short, long, parse(from_os_str))]
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

pub fn run() -> Result<()> {
    let args = Args::parse();
    let pcb = load_pcb(&args.data_path)?;
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "memeroute",
        options,
        Box::new(|cc| Box::new(MemerouteGui::new(pcb, args.data_path, cc))),
    );
    Ok(())
}
