mod file_conc;

use clap::{Arg, Command};

fn main() {
    let matches = Command::new("reposyn")
        .version("0.1.0")
        .author("Your Name <your.email@example.com>")
        .about("Creates AI-friendly text summary of a repo")
        .arg(
            Arg::new("input_dir")
                .short('f')
                .long("folder")
                .value_name("DIR")
                .default_value("./"),
        )
        .get_matches();

    let input_dir = matches.get_one::<String>("input_dir").unwrap();

    match file_conc::concatenate_files(input_dir) {
        Ok(()) => println!("Files successfully concatenated!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
