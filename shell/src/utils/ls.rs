use chrono::Local;
use std::fs::{self, Metadata};
use std::os::unix::fs::{MetadataExt, PermissionsExt};

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

        let mut entries: Vec<std::path::PathBuf> = match fs::read_dir(cur_path) {
            Ok(read) => read.filter_map(|e| e.ok().map(|d| d.path())).collect(),
            Err(e) => {
                eprintln!("ls: cannot access {}: {}", cur_path, e);
                continue;
            }
        };

        if flag_a {
            entries.push(std::path::Path::new(cur_path).join("."));
            entries.push(std::path::Path::new(cur_path).join(".."));
        }

        if flag_l {
            let mut total: u64 = 0;
            for entry in entries.iter() {
                let meta = entry.metadata().unwrap();
                total += meta.blocks();
            }
            println!("total {}", total / 2);
        }

        if !flag_f {
            entries.sort();
        }

        println!("entries {:?}", entries);
        for entry in entries {
            let file_name = entry.file_name().unwrap_or_default();
            let file_name_str = file_name.to_string_lossy();

            if !flag_a && file_name_str.starts_with('.') {
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let mut name_out = file_name_str.to_string();
            let username = users::get_user_by_uid(metadata.uid())
                .and_then(|u| u.name().to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| metadata.uid().to_string());
            let groupname = users::get_group_by_gid(metadata.gid())
                .and_then(|g| g.name().to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| metadata.gid().to_string());
            let nlink = metadata.nlink();

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
                    "{} {} {} {} {} {} {}",
                    perms,
                    nlink,
                    username,
                    groupname,
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
