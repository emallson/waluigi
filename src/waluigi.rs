#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;
extern crate serde_json;
extern crate docopt;
extern crate regex;
#[macro_use]
extern crate itertools;
#[macro_use]
extern crate maplit;
extern crate rustc_serialize;
extern crate glob;

mod structs;
mod errors;

use docopt::Docopt;
use std::fs::File;
use std::collections::HashMap;
use glob::glob;

use structs::*;
use errors::*;

#[cfg_attr(rustfmt, rustfmt_skip)]
const USAGE: &'static str = "
Waluigi task builder

Usage:
  waluigi debug <experiment> [options]
  waluigi (-h | --help)
  waluigi --version

Options:
  -h --help             Show this screen.
  --version             Show version information.
  --program <path>      Add <path> to program specifications. By default, ./ and ./programs/ are searched for program specifications.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    cmd_debug: bool,
    arg_experiment: String,
    flag_program: Vec<String>,
}

fn load_program_specs(given: Vec<String>) -> Result<HashMap<String, Program>> {
    let mut progs = vec![];
    for entry in glob("./*.yaml")
        .expect("failed to parse glob pattern")
        .chain(glob("./programs/*.yaml").expect("failed to parse glob pattern")) {
        let prog: Option<Program> = match entry {
            Ok(path) => {
                serde_yaml::from_reader(File::open(path.clone()).unwrap())
                    .map(|x| Some(x))
                    .unwrap_or_else(|e| {
                        // println!("failed to read program from {:?}: {:?}", path, e);
                        None
                    })
            }
            Err(_) => unreachable!(),
        };

        if let Some(p) = prog {
            progs.push(p);
        } else {
            continue;
        }
    }

    for path in given {
        progs.push(serde_yaml::from_reader(File::open(path)?)?);
    }

    Ok(progs.into_iter().map(|prog| (prog.name.clone(), prog)).collect())
}

fn load_experiment(experiment: String) -> Result<Experiment> {
    Ok(serde_yaml::from_reader(File::open(experiment)?)?)
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.version(Some(env!("CARGO_PKG_VERSION").to_string())).decode())
        .unwrap_or_else(|e| e.exit());

    let progs = load_program_specs(args.flag_program).unwrap();
    let exp = load_experiment(args.arg_experiment).unwrap();

    for job in exp.plan(1, &progs).unwrap() {
        println!("{}", serde_json::to_string(&job).unwrap());
    }
}
