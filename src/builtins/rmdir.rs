use crate::models::ShellCore;

pub fn rmdir(dirname: &str, shell: &mut ShellCore) {
    if dirname.is_empty() {
        eprintln!("rmdir: missing directory name");
        return;
    }

    let parent_cluster = shell.cwd_cluster;

    let (entry_cluster, entry_offset) =
        match shell.vol.find_entry_in_directory(parent_cluster, dirname) {
            Some(v) => v,
            None => {
                eprintln!("rmdir: directory not found: {}", dirname);
                return;
            }
        };

    let entry_raw = match shell.vol.read_raw_entry(entry_cluster, entry_offset) {
        Ok(e) => e,
        Err(_) => {
            eprintln!("rmdir: failed to read directory entry '{}'", dirname);
            return;
        }
    };

    if (entry_raw[11] & 0x10) == 0 {
        eprintln!("rmdir: {} is not a directory", dirname);
        return;
    }

    if dirname == "." || dirname == ".." {
        eprintln!("rmdir: cannot remove '.' or '..'");
        return;
    }

    let hi = u16::from_le_bytes([entry_raw[20], entry_raw[21]]) as u32;
    let lo = u16::from_le_bytes([entry_raw[26], entry_raw[27]]) as u32;
    let start_cluster = (hi << 16) | lo;

    if start_cluster == 0 {
        eprintln!("rmdir: invalid directory cluster");
        return;
    }

    if !dir_is_empty(shell, start_cluster) {
        eprintln!("rmdir: directory not empty: {}", dirname);
        return;
    }

    if let Err(e) = shell.vol.dealloc_chain(start_cluster) {
        eprintln!("rmdir: failed to deallocate clusters: {}", e);
        return;
    }

    if let Err(e) = shell.vol.mark_entry_deleted(entry_cluster, entry_offset) {
        eprintln!("rmdir: failed to delete directory entry: {}", e);
        return;
    }

    if let Err(e) = shell.vol.flush_fat() {
        eprintln!("rmdir: failed to flush FAT: {}", e);
    }
}

fn dir_is_empty(shell: &mut ShellCore, start: u32) -> bool {
    let bytes_per_sector = shell.vol.bpb.bpb_byts_per_sec as usize;
    let sectors = shell.vol.bpb.bpb_sec_per_clus as usize;
    let bytes_per_cluster = bytes_per_sector * sectors;

    let mut cluster = start;

    loop {
        let first_sector = shell.vol.get_first_sector_of_cluster(cluster);

        let mut buf = vec![0u8; bytes_per_cluster];
        for s in 0..sectors {
            shell.vol.read_sector(
                first_sector + s as u32,
                &mut buf[s * bytes_per_sector..(s + 1) * bytes_per_sector],
            ).unwrap();
        }

        for off in (0..bytes_per_cluster).step_by(32) {
            let e = &buf[off..off + 32];

            let first = e[0];
            if first == 0x00 {
                return true;
            }
            if first == 0xE5 || e[11] == 0x0F {
                continue;
            }

            let name = shell.vol.parse_short_name(&e[..11]);
            if name != "." && name != ".." {
                return false;
            }
        }

        let next = shell.vol.fat[cluster as usize];
        if next >= 0x0FFFFFF8 {
            break;
        }
        cluster = next;
    }

    true
}