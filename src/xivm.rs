extern crate clap;

use clap::{App, Arg};

fn main() {
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
                Arg::with_name("cp")
                    .help("Additional class path")
                    .short("cp")
                    .long("classpath")
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

        // Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
        // required we could have used an 'if let' to conditionally get the value)
        println!("Entry: {}", matches.value_of("entry").unwrap());

        let class_path = matches.value_of("cp").unwrap_or("");
        println!("Class path: {}", class_path);

        if matches.is_present("diagnose") {
            println!("Use diagnose.");
        } else {
            println!("No diagnose.");
        }
    }
}
