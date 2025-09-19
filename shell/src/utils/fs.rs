pub fn cp(input: &[&str]) {
    if input.is_empty() {
        eprintln!("cp: missing file operand");
        return;
    }
    if input.len() < 2 {
        eprintln!("cp: missing destination file operand after '{}'", input[0]);
        return;
    }

    let dest = input.last().unwrap();
    let sources = &input[..input.len() - 1];

    for &src in sources {
        let result = std::fs::copy(src, dest);
        if let Err(e) = result {
            eprintln!("cp: cannot copy '{}': {}", src, e);
        }
    }
}

pub fn rm(input: &[&str]) {
    if input.is_empty() {
        eprintln!("rm: missing operand");
        return;
    }

    let flag_r = input[0].to_lowercase() == "-r" || input[0] == "-rf" || input[0] == "-fr";
    let paths: Vec<&str> = if flag_r {
        input[1..].to_vec()
    } else {
        input.to_vec()
    };

    for path in paths {
        let result = if flag_r {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        };

        if let Err(e) = result {
            eprintln!("rm: cannot remove '{}': {}", path, e);
        }
    }
}

pub fn mv(input: &[&str]) {
    if input.is_empty() {
        eprintln!("mv: missing file operand");
        return;
    }
    if input.len() < 2 {
        eprintln!("mv: missing destination file operand after '{}'", input[0]);
        return;
    }

    let dest = input.last().unwrap();
    let sources = &input[..input.len() - 1];

    for &src in sources {
        let result = std::fs::rename(src, dest);
        if let Err(e) = result {
            eprintln!("mv: cannot move '{}': {}", src, e);
        }
    }
}
