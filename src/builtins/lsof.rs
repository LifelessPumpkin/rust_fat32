use crate::models::{FileMode, ShellCore};


pub fn lsof(shell: &ShellCore) {
    if shell.open_files.is_empty() {
        println!("No open files.");
        return;
    }
    println!("Open Files:");
    for of in shell.open_files.iter() {
        let mode_str = match of.mode {
            FileMode::Read => "r",
            FileMode::Write => "w",
            FileMode::ReadWrite => "rw",
        };
        let dir_path = of.dir_cluster_path.clone();
        let full_path = if dir_path == "/" {
            format!("/{}", of.name)
        } else {
            format!("{}/{}", dir_path, of.name)
        };
        println!("Name: {}, Mode: {}, Offset: {}, Path: {}, FD: {}, Size: {}", of.name, mode_str, of.offset, full_path, of.file_descriptor, of.size);
    }
}
