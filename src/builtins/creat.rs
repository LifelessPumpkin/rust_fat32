use crate::models::ShellCore;

pub fn creat(filename: &str, shell: &mut ShellCore) {
    if filename.is_empty() {
        eprintln!("creat: missing filename :(");
        return;
    }

    let parent_cluster = shell.cwd_cluster;

    // 1) Check if an entry with this name already exists
    if shell
        .vol
        .find_entry_in_directory(parent_cluster, filename)
        .is_some()
    {
        eprintln!("creat: file already exists: {}", filename);
        return;
    }

    let bytes_per_sector = shell.vol.bpb.bpb_byts_per_sec as usize;
    let sectors_per_cluster = shell.vol.bpb.bpb_sec_per_clus as usize;

    // 2) Find a free directory entry in parent dir
    let (entry_cluster, entry_offset_in_cluster) =
        match shell.vol.find_free_directory_entry(parent_cluster) {
            Some(v) => v,
            None => {
                // No free entry → extend parent directory
                match shell.vol.append_cluster(parent_cluster) {
                    Some(new_cl) => {
                        // New cluster is empty, first entry at offset 0
                        (new_cl, 0)
                    }
                    None => {
                        eprintln!("creat: no free clusters available for parent directory :(");
                        return;
                    }
                }
            }
        };

    // 3) For a 0-byte file, we can set first_cluster = 0
    let first_cluster_for_file: u32 = 0;
    let file_size: u32 = 0;

    // 4) Write file entry into parent directory
    {
        let sector_index_in_cluster = entry_offset_in_cluster / bytes_per_sector;
        let entry_offset_in_sector = entry_offset_in_cluster % bytes_per_sector;

        if sector_index_in_cluster >= sectors_per_cluster {
            eprintln!("creat: internal error: entry offset outside cluster :(");
            return;
        }

        let first_sector = shell.vol.get_first_sector_of_cluster(entry_cluster);
        let sector_number = first_sector + sector_index_in_cluster as u32;

        // Read that sector
        let mut sector_buf = vec![0u8; bytes_per_sector];
        if let Err(e) = shell.vol.read_sector(sector_number, &mut sector_buf) {
            eprintln!("creat: failed to read parent directory sector: {e}");
            return;
        }

        if entry_offset_in_sector + 32 > bytes_per_sector {
            eprintln!("creat: internal error: entry crosses sector boundary :(");
            return;
        }

        let entry_slice =
            &mut sector_buf[entry_offset_in_sector..entry_offset_in_sector + 32];

        // attr 0x20 = archive/regular file
        shell.vol.write_directory_entry(
            entry_slice,
            filename,
            0x20,
            first_cluster_for_file,
            file_size,
        );

        if let Err(e) = shell.vol.write_sector(sector_number, &sector_buf) {
            eprintln!("creat: failed to write parent directory sector: {e}");
            return;
        }
    }

    if let Err(e) = shell.vol.flush_fat() {
        eprintln!("creat: failed to flush FAT to disk: {e}");
        return;
    }

    // Optional debug:
    // println!("Created file '{}'", filename);
}