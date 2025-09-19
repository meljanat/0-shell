use chrono::{DateTime, Local};
use std::ffi::OsString;
use std::fs::{self, DirEntry, Metadata};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use users::{get_group_by_gid, get_user_by_uid};

fn format_permissions(metadata: &Metadata) -> String {
    let mode = metadata.permissions().mode();
    let file_type = if metadata.is_dir() {
        'd'
    } else if metadata.file_type().is_symlink() {
        'l'
    } else {
        '-'
    };

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

fn format_time(mtime: SystemTime) -> String {
    let datetime: DateTime<Local> = DateTime::from(mtime);
    datetime.format("%b %e %H:%M").to_string()
}

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

        let entries: Vec<DirEntry> = match fs::read_dir(cur_path) {
            Ok(read) => read.filter_map(Result::ok).collect(),
            Err(e) => {
                eprintln!("ls: cannot access {}: {}", cur_path, e);
                continue;
            }
        };

         let mut visible_entries: Vec<(PathBuf, OsString)> = vec![];

        if flag_a {
            visible_entries.push((PathBuf::from("."), OsString::from(".")));
            visible_entries.push((PathBuf::from(".."), OsString::from("..")));
        }

        for entry in entries {
            let name = entry.file_name();
            if !flag_a && name.to_string_lossy().starts_with('.') {
                continue;
            }
            visible_entries.push((entry.path(), name));
        }

        if !flag_f {
            visible_entries.sort_by_key(|(_, name)| name.clone());
        }

        if flag_l {
            let total_blocks: u64 = visible_entries
                .iter()
                .filter_map(|(path, _)| fs::symlink_metadata(path).ok())
                .map(|meta| meta.blocks())
                .sum();
            println!("total {}", total_blocks / 2);
        }

        for (path, name) in visible_entries {
            match fs::symlink_metadata(&path) {
                Ok(metadata) => {
                    if flag_l {
                        print_long_entry(&metadata, name, flag_f);
                    } else {
                        let mut name_out = name.to_string_lossy().to_string();
                        if flag_f {
                            if metadata.is_dir() {
                                name_out.push('/');
                            } else if metadata.file_type().is_symlink() {
                                name_out.push('@');
                            } else if metadata.permissions().mode() & 0o111 != 0 {
                                name_out.push('*');
                            }
                        }
                        print!("{}  ", name_out);
                    }
                }
                Err(_) => continue,
            }
        }

        if !flag_l {
            println!();
        }
    }
}

fn print_long_entry(metadata: &Metadata, name: OsString, flag_f: bool) {
    let perms = format_permissions(metadata);
    let nlink = metadata.nlink();
    let user = get_user_by_uid(metadata.uid())
        .and_then(|u| u.name().to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| metadata.uid().to_string());
    let group = get_group_by_gid(metadata.gid())
        .and_then(|g| g.name().to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| metadata.gid().to_string());
    let size = metadata.len();
    let mtime = metadata.modified().unwrap_or(UNIX_EPOCH);
    let mtime_str = format_time(mtime);

    let mut name_out = name.to_string_lossy().to_string();
    if flag_f {
        if metadata.is_dir() {
            name_out.push('/');
        } else if metadata.file_type().is_symlink() {
            name_out.push('@');
        } else if metadata.permissions().mode() & 0o111 != 0 {
            name_out.push('*');
        }
    }

    println!(
        "{} {} {} {} {} {} {}",
        perms, nlink, user, group, size, mtime_str, name_out
    );
}
