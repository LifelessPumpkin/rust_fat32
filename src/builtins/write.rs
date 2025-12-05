use crate::models::{FileMode, ShellCore};

pub fn write(fd: u32, data: &str, shell: &mut ShellCore) {
    // 1) Find open file
    let of = if let Some(of) = shell
        .open_files
        .iter_mut()
        .find(|of| of.file_descriptor == fd as usize)
    {
        of
    } else {
        eprintln!("write: file not open: {}", fd);
        return;
    };

    // 2) Check mode
    match of.mode {
        FileMode::Write | FileMode::ReadWrite => {}
        _ => {
            eprintln!("write: file not opened in write mode: {}", fd);
            return;
        }
    }

    let data_bytes = data.as_bytes();
    let mut remaining = data_bytes.len();
    if remaining == 0 {
        return;
    }

    let bytes_per_sector = shell.vol.bpb.bpb_byts_per_sec as usize;
    let sectors_per_cluster = shell.vol.bpb.bpb_sec_per_clus as usize;
    let bytes_per_cluster = bytes_per_sector * sectors_per_cluster;

    // 3) If no cluster yet, allocate first cluster for this file
    if of.start_cluster == 0 {
        match shell.vol.alloc_cluster() {
            Some(new_cl) => {
                of.start_cluster = new_cl;
                // NOTE: you should also update dir entry later to store this!
            }
            None => {
                eprintln!("write: failed to allocate first cluster");
                return;
            }
        }
    }

    // Logical offset from start of file
    let mut file_offset = of.offset as usize;

    // Which cluster in the chain does this offset start in?
    let mut cluster = of.start_cluster;
    let mut cluster_index = file_offset / bytes_per_cluster;
    let mut offset_in_cluster = file_offset % bytes_per_cluster;

    // Walk chain to the right cluster
    while cluster_index > 0 {
        let next = shell.vol.fat[cluster as usize];
        if next >= 0x0FFFFFF8 {
            // Need to extend chain to reach this offset
            match shell.vol.alloc_cluster() {
                Some(new_cl) => {
                    shell.vol.fat[cluster as usize] = new_cl;
                    shell.vol.fat[new_cl as usize] = 0x0FFFFFF8;
                    cluster = new_cl;
                }
                None => {
                    eprintln!("write: failed to extend cluster chain");
                    return;
                }
            }
        } else {
            cluster = next;
        }
        cluster_index -= 1;
    }

    let mut written_total = 0;

    // 4) Write loop
    while remaining > 0 {
        // Read whole cluster into memory
        let first_sector = shell.vol.get_first_sector_of_cluster(cluster);
        let mut cluster_buf = vec![0u8; bytes_per_cluster];

        for sec in 0..sectors_per_cluster {
            if let Err(e) = shell.vol.read_sector(
                first_sector + sec as u32,
                &mut cluster_buf[sec * bytes_per_sector..(sec + 1) * bytes_per_sector],
            ) {
                eprintln!("write: failed to read cluster sector: {}", e);
                return;
            }
        }

        // How many bytes can we write into this cluster from offset_in_cluster?
        let available_in_cluster = bytes_per_cluster - offset_in_cluster;
        let take = remaining.min(available_in_cluster);

        // Copy from data_bytes into cluster_buf
        let src_start = written_total;
        let src_end = written_total + take;
        let dst_start = offset_in_cluster;
        let dst_end = offset_in_cluster + take;

        cluster_buf[dst_start..dst_end].copy_from_slice(&data_bytes[src_start..src_end]);

        // Write cluster back to disk
        for sec in 0..sectors_per_cluster {
            let sector_num = first_sector + sec as u32;
            let start = sec * bytes_per_sector;
            let end = start + bytes_per_sector;
            if let Err(e) =
                shell
                    .vol
                    .write_sector(sector_num, &cluster_buf[start..end])
            {
                eprintln!("write: failed to write cluster sector: {}", e);
                return;
            }
        }

        written_total += take;
        remaining -= take;
        file_offset += take;

        // After first cluster, always start at offset 0 in following clusters
        offset_in_cluster = 0;

        if remaining > 0 {
            let next = shell.vol.fat[cluster as usize];
            if next >= 0x0FFFFFF8 {
                // Need to allocate a new cluster
                match shell.vol.alloc_cluster() {
                    Some(new_cl) => {
                        shell.vol.fat[cluster as usize] = new_cl;
                        shell.vol.fat[new_cl as usize] = 0x0FFFFFF8; // end of chain
                        cluster = new_cl;
                    }
                    None => {
                        eprintln!("write: failed to allocate new cluster");
                        return;
                    }
                }
            } else {
                cluster = next;
            }
        }
    }

    // 5) Update open file’s offset and size
    of.offset = file_offset as u32;
    if file_offset as u32 > of.size {
        of.size = file_offset as u32;
    }

    // Now update dir entry on disk
    if let Err(e) = shell.vol.update_dir_entry(
        of.dir_cluster,      // parent directory
        &of.name,            // short filename
        of.start_cluster,    // updated first cluster
        of.size,             // updated size
    ) {
        eprintln!("write: failed to update directory entry: {}", e);
    }

    // 7) Flush FAT to disk since we may have allocated clusters
    if let Err(e) = shell.vol.flush_fat() {
        eprintln!("write: failed to flush FAT to disk: {}", e);
        return;
    }
}