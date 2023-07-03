// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod engines;
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
            let mut engine = engines::typescript::Engine::new();
            engines::typescript::Engine::print_preamble();
            engine.visit_cddl(&cddl)?;
        }
    };

    Ok(())
}
