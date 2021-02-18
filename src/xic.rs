extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate clap;
extern crate lazy_static;
extern crate regex;

mod ir;
mod lang;

use lang::gen::module_mgr;

use clap::{App, Arg};

use std::path::PathBuf;
use std::time::SystemTime;

struct Config {
    ext_paths: Vec<String>,
    dir: PathBuf,
    out_dir: PathBuf,
    verbose: usize,
}

fn main() {
    let cfg: Config;

    {
        let matches =
            App::new("xic")
                .version("0.1.0")
                .author("Xi")
                .about("Hello world! This is xic")
                .arg(
                    Arg::with_name("root")
                        .help("Input root directory")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("output")
                        .help("Output directory. Default to be <root> if not specified")
                        .short("o")
                        .long("output")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("ext")
                        .help("External module paths")
                        .short("i")
                        .long("import")
                        .takes_value(true),
                )
                .arg(Arg::with_name("v").short("v").multiple(true).help(
                    "Level of verbosity. Level1: Display project tree; Level2: Dump .ast.json",
                ))
                .get_matches();

        let ext_paths = matches.value_of("ext").unwrap_or("");
        let input_dir = matches.value_of("root").unwrap();

        cfg = Config {
            ext_paths: ext_paths
                .split(';')
                .map(|x| x.to_owned())
                .collect::<Vec<String>>(), // TODO: 暂时没有cp
            dir: PathBuf::from(input_dir),
            out_dir: PathBuf::from(matches.value_of("output").unwrap_or(input_dir)),
            verbose: matches.occurrences_of("v") as usize,
        };
    }

    let start_time = SystemTime::now();
    let mut module_mgr = module_mgr::ModuleMgr::new(&cfg.dir, &cfg.ext_paths, cfg.verbose >= 2);
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
