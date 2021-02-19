extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate clap;
extern crate lazy_static;
extern crate regex;

mod ir;
mod lang;

use lang::gen::xi_crate::Crate;
use lang::XicCfg;

use clap::{App, Arg};
use lazy_static::lazy_static;

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

lazy_static! {
    // Same as identifier
    static ref NAME_RULE : regex::Regex = regex::Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*.xi").unwrap();
}

fn main() {
    let cfg: XicCfg;

    {
        let matches =
            App::new("xic")
                .version("0.1.0")
                .author("Xi")
                .about("Hello world! This is xic")
                .arg(
                    Arg::with_name("root")
                        .help("Root path")
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
                .arg(
                    Arg::with_name("optim")
                        .help("Optimization level: 0 | 1")
                        .short("O")
                        .takes_value(true),
                )
                .arg(Arg::with_name("v").short("v").multiple(true).help(
                    "Level of verbosity. Level1: Display project tree; Level2: Dump .ast.json",
                ))
                .get_matches();

        let ext_paths = matches.value_of("ext").unwrap_or("");
        let root_path = matches.value_of("root").unwrap();
        let output_dir = matches.value_of("output");
        let root_path =
            fs::canonicalize(root_path).expect(&format!("Fail to canonicalize {}", root_path));
        if !root_path.is_file() {
            panic!("Root path {} is not a file", root_path.to_str().unwrap());
        }
        let root_dir = root_path.parent().unwrap().to_owned();
        let crate_name = root_dir.file_name().unwrap().to_str().unwrap().to_owned();
        let root_file_name = root_path.file_name().unwrap().to_str().unwrap().to_owned();

        if !NAME_RULE.is_match(&root_file_name) {
            panic!("Invalid root file name {}", root_file_name);
        }

        cfg = XicCfg {
            ext_paths: ext_paths
                .split(';')
                .map(|x| x.to_owned())
                .collect::<Vec<String>>(),
            crate_name,
            root_path,
            out_dir: if let Some(output_dir) = output_dir {
                PathBuf::from(output_dir)
            } else {
                root_dir
            },
            optim: matches.value_of("optim").unwrap().parse::<usize>().unwrap(),
            verbose: matches.occurrences_of("v") as usize,
        };
    }

    let start_time = SystemTime::now();
    let mut module_mgr = Crate::new(&cfg);
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
