mod file_conc;
mod glob;
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
                .help("Folder to summarize")
                .default_value("./"),
        )
        .arg(
            Arg::new("ignore")
                .short('i')
                .long("ignore")
                .value_name("PATTERNS")
                .help("Comma-separated paths to ignore (e.g., 'target,node_modules')")
                .default_value(".git"),
        )
        .arg(
            Arg::new("summarize")
                .short('s')
                .long("summarize")
                .value_name("SUMMARIZE")
                .help("Comma-separated paths to summarise (e.g., '*.json' for all json files)")
                .default_value(""),
        )
        .arg(
            Arg::new("output_file")
                .short('o')
                .long("output_file")
                .value_name("output_file")
                .help("Output text file")
                .default_value("repo-synopsis.txt"),
        )
        .arg(
            Arg::new("clipboard")
                .short('c')
                .long("clipboard")
                .help("Copy to clipboard, will ignore <output_file> if set")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let input_dir = matches.get_one::<String>("input_folder").unwrap();
    let ignore = matches.get_one::<String>("ignore").unwrap();
    let summarize = matches.get_one::<String>("summarize").unwrap();
    let target = matches.get_one::<String>("output_file").unwrap();
    let use_clipboard = matches.get_flag("clipboard");

    match file_conc::concatenate_files(input_dir, ignore, target, use_clipboard, summarize) {
        Ok(()) => (),
        Err(e) => eprintln!("Error: {}", e),
    }
}
