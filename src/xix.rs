extern crate clap;
extern crate xir;

mod vm;

use clap::{App, Arg};

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use vm::exec::TExecutor;
use vm::loader::load;
use vm::shared_mem::SharedMem;
use vm::VMCfg;

fn main() {
    let (entry, cfg) = {
        let matches = App::new("xix")
            .version("0.4.0")
            .author("Xi")
            .about("Hello world! This is xix")
            .arg(
                Arg::with_name("entry")
                    .help("Entry module of executable")
                    .required(true)
                    .index(1),
            )
            .arg(
                Arg::with_name("ext")
                    .help("External module paths")
                    .short("i")
                    .long("import")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("diagnose")
                    .short("d")
                    .long("diagnose")
                    .help("Show diagnose info or not")
                    .takes_value(false),
            )
            .get_matches();

        let entry = matches.value_of("entry").unwrap();
        let ext_paths = matches.value_of("ext").unwrap_or("");

        let mut ext_paths = if ext_paths.len() == 0 {
            HashSet::new()
        } else {
            ext_paths
                .split(';')
                .map(|x| fs::canonicalize(x).unwrap())
                .collect()
        };

        let mut std_path = env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        std_path.push("std/");
        ext_paths.insert(std_path.canonicalize().unwrap());

        let entry = fs::canonicalize(entry).unwrap();
        let entry_root = entry.parent().unwrap().to_owned();

        (
            entry,
            VMCfg {
                entry_root,
                ext_paths: ext_paths.into_iter().collect::<Vec<PathBuf>>(),
                diagnose: matches.is_present("diagnose"),
            },
        )
    };

    let mut m = SharedMem::new();

    // loading
    let start_time = SystemTime::now();
    let (static_inits, entry) = load(entry, &mut m, &cfg);
    let mod_load_time = SystemTime::now()
        .duration_since(start_time)
        .unwrap()
        .as_secs_f32();

    // static inits
    let start_time = SystemTime::now();
    for static_init in static_inits.into_iter() {
        let mut executor = TExecutor::new(static_init);
        executor.run(&mut m);
    }
    let static_exec_time = SystemTime::now()
        .duration_since(start_time)
        .unwrap()
        .as_secs_f32();

    let start_time = SystemTime::now();
    let mut executor = TExecutor::new(entry);
    let ret = executor.run(&mut m);
    let main_exec_time = SystemTime::now()
        .duration_since(start_time)
        .unwrap()
        .as_secs_f32();
    println!("Thread exits with code {}", ret);

    if cfg.diagnose {
        println!("=============== Diagnose =================");
        println!("Module load time: {}", mod_load_time);
        println!("Static init execution time: {}", static_exec_time);
        println!("Main execution time: {}", main_exec_time);
    }
}
