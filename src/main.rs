mod type_script_engine;
mod util;

use anyhow::Result;
use cddl::visitor::Visitor;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum EngineType {
    TypeScript,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to convert.
    file: PathBuf,
    /// Format to output.
    #[arg(short, long, value_enum, default_value_t = EngineType::TypeScript)]
    format: EngineType,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input = std::fs::read_to_string(args.file)?;
    let cddl =
        cddl::parser::cddl_from_str(&input, true).map_err(|error| anyhow::Error::msg(error))?;

    match args.format {
        EngineType::TypeScript => {
            let mut engine = type_script_engine::TypeScriptEngine::new();
            engine.visit_cddl(&cddl)?;
        }
    };

    Ok(())
}
