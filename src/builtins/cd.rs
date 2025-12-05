use crate::models::ShellCore;


pub fn cd(target_dir: &str, shell: &mut ShellCore) {
    if target_dir == "" {
        eprintln!("cd: missing operand");
        return;
    }
    if target_dir == "." {
        return;
    }

    if target_dir == ".." {
        let root = shell.vol.bpb.bpb_root_clus;
        let mut cluster = shell.cwd_cluster;
        if cluster == root {
            return;
        }

        loop {
            let first_sector = shell.vol.get_first_sector_of_cluster(cluster);
            let bytes_per_sector = shell.vol.bpb.bpb_byts_per_sec as usize;
            let sectors_per_cluster = shell.vol.bpb.bpb_sec_per_clus as usize;
            let cluster_size = bytes_per_sector * sectors_per_cluster;
            let mut buffer = vec![0u8; cluster_size];

            for sec in 0..sectors_per_cluster {
                let sector = first_sector + sec as u32;
                let offset = sec * bytes_per_sector;
                shell.vol.read_sector(sector, &mut buffer[offset..offset + bytes_per_sector]).unwrap();
            }

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

                    shell.cwd_cluster = if parent_cluster == 0 { root } else { parent_cluster };

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

            let next = shell.vol.fat[cluster as usize];
            if next >= 0x0FFFFFF8 {
                break;
            }
            cluster = next;
        }
        return;
    }
    
    let mut cwd_cluster = shell.cwd_cluster as usize;
    'cluster_loop: loop {
        let first_sector = shell.vol.get_first_sector_of_cluster(cwd_cluster as u32);
        let bytes_per_sector = shell.vol.bpb.bpb_byts_per_sec as usize;
        let sectors_per_cluster = shell.vol.bpb.bpb_sec_per_clus as usize;
        let cluster_size = bytes_per_sector * sectors_per_cluster;
        let mut buffer = vec![0u8; cluster_size];
        'sector_loop: for sector_offset in 0..sectors_per_cluster {
            let sector_number = first_sector + sector_offset as u32;
            let offset = sector_offset * bytes_per_sector;
            shell.vol.read_sector(sector_number, &mut buffer[offset..offset + bytes_per_sector]).expect("Failed to read sector");
            for entry_offset in (0..bytes_per_sector).step_by(32) {
                let offset = sector_offset * bytes_per_sector + entry_offset;
                let first_byte = buffer[offset];

                if first_byte == 0x00 {
                    break 'sector_loop;
                }
                if first_byte == 0xE5 {
                    continue;
                }

                let attr = buffer[offset + 11];
                if attr == 0x0F {
                    continue;
                }

                let short_name = shell.vol.parse_short_name(&buffer[offset..offset + 11]);
                if short_name.eq_ignore_ascii_case(target_dir){
                    break 'cluster_loop;
                }
            }

            
        }
        let next = shell.vol.fat[cwd_cluster as usize] as usize;

        if next >= 0x0FFFFFF8 {
            eprintln!("cd: no such directory: {}", target_dir);
            break;
        }
        cwd_cluster = next;
    }

    let mut cwd_cluster = shell.cwd_cluster as usize;
    loop {
        let first_sector = shell.vol.get_first_sector_of_cluster(cwd_cluster as u32);
        let bytes_per_sector = shell.vol.bpb.bpb_byts_per_sec as usize;
        let sectors_per_cluster = shell.vol.bpb.bpb_sec_per_clus as usize;
        let cluster_size = bytes_per_sector * sectors_per_cluster;
        let mut buffer = vec![0u8; cluster_size];
        'cluster_loop: for sector_offset in 0..sectors_per_cluster {
            let sector_number = first_sector + sector_offset as u32;
            let offset = sector_offset * bytes_per_sector;
            shell.vol.read_sector(sector_number, &mut buffer[offset..offset + bytes_per_sector]).expect("Failed to read sector");
            for entry_offset in (0..bytes_per_sector).step_by(32) {
                let offset = sector_offset * bytes_per_sector + entry_offset;
                let first_byte = buffer[offset];

                if first_byte == 0x00 {
                    break 'cluster_loop;
                }
                if first_byte == 0xE5 {
                    continue;
                }

                let attr = buffer[offset + 11];
                if attr == 0x0F {
                    continue;
                }

                let short_name = shell.vol.parse_short_name(&buffer[offset..offset + 11]);
                if short_name.eq_ignore_ascii_case(target_dir){
                    let is_dir = (attr & 0x10) != 0;
                    if !is_dir {
                        eprintln!("cd: not a directory: {}", target_dir);
                        return;
                    }
                    let high_cluster = u16::from_le_bytes([buffer[offset + 20], buffer[offset + 21]]);
                    let low_cluster = u16::from_le_bytes([buffer[offset + 26], buffer[offset + 27]]);
                    let new_cluster = ((high_cluster as u32) << 16) | (low_cluster as u32);
                    shell.cwd_cluster = new_cluster;
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
        let next = shell.vol.fat[cwd_cluster as usize] as usize;

        if next >= 0x0FFFFFF8 {
            break;
        }

        cwd_cluster = next;
    }
}
