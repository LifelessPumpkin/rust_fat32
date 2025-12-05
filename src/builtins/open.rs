use crate::models::{FileMode, OpenFile, ShellCore};


pub fn open(filename: &str, mode: &str, shell: &mut ShellCore) {
    if filename == "" {
        eprintln!("open: missing filename");
        return;
    }
    if mode != "r" && mode != "w" && mode != "rw" && mode != "wr" && mode != "-r" && mode != "-w" && mode != "-rw" && mode != "-wr" {
        eprintln!("Invalid mode specified for open command. Use -r, -w, -rw, or -wr.");
        return;
    }
    for of in shell.open_files.iter() {
        if of.name.eq_ignore_ascii_case(filename) {
            eprintln!("open: file already open: {}", filename);
            return;
        }
    }
    if shell.open_files.len() >= 10 {
        eprintln!("open: maximum number of open files reached.");
        return;
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
                if !short_name.eq_ignore_ascii_case(filename) {
                    continue;
                }
                let is_dir = (attr & 0x10) != 0;
                if is_dir {
                    eprintln!("open: not a file: {}", filename);
                    return;
                }
                if mode.contains('w') && (attr & 0x01 != 0) {
                    eprintln!("open: file is read-only: {}", filename);
                    return;
                }
                let high_cluster = u16::from_le_bytes([buffer[offset + 20], buffer[offset + 21]]);
                let low_cluster = u16::from_le_bytes([buffer[offset + 26], buffer[offset + 27]]);
                let new_cluster = ((high_cluster as u32) << 16) | (low_cluster as u32);
                let file_size = u32::from_le_bytes([buffer[offset + 28], buffer[offset + 29], buffer[offset + 30], buffer[offset + 31]]);
                let mut file_descriptor = 0;
                let mut fd_found = false;
                while !fd_found {
                    fd_found = true;
                    for of in shell.open_files.iter() {
                        if of.file_descriptor == file_descriptor {
                            file_descriptor += 1;
                            fd_found = false;
                            break;
                        }
                    }
                }
                let open_file = OpenFile {
                    name: short_name,
                    file_descriptor,
                    dir_cluster: cwd_cluster as u32,
                    dir_cluster_path: shell.cwd_path.clone(),
                    start_cluster: new_cluster,
                    size: file_size,
                    offset: 0,
                    mode: match mode {
                        "r" => FileMode::Read,
                        "w" => FileMode::Write,
                        "rw" | "wr" => FileMode::ReadWrite,
                        _ => unreachable!(),
                    },
                };
                shell.open_files.push(open_file);
                
                return;
            }

            
        }
        let next = shell.vol.fat[cwd_cluster as usize] as usize;

        if next >= 0x0FFFFFF8 {
            break;
        }

        cwd_cluster = next;
    }
}
