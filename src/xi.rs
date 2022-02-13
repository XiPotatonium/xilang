extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate clap;
extern crate regex;

extern crate core;

mod lang;
mod runtime;

use lang::build::prepare_crate_for_build;
use lang::XiCfg;

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
    let cfg = {
        let matches = App::new("xi")
            .version("0.1.0")
            .author("shwu")
            .about("Hello world! This is xi")
            .arg(Arg::new("root").help("Root path").required(true))
            .arg(
                Arg::new("output")
                    .help("Output directory. Default to be <root> if not specified")
                    .short('o')
                    .takes_value(true),
            )
            .arg(
                Arg::new("compile")
                    .short('c')
                    .long("compile")
                    .help("Only generate byte code."),
            )
            .arg(Arg::new("ast").long("ast").help("Dump .ast.json"))
            .get_matches();

        let root_path = matches.value_of("root").unwrap();
        let output_dir = matches.value_of("output");
        let root_path = fs::canonicalize(root_path).unwrap();
        if !root_path.is_file() {
            panic!("Root path {} is not a file", root_path.display());
        }
        let root_dir = root_path.parent().unwrap().to_owned();
        let crate_name = root_dir.file_name().unwrap().to_str().unwrap().to_owned();
        let root_fname = root_path.file_name().unwrap().to_str().unwrap().to_owned();

        if !NAME_RULE.is_match(&root_fname) {
            panic!("Invalid root file name {}", root_fname);
        }

        XiCfg {
            root_dir: root_dir.clone(),
            crate_name,
            root_path,
            out_dir: if let Some(output_dir) = output_dir {
                PathBuf::from(output_dir)
            } else {
                root_dir
            },
            dump_ast: matches.is_present("ast"),
            compile: matches.is_present("compile"),
        }
    };

    let start_time = SystemTime::now();
    let mut krate_builder = prepare_crate_for_build(&cfg);
    println!(
        "Parsing finished in {} seconds",
        SystemTime::now()
            .duration_since(start_time)
            .unwrap()
            .as_secs_f32()
    );

    if cfg.compile {
        let start_time = SystemTime::now();
        krate_builder.build(&cfg);
        println!(
            "Build finished in {} seconds",
            SystemTime::now()
                .duration_since(start_time)
                .unwrap()
                .as_secs_f32()
        );

        let start_time = SystemTime::now();
        krate_builder.dump(&cfg);
        println!(
            "Dump finished in {} seconds",
            SystemTime::now()
                .duration_since(start_time)
                .unwrap()
                .as_secs_f32()
        );
    }
}
