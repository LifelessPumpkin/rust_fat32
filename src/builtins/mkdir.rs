use crate::models::ShellCore;

pub fn mkdir(dirname: &str, shell: &mut ShellCore) {
    if dirname.is_empty() {
        eprintln!("mkdir: missing directory name :(");
        return;
    }

    let parent_cluster = shell.cwd_cluster;

    if shell
        .vol
        .find_entry_in_directory(parent_cluster, dirname)
        .is_some()
    {
        eprintln!("mkdir: directory already exists: {}", dirname);
        return;
    }

    let bytes_per_sector = shell.vol.bpb.bpb_byts_per_sec as usize;
    let sectors_per_cluster = shell.vol.bpb.bpb_sec_per_clus as usize;
    let _bytes_per_cluster = bytes_per_sector * sectors_per_cluster;

    let (entry_cluster, entry_offset_in_cluster) =
        match shell.vol.find_free_directory_entry(parent_cluster) {
            Some(v) => v,
            None => {
                match shell.vol.append_cluster(parent_cluster) {
                    Some(new_cl) => {
                        (new_cl, 0)
                    }
                    None => {
                        eprintln!("mkdir: no free clusters available for parent directory :(");
                        return;
                    }
                }
            }
        };

    let new_dir_cluster = match shell.vol.alloc_cluster() {
        Some(c) => c,
        None => {
            eprintln!("mkdir: no free clusters available for new directory :(");
            return;
        }
    };

    {
        let sector_index_in_cluster = entry_offset_in_cluster / bytes_per_sector;
        let entry_offset_in_sector = entry_offset_in_cluster % bytes_per_sector;

        if sector_index_in_cluster >= sectors_per_cluster {
            eprintln!("mkdir: internal error: entry offset outside cluster :(");
            return;
        }

        let first_sector = shell.vol.get_first_sector_of_cluster(entry_cluster);
        let sector_number = first_sector + sector_index_in_cluster as u32;

        let mut sector_buf = vec![0u8; bytes_per_sector];
        if let Err(e) = shell.vol.read_sector(sector_number, &mut sector_buf) {
            eprintln!("mkdir: failed to read parent directory sector: {e}");
            return;
        }

        if entry_offset_in_sector + 32 > bytes_per_sector {
            eprintln!("mkdir: internal error: entry crosses sector boundary :(");
            return;
        }

        let entry_slice =
            &mut sector_buf[entry_offset_in_sector..entry_offset_in_sector + 32];

        shell
            .vol
            .write_directory_entry(entry_slice, dirname, 0x10, new_dir_cluster, 0);

        if let Err(e) = shell.vol.write_sector(sector_number, &sector_buf) {
            eprintln!("mkdir: failed to write parent directory sector: {e}");
            return;
        }
    }

    shell
        .vol
        .initialize_directory_cluster(new_dir_cluster, parent_cluster);

    if let Err(e) = shell.vol.flush_fat() {
        eprintln!("mkdir: failed to flush FAT to disk: {e}");
        return;
    }
}