use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

fn concatenate_files(input_dir: &str, output_file: &str) -> Result<(), Box<dyn Error>> {
    // Create or truncate the output file
    let mut output = File::create(output_file)?;

    // Read the directory
    let entries = fs::read_dir(input_dir)?;

    // Process each file in the directory
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Check if it's a file and has the correct extension
        if path.is_file() {
            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if ext == "py" || ext == "txt" {
                    // Read the file content
                    let mut content = String::new();
                    let mut file = File::open(&path)?;
                    file.read_to_string(&mut content)?;

                    // Write file name as a comment
                    writeln!(output, "\n# File: {}", path.display())?;

                    // Write the content
                    write!(output, "{}", content)?;

                    // Add a newline for separation
                    writeln!(output)?;
                }
            }
        }
    }

    Ok(())
}

// // Example usage
// fn main() -> Result<(), Box<dyn Error>> {
//     let input_directory = "path/to/your/directory";
//     let output_file = "concatenated_output.txt";

//     match concatenate_files(input_directory, output_file) {
//         Ok(()) => println!("Files successfully concatenated to {}", output_file),
//         Err(e) => eprintln!("Error: {}", e),
//     }

//     Ok(())
// }
