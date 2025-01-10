mod file_conc; // This tells Rust to look for file_conc.rs

fn main() {
    let input_dir = "requests/src/requests";
    let output_file = "repo-synopsis.txt";

    match file_conc::concatenate_files(input_dir, output_file) {
        Ok(()) => println!("Files successfully concatenated!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
