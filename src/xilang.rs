extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate clap;
extern crate lazy_static;
extern crate regex;

mod ir;
mod lang;

use lang::module_mgr;

use clap::{App, Arg};

use std::path::PathBuf;
use std::time::SystemTime;

struct Config {
    class_path: Vec<String>,
    dir: PathBuf,
    out_dir: PathBuf,
    verbose: usize,
}

fn main() {
    let cfg: Config;

    {
        let matches = App::new("xivm")
            .version("0.1.0")
            .author("Xi")
            .about("Hello world! This is xivm")
            .arg(
                Arg::with_name("root")
                    .help("Crate root directory")
                    .required(true)
                    .index(1),
            )
            .arg(
                Arg::with_name("output")
                    .help("Output directory")
                    .short("o")
                    .long("output")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("cp")
                    .help("Additional class path")
                    .short("cp")
                    .long("classpath")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("v")
                    .short("v")
                    .multiple(true)
                    .help("Level of verbosity"),
            )
            .get_matches();

        let class_path = matches.value_of("cp").unwrap_or("");
        let input_dir = matches.value_of("root").unwrap();

        cfg = Config {
            class_path: class_path
                .split(';')
                .map(|x| x.to_owned())
                .collect::<Vec<String>>(), // TODO: 暂时没有cp
            dir: PathBuf::from(input_dir),
            out_dir: PathBuf::from(matches.value_of("cp").unwrap_or(input_dir)),
            verbose: matches.occurrences_of("v") as usize,
        };
    }

    let start_time = SystemTime::now();
    let mut module_mgr = module_mgr::ModuleMgr::new(&cfg.dir, &cfg.class_path, cfg.verbose >= 2);
    if cfg.verbose >= 1 {
        println!(
            "Parsing finished in {} seconds",
            SystemTime::now()
                .duration_since(start_time)
                .unwrap()
                .as_secs_f32()
        );
    }

    if cfg.verbose >= 1 {
        println!("Project structure:");
        module_mgr.tree();
    }

    let start_time = SystemTime::now();
    module_mgr.build();
    if cfg.verbose >= 1 {
        println!(
            "Build finished in {} seconds",
            SystemTime::now()
                .duration_since(start_time)
                .unwrap()
                .as_secs_f32()
        );
    }

    let start_time = SystemTime::now();
    module_mgr.dump(&cfg.out_dir);
    if cfg.verbose >= 1 {
        println!(
            "Dump finished in {} seconds",
            SystemTime::now()
                .duration_since(start_time)
                .unwrap()
                .as_secs_f32()
        );
    }
}
