mod file_conc;

fn main() {
    let input_dir = "requests";
    let output_file = "repo-synopsis.md";

    match file_conc::concatenate_files(input_dir, output_file) {
        Ok(()) => println!("Files successfully concatenated!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
