use crate::models::ShellCore;

pub fn rm(filename: &str, shell: &mut ShellCore) {
    if filename.is_empty() {
        eprintln!("rm: missing file name :(");
        return;
    }

    let parent_cluster = shell.cwd_cluster;

    let (entry_cluster, entry_offset) =
        match shell.vol.find_entry_in_directory(parent_cluster, filename) {
            Some((cl, off)) => (cl, off),
            None => {
                eprintln!("rm: file not found: {}", filename);
                return;
            }
        };

    let raw = match shell.vol.read_raw_entry(entry_cluster, entry_offset) {
        Ok(r) => r,
        Err(_) => {
            eprintln!("rm: failed to read directory entry: {}", filename);
            return;
        }
    };

    let attr = raw[11];
    if (attr & 0x10) != 0 {
        eprintln!("rm: {} is a directory", filename);
        return;
    }

    for of in shell.open_files.iter() {
        if of.name.eq_ignore_ascii_case(filename) && of.dir_cluster == parent_cluster {
            eprintln!("rm: cannot remove open file: {}", filename);
            return;
        }
    }

    let hi = u16::from_le_bytes([raw[20], raw[21]]) as u32;
    let lo = u16::from_le_bytes([raw[26], raw[27]]) as u32;
    let starting_cluster = (hi << 16) | lo;

    if starting_cluster != 0 {
        if let Err(e) = shell.vol.dealloc_chain(starting_cluster) {
            eprintln!("rm: failed to deallocate clusters: {}", e);
            return;
        }
    }

    if let Err(e) = shell.vol.mark_entry_deleted(entry_cluster, entry_offset) {
        eprintln!("rm: failed to mark directory entry deleted: {}", e);
        return;
    }

    if let Err(e) = shell.vol.flush_fat() {
        eprintln!("rm: failed to flush FAT: {}", e);
    }
}