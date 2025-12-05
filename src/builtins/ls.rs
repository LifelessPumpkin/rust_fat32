use crate::models::ShellCore;


pub fn ls(shell: &mut ShellCore) {
    // I will first get the current directory cluster from ShellCore, 
    let mut cwd_cluster = shell.cwd_cluster as usize;
    // Then look at FAT to make sure I go through all the linked clusters,
    loop {
        // First perform a read of the cluster
        let first_sector = shell.vol.get_first_sector_of_cluster(cwd_cluster as u32);
        let bytes_per_sector = shell.vol.bpb.bpb_byts_per_sec as usize;
        let sectors_per_cluster = shell.vol.bpb.bpb_sec_per_clus as usize;
        let cluster_size = bytes_per_sector * sectors_per_cluster;
        let mut buffer = vec![0u8; cluster_size];
        // Read through the sectors of the cluster
        'cluster_loop: for sector_offset in 0..sectors_per_cluster {
            let sector_number = first_sector + sector_offset as u32;
            let offset = sector_offset * bytes_per_sector;
            shell.vol.read_sector(sector_number, &mut buffer[offset..offset + bytes_per_sector]).expect("Failed to read sector");
            // Another loop reading in 32 bytes at a time for directory entries in each sector
            for entry_offset in (0..bytes_per_sector).step_by(32) {
                let offset = sector_offset * bytes_per_sector + entry_offset;
                // Check to see if entry is valid and print name/attributes
                let first_byte = buffer[offset];

                if first_byte == 0x00 {
                    break 'cluster_loop; // end of directory
                }
                if first_byte == 0xE5 {
                    continue; // deleted, skip
                }

                let attr = buffer[offset + 11];
                if attr == 0x0F {
                    continue; // long name entry, skip for this project
                }

                let short_name = shell.vol.parse_short_name(&buffer[offset..offset + 11]);
                let is_dir = (attr & 0x10) != 0;

                if is_dir {
                    println!("[DIR]  {}", short_name);
                } else {
                    println!("[FILE] {}", short_name);
                }
            }

            
        }
        // Check for end of cluster chain
        let next = shell.vol.fat[cwd_cluster as usize] as usize;

        // stop at end of chain
        if next >= 0x0FFFFFF8 {
            break;
        }

        // IMPORTANT!!!! update cluster here
        cwd_cluster = next;
    }
}
