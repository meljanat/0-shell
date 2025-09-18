use std::fs::{self, Metadata};
use std::os::unix::fs::PermissionsExt;
use chrono::Local;

pub fn ls(input: &[&str]) {
    let mut flag_l = false;
    let mut flag_a = false;
    let mut flag_f = false;
    let mut desired_paths: Vec<&str> = vec![];

    for arg in input {
        if arg.starts_with('-') {
            flag_l |= arg.contains('l');
            flag_a |= arg.contains('a');
            flag_f |= arg.contains('F');
        } else {
            desired_paths.push(arg);
        }
    }

    if desired_paths.is_empty() {
        desired_paths.push(".");
    }

    for cur_path in &desired_paths {
        if desired_paths.len() > 1 {
            println!("{}:", cur_path);
        }

        let mut entries: Vec<_> = match fs::read_dir(cur_path) {
            Ok(read) => read.filter_map(Result::ok).collect(),
            Err(e) => {
                eprintln!("ls: cannot access {}: {}", cur_path, e);
                continue;
            }
        };

        if !flag_f {
            entries.sort_by_key(|dir| dir.path());
        }

        for entry in entries {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if !flag_a && file_name_str.starts_with('.') {
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let mut name_out = file_name_str.to_string();

            if flag_f {
                let ft = metadata.file_type();
                if ft.is_dir() {
                    name_out.push('/');
                } else if ft.is_symlink() {
                    name_out.push('@');
                } else if metadata.permissions().mode() & 0o111 != 0 {
                    name_out.push('*');
                }
            }

            if flag_l {
                let perms = format_permissions(&metadata);
                let size = metadata.len();
                let modified = metadata.modified().unwrap();
                let datetime: chrono::DateTime<Local> = modified.into();

                println!(
                    "{} {:>8} {} {}",
                    perms,
                    size,
                    datetime.format("%b %d %H:%M"),
                    name_out
                );
            } else {
                print!("{}  ", name_out);
            }
        }

        if !flag_l {
            println!();
        }
    }
}

fn format_permissions(metadata: &Metadata) -> String {
    let mode = metadata.permissions().mode();
    let file_type = if metadata.is_dir() { 'd' } else { '-' };

    let mut perms = String::new();
    perms.push(file_type);

    let flags = ['r', 'w', 'x'];
    for i in (0..9).rev() {
        if (mode >> i) & 1 == 1 {
            perms.push(flags[i % 3]);
        } else {
            perms.push('-');
        }
    }
    perms
}
