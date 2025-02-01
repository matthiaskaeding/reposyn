mod file_conc;
use clap::{Arg, Command};

fn main() {
    let matches = Command::new("reposyn")
        .version("0.1.0")
        .author("Matthias Kaeding <kaedingmatthias@gmail.com>")
        .about("Creates AI-friendly text summary of a repo")
        .arg(
            Arg::new("input_folder")
                .short('f')
                .long("folder")
                .value_name("DIR")
                .default_value("./"),
        )
        .arg(
            Arg::new("ignore")
                .short('i')
                .long("ignore")
                .value_name("PATTERNS")
                .help("Comma-separated paths to ignore (e.g., 'target,node_modules')")
                .default_value("target,node_modules,.git"),
        )
        .arg(
            Arg::new("target")
                .short('t')
                .long("target_output")
                .value_name("TARGET")
                .help("Target file")
                .default_value("repo-synopsis.txt"),
        )
        .get_matches();

    let input_dir = matches.get_one::<String>("input_folder").unwrap();
    let ignore_patterns: Vec<&str> = matches
        .get_one::<String>("ignore")
        .unwrap()
        .split(",")
        .collect::<Vec<&str>>();
    let target = matches.get_one::<String>("target").unwrap();

    match file_conc::concatenate_files(input_dir, ignore_patterns, target) {
        Ok(()) => println!("Files successfully concatenated!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
