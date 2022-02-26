#![warn(
    clippy::all,
    clippy::pedantic,
    future_incompatible,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    must_not_suspend,
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
#![feature(
    array_chunks,
    array_windows,
    bool_to_option,
    const_discriminant,
    const_for,
    const_mut_refs,
    const_trait_impl,
    drain_filter,
    is_sorted,
    map_first_last,
    must_not_suspend,
    once_cell,
    option_result_contains,
    stmt_expr_attributes,
    trait_alias
)]

use std::fmt::Debug;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use clap::StructOpt;
use eyre::Result;
use memeroute::dsn::design_to_pcb::DesignToPcb;
use memeroute::dsn::lexer::Lexer;
use memeroute::dsn::parser::Parser;
use memeroute::model::pcb::Pcb;

use crate::gui::MemerouteGui;

pub mod gui;
pub mod pcb;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

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
    let mut app = MemerouteGui::new(pcb, args.data_path);
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "memeroute gui",
        options,
        Box::new(|cc| {
            app.init(cc);
            Box::new(app)
        }),
    );
}
