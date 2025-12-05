use crate::models::ShellCore;


pub fn cd(target_dir: &str, shell: &mut ShellCore) {
    if target_dir == "" {
        eprintln!("cd: missing operand");
        return;
    }
    if target_dir == "." {
        return;
    }

    // 2. SPECIAL CASE: ".."
    if target_dir == ".." {
        let root = shell.vol.bpb.bpb_root_clus;
        let mut cluster = shell.cwd_cluster;
        // ROOT special case
        if cluster == root {
            return; // already at root
        }

        // Read current directory clusters and find ".." entry to get parent cluster
        loop {
            let first_sector = shell.vol.get_first_sector_of_cluster(cluster);
            let bytes_per_sector = shell.vol.bpb.bpb_byts_per_sec as usize;
            let sectors_per_cluster = shell.vol.bpb.bpb_sec_per_clus as usize;
            let cluster_size = bytes_per_sector * sectors_per_cluster;
            let mut buffer = vec![0u8; cluster_size];

            // Read cluster
            for sec in 0..sectors_per_cluster {
                let sector = first_sector + sec as u32;
                let offset = sec * bytes_per_sector;
                shell.vol.read_sector(sector, &mut buffer[offset..offset + bytes_per_sector]).unwrap();
            }

            // scan entries for ".."
            for offset in (0..cluster_size).step_by(32) {
                let entry = &buffer[offset..offset + 32];
                if entry[0] == 0x00 { break; }
                if entry[0] == 0xE5 { continue; }
                if entry[11] == 0x0F { continue; }

                let name = shell.vol.parse_short_name(&entry[0..11]);
                if name == ".." {
                    let hi = u16::from_le_bytes([entry[20], entry[21]]) as u32;
                    let lo = u16::from_le_bytes([entry[26], entry[27]]) as u32;
                    let parent_cluster = (hi << 16) | lo;

                    // parent_cluster == 0 means root in some implementations
                    shell.cwd_cluster = if parent_cluster == 0 { root } else { parent_cluster };

                    // Update cwd_path: remove last path component
                    if shell.cwd_path != "/" {
                        if let Some(pos) = shell.cwd_path.rfind('/') {
                            if pos == 0 {
                                shell.cwd_path = "/".to_string();
                            } else {
                                shell.cwd_path.truncate(pos);
                            }
                        }
                    }
                    return;
                }
            }

            // follow FAT chain of the current directory if ".." wasn't in this cluster
            let next = shell.vol.fat[cluster as usize];
            if next >= 0x0FFFFFF8 {
                break;
            }
            cluster = next;
        }
        return;
    }
    
    // I should first check if the directory already exists in the current directory
    let mut cwd_cluster = shell.cwd_cluster as usize;
    'cluster_loop: loop {
        // First perform a read of the cluster
        let first_sector = shell.vol.get_first_sector_of_cluster(cwd_cluster as u32);
        let bytes_per_sector = shell.vol.bpb.bpb_byts_per_sec as usize;
        let sectors_per_cluster = shell.vol.bpb.bpb_sec_per_clus as usize;
        let cluster_size = bytes_per_sector * sectors_per_cluster;
        let mut buffer = vec![0u8; cluster_size];
        // Read through the sectors of the cluster
        'sector_loop: for sector_offset in 0..sectors_per_cluster {
            let sector_number = first_sector + sector_offset as u32;
            let offset = sector_offset * bytes_per_sector;
            shell.vol.read_sector(sector_number, &mut buffer[offset..offset + bytes_per_sector]).expect("Failed to read sector");
            // Another loop reading in 32 bytes at a time for directory entries in each sector
            for entry_offset in (0..bytes_per_sector).step_by(32) {
                let offset = sector_offset * bytes_per_sector + entry_offset;
                // Check to see if entry is valid and print name/attributes
                let first_byte = buffer[offset];

                if first_byte == 0x00 {
                    break 'sector_loop; // end of directory
                }
                if first_byte == 0xE5 {
                    continue; // deleted, skip
                }

                let attr = buffer[offset + 11];
                if attr == 0x0F {
                    continue; // long name entry, skip for this project
                }

                // Check the short name and if it is the same as dirname, print error and return
                let short_name = shell.vol.parse_short_name(&buffer[offset..offset + 11]);
                // If it matches then it is a valid directory and we can continue
                if short_name.eq_ignore_ascii_case(target_dir){
                    break 'cluster_loop;
                }
            }

            
        }
        // Check for end of cluster chain
        let next = shell.vol.fat[cwd_cluster as usize] as usize;

        // stop at end of chain and print an error if not found
        if next >= 0x0FFFFFF8 {
            eprintln!("cd: no such directory: {}", target_dir);
            break;
        }

        // IMPORTANT!!!! update cluster here
        cwd_cluster = next;
    }
    // If we reach here, the directory does exist, so we can change into it

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
                // If it matches, I need to update the cwd_cluster in ShellCore to the new directory's starting cluster
                if short_name.eq_ignore_ascii_case(target_dir){
                    // If it is not a directory, print error and return
                    let is_dir = (attr & 0x10) != 0;
                    if !is_dir {
                        eprintln!("cd: not a directory: {}", target_dir);
                        return;
                    }
                    let high_cluster = u16::from_le_bytes([buffer[offset + 20], buffer[offset + 21]]);
                    let low_cluster = u16::from_le_bytes([buffer[offset + 26], buffer[offset + 27]]);
                    let new_cluster = ((high_cluster as u32) << 16) | (low_cluster as u32);
                    shell.cwd_cluster = new_cluster;
                    // Update cwd_path
                    // Handle ".." to go up a directory
                    if target_dir == "/" {
                        shell.cwd_path = String::from("/");
                    } else {
                        if shell.cwd_path != "/" {
                            shell.cwd_path.push('/');
                        }
                        shell.cwd_path.push_str(&short_name);
                    }
                    return;
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
