use crate::models::ShellCore;


pub fn ls(shell: &mut ShellCore) {
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
                let is_dir = (attr & 0x10) != 0;

                if is_dir {
                    println!("[DIR]  {}", short_name);
                } else {
                    println!("[FILE] {}", short_name);
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
