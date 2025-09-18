pub fn echo(input: &[&str]) {
    println!("{}", input.join(" "));
}

pub fn pwd() {
    match std::env::current_dir() {
        Ok(path) => println!("{}", path.display()),
        Err(e) => eprintln!("Error getting current directory: {}", e),
    }
}

pub fn cat(input: &[&str]) {
    if input.is_empty() {
        return;
    }

    for &file in input {
        match std::fs::read_to_string(file) {
            Ok(content) => print!("{}", content),
            Err(e) => eprintln!("cat: {}: {}", file, e),
        }
    }
}
