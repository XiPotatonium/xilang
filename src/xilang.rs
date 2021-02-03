extern crate clap;
extern crate lazy_static;
extern crate regex;

mod ir;
mod lang;

use lang::crate_mgr;

use clap::{App, Arg};

use std::path::PathBuf;

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
        println!("Additional class-path: {}", cfg.class_path.join(";"));
        println!("Crate root directory: {:?}", cfg.dir);
        println!("Output directory: {:?}", cfg.out_dir);
        println!("Verbose level: {}", cfg.verbose);
    }

    let crate_mgr = crate_mgr::CrateMgr::new(&cfg.dir, cfg.verbose >= 2);

    if cfg.verbose >= 1 {
        crate_mgr.tree();
    }

    crate_mgr.build();
    crate_mgr.dump(&cfg.out_dir);
}
