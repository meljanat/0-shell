pub fn cd(input: &[&str]) {
    let target_dir = if input.is_empty() {
        std::env::var("HOME").unwrap_or_else(|_| String::from("/"))
    } else {
        input[0].to_string()
    };

    if let Err(e) = std::env::set_current_dir(&target_dir) {
        eprintln!("cd: {}: {}", target_dir, e);
    }
}

pub fn mkdir(input: &[&str]) {
    for &dir in input {
        if let Err(e) = std::fs::create_dir(dir) {
            eprintln!("mkdir: cannot create directory ‘{}’: {}", dir, e);
        }
    }
}
