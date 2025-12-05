use crate::models::{FileMode, ShellCore};


pub fn read(fd: usize, size: usize, shell: &mut ShellCore) {
    let of = if let Some(of) = shell.open_files.iter_mut().find(|of| of.file_descriptor == fd) {
        of
    } else {
        eprintln!("read: file not open: {}", fd);
        return;
    };

    match of.mode {
        FileMode::Read | FileMode::ReadWrite => {}
        _ => {
            eprintln!("read: file not opened in read mode: {}", fd);
            return;
        }
    }

    if of.offset >= of.size {
        return;
    }

    let max_readable = of.size - of.offset;
    let bytes_to_read = std::cmp::min(size as u32, max_readable) as usize;
    if bytes_to_read == 0 {
        return;
    }

    let bytes_per_sector = shell.vol.bpb.bpb_byts_per_sec as usize;
    let sectors_per_cluster = shell.vol.bpb.bpb_sec_per_clus as usize;
    let bytes_per_cluster = bytes_per_sector * sectors_per_cluster;

    let mut remaining = bytes_to_read;
    let mut file_offset = of.offset as usize;

    let mut cluster = of.start_cluster;
    let cluster_index = file_offset / bytes_per_cluster;
    let mut inner_offset = file_offset % bytes_per_cluster;

    for _ in 0..cluster_index {
        let next = shell.vol.fat[cluster as usize];
        if next >= 0x0FFFFFF8 {
            return;
        }
        cluster = next;
    }

    while remaining > 0 {
        let first_sector = shell.vol.get_first_sector_of_cluster(cluster);
        let mut cluster_buf = vec![0u8; bytes_per_cluster];

        for sec in 0..sectors_per_cluster {
            let sector_number = first_sector + sec as u32;
            let offset = sec * bytes_per_sector;
            shell
                .vol
                .read_sector(sector_number, &mut cluster_buf[offset..offset + bytes_per_sector])
                .expect("Failed to read sector");
        }

        let available_in_cluster = bytes_per_cluster - inner_offset;
        let take = remaining.min(available_in_cluster);

        let slice = &cluster_buf[inner_offset..inner_offset + take];
        for &b in slice {
            print!("{}", b as char);
        }

        remaining -= take;
        file_offset += take;

        inner_offset = 0;

        if remaining > 0 {
            let next = shell.vol.fat[cluster as usize];
            if next >= 0x0FFFFFF8 {
                break;
            }
            cluster = next;
        }
    }

    use std::io::Write;
    std::io::stdout().flush().ok();

    of.offset += bytes_to_read as u32;
}
