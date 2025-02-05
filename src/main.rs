mod file_conc;

fn main() {
    match file_conc::concatenate_files("./", "", "repo-synopsis.txt", true) {
        Ok(()) => (),
        Err(e) => eprintln!("Error: {}", e),
    }
}
