extern crate clap;

use clap::{App, Arg};

use std::path::PathBuf;

struct Config {
    module_dir: PathBuf,
    ext_paths: Vec<String>,
    diagnose: bool,
}

fn main() {
    let cfg: Config;
    {
        let matches = App::new("xivm")
            .version("0.1.0")
            .author("Xi")
            .about("Hello world! This is xivm")
            .arg(
                Arg::with_name("entry")
                    .help("Entry of executable")
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
                    .help("Run diagnose or not")
                    .takes_value(false),
            )
            .get_matches();

        let entry = matches.value_of("entry").unwrap();
        let ext_paths = matches.value_of("ext").unwrap_or("");

        cfg = Config {
            module_dir: PathBuf::from(entry),
            ext_paths: ext_paths
                .split(';')
                .map(|x| x.to_owned())
                .collect::<Vec<String>>(), // TODO: 暂时没有cp
            diagnose: matches.is_present("diagnose"),
        };
    }

    println!("Module path: {}", cfg.module_dir.to_str().unwrap());
    println!("External module paths:");
    for cp in cfg.ext_paths.iter() {
        println!("    {}", cp);
    }
    println!(
        "Use diagnose: {}",
        if cfg.diagnose { "true" } else { "false" }
    );
}
