use std::path::Path;


pub fn io_error(path: &Path, error: &std::io::Error, hint: &str) -> ! {
    println!("❌ Error in file \"{}\"\n\n{}\n{}\n",
        path.display(), error, hint
    );
    std::process::exit(1);
}

pub fn error(message: &str) -> ! {
    println!("❌ Error: {}\n", message);
    std::process::exit(1);
}
