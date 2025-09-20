use chrono::{DateTime, Local};
use std::ffi::OsString;
use std::fs::File;
use std::fs::{self, DirEntry, Metadata, read_link};
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use users::{get_group_by_gid, get_user_by_uid};
use xattr::FileExt;

fn has_extended_acl(path: &PathBuf) -> bool {
    if let Ok(file) = File::open(path) {
        if let Ok(attrs) = file.list_xattr() {
            return attrs.into_iter().any(|a| a == "system.posix_acl_access");
        }
    }
    false
}

fn format_permissions(metadata: &Metadata, path: &PathBuf) -> String {
    let mode = metadata.permissions().mode();
    println!("{}", mode);
    let file_type = if metadata.is_dir() {
        'd'
    } else if metadata.file_type().is_symlink() {
        'l'
    } else if metadata.file_type().is_fifo() {
        'p'
    } else if metadata.file_type().is_socket() {
        's'
    } else if metadata.file_type().is_block_device() {
        'b'
    } else if metadata.file_type().is_char_device() {
        'c'
    } else {
        '-'
    };

    let mut perms = String::new();
    perms.push(file_type);

    let flags = ['x', 'w', 'r'];
    for i in (0..9).rev() {
        if (mode >> i) & 1 == 1 {
            perms.push(flags[i % 3]);
        } else {
            perms.push('-');
        }
    }
    if has_extended_acl(path) {
        perms.push('+');
    }
    perms
}

fn format_time(mtime: SystemTime) -> String {
    let datetime: DateTime<Local> = DateTime::from(mtime);
    datetime.format("%b %e %H:%M").to_string()
}

fn ls_suffix(metadata: &Metadata, path: &PathBuf) -> char {
    let target_metadata = if metadata.file_type().is_symlink() {
        fs::metadata(path).ok()
    } else {
        None
    };

    let meta = target_metadata.as_ref().unwrap_or(metadata);

    let ft;
    ft = meta.file_type();
   
    if ft.is_dir() {
        '/'
    } else if ft.is_symlink() {
        '@'
    } else if ft.is_fifo() {
        '|'
    } else if ft.is_socket() {
        '='
    } else if meta.permissions().mode() & 0o111 != 0 {
        '*'
    } else {
        '\0'
    }
}

struct LsEntry {
    perms: String,
    nlink: usize,
    user: String,
    group: String,
    size: u64,
    mtime_str: String,
    name: String,
    symlink_target: Option<String>,
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

    for cur_path in desired_paths.iter() {

        let entries: Vec<DirEntry> = match fs::read_dir(cur_path) {
            Ok(read) => read.filter_map(Result::ok).collect(),
            Err(e) => {
                eprintln!("ls: cannot access {}: {}", cur_path, e);
                continue;
            }
        };

        let mut visible_entries: Vec<(PathBuf, OsString)> = vec![];

        if flag_a {
            visible_entries.push((PathBuf::from(cur_path).join("."), OsString::from(".")));
            visible_entries.push((PathBuf::from(cur_path).join(".."), OsString::from("..")));
        }

        for entry in entries {
            let name = entry.file_name();
            if !flag_a && name.to_string_lossy().starts_with('.') {
                continue;
            }
            visible_entries.push((entry.path(), name));
        }

        visible_entries.sort();

        if flag_l {
            let total_blocks: u64 = visible_entries
                .iter()
                .filter_map(|(path, _)| fs::symlink_metadata(path).ok())
                .map(|meta| meta.blocks())
                .sum();
            println!("total {}", total_blocks / 2);

            let mut entries_info = Vec::new();

            for (path, name) in &visible_entries {
                match fs::symlink_metadata(path) {
                    Ok(metadata) => {
                        let perms = format_permissions(&metadata, path);
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
                        let mut symlink_target = None;
                        if metadata.file_type().is_symlink() {
                            if let Ok(target) = read_link(path) {
                                symlink_target = Some(target.to_string_lossy().to_string());
                            }
                        }
                        if flag_f {
                            let suffix = ls_suffix(&metadata, path);
                            if suffix != '\0' {
                                name_out.push(suffix);
                            }
                        }
                        if name_out.chars().any(|c| c.is_whitespace()) {
                            name_out = format!("'{}'", name_out);
                        }

                        entries_info.push(LsEntry {
                            perms,
                            nlink: nlink.try_into().unwrap(),
                            user,
                            group,
                            size,
                            mtime_str,
                            name: name_out,
                            symlink_target,
                        });
                    }
                    Err(e) => {
                        eprintln!("ls: cannot access {}: {}", name.to_string_lossy(), e);
                    }
                }
            }

            let perms_width = entries_info
                .iter()
                .map(|e| e.perms.len())
                .max()
                .unwrap_or(0);
            let nlink_width = entries_info
                .iter()
                .map(|e| e.nlink.to_string().len())
                .max()
                .unwrap_or(0);
            let user_width = entries_info.iter().map(|e| e.user.len()).max().unwrap_or(0);
            let group_width = entries_info
                .iter()
                .map(|e| e.group.len())
                .max()
                .unwrap_or(0);
            let size_width = entries_info
                .iter()
                .map(|e| e.size.to_string().len())
                .max()
                .unwrap_or(0);

            for e in entries_info {
                if let Some(target) = e.symlink_target {
                    let target_path = PathBuf::from(&target);
                    let target_meta = fs::metadata(&target_path).ok();
                    let suffix = target_meta
                        .as_ref()
                        .map(|m| ls_suffix(m, &target_path))
                        .filter(|&c| c != '\0')
                        .unwrap_or('\0');
                    if suffix != '\0' {
                        println!(
                            "{:<perms_width$} {:>nlink_width$} {:<user_width$} {:<group_width$} {:>size_width$} {} {} -> {}{}",
                            e.perms,
                            e.nlink,
                            e.user,
                            e.group,
                            e.size,
                            e.mtime_str,
                            e.name,
                            target,
                            suffix,
                        );
                    } else {
                        println!(
                            "{:<perms_width$} {:>nlink_width$} {:<user_width$} {:<group_width$} {:>size_width$} {} {} -> {}",
                            e.perms, e.nlink, e.user, e.group, e.size, e.mtime_str, e.name, target,
                        );
                    }
                } else {
                    println!(
                        "{:<perms_width$} {:>nlink_width$} {:<user_width$} {:<group_width$} {:>size_width$} {} {}",
                        e.perms, e.nlink, e.user, e.group, e.size, e.mtime_str, e.name,
                    );
                }
            }
        } else {
            for (path, name) in &visible_entries {
                match fs::symlink_metadata(path) {
                    Ok(metadata) => {
                        let mut name_out = name.to_string_lossy().to_string();
                        if flag_f {
                            let suffix = ls_suffix(&metadata, path);
                            if suffix != '\0' {
                                name_out.push(suffix);
                            }
                        }
                        print!("{:>10}  ", name_out);
                    }
                    Err(e) => {
                        eprint!("ls: cannot access {}: {}  ", name.to_string_lossy(), e);
                    }
                }
            }
            println!();
        }
    }
}
