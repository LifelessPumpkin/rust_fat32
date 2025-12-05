use std::{fs::File, io::{Read, Seek, SeekFrom, Write}};

use crate::models::{Volume, ShellCore, BootSector};

// Main core functionality would go here
// For example, functions to read/write clusters, manage FAT, etc.

impl ShellCore {
    // I want to add methods here to init the shell which will include 
    // Setting the volume, current working directory, and open files
    // Also a method to init and set the BootSector struct
    pub fn new(mut image: File) -> Self {
        let bpb = BootSector::new(&mut image);
        let vol = Volume::new(image, bpb);
        ShellCore {
            vol,
            cwd_cluster: bpb.bpb_root_clus,
            cwd_path: String::from("/"),
            open_files: Vec::new(),
        }
    }
}


impl Volume {
    pub fn new(mut file: File, bpb: BootSector) -> Self {
        let first_fat_sector = bpb.bpb_rsvd_sec_cnt as u32;
        let first_data_sector = first_fat_sector + bpb.bpb_fatsz32 * bpb.bpb_num_fats as u32;

        let fat_offset_bytes = first_fat_sector as u64 * bpb.bpb_byts_per_sec as u64;
        let fat_size_bytes = (bpb.bpb_fatsz32 as u64) * bpb.bpb_byts_per_sec as u64;

        let mut fat_buffer = vec![0u8; fat_size_bytes as usize];

        file.seek(SeekFrom::Start(fat_offset_bytes)).unwrap();
        file.read_exact(&mut fat_buffer).unwrap();

        // Convert raw FAT bytes into u32 entries
        let mut fat = Vec::new();
        for chunk in fat_buffer.chunks_exact(4) {
            fat.push(u32::from_le_bytes(chunk.try_into().unwrap()));
        }

        Volume {
            file,
            bpb,
            first_fat_sector,
            first_data_sector,
            fat,
        }
    }

    pub fn get_first_sector_of_cluster(&self, cluster: u32) -> u32 {
        if cluster < 2 {
            panic!("Invalid cluster: {}", cluster);
        }
        self.first_data_sector + (cluster - 2) * self.bpb.bpb_sec_per_clus as u32
    }

    pub fn find_free_directory_entry(&mut self, start_cluster: u32) -> Option<(u32, usize)> {
        let mut cluster = start_cluster;

        loop {
            let first_sector = self.get_first_sector_of_cluster(cluster);
            let bytes_per_sector = self.bpb.bpb_byts_per_sec as usize;
            let sectors = self.bpb.bpb_sec_per_clus as usize;
            let mut buffer = vec![0u8; bytes_per_sector * sectors];

            for s in 0..sectors {
                self.read_sector(first_sector + s as u32,
                    &mut buffer[s*bytes_per_sector..(s+1)*bytes_per_sector]).unwrap();

                for entry_off in (0..bytes_per_sector).step_by(32) {
                    let first_byte = buffer[s*bytes_per_sector + entry_off];

                    if first_byte == 0x00 || first_byte == 0xE5 {
                        // Free entry
                        return Some((cluster, s * bytes_per_sector + entry_off));
                    }
                }
            }

            let next = self.fat[cluster as usize];
            if next >= 0x0FFFFFF8 {
                return None;
            }
            cluster = next;
        }
    }

    pub fn alloc_cluster(&mut self) -> Option<u32> {
        // FAT[0] and FAT[1] are reserved, so start at cluster 2
        for cluster in 2..self.fat.len() {
            if self.fat[cluster] == 0 {
                // Mark as end-of-chain
                self.fat[cluster] = 0x0FFFFFF8;
                return Some(cluster as u32);
            }
        }
        None
    }

    pub fn find_entry_in_directory(&mut self, start_cluster: u32, name: &str) -> Option<(u32, usize)> {
        let mut cluster = start_cluster;

        loop {
            let first_sector = self.get_first_sector_of_cluster(cluster);
            let bytes_per_sector = self.bpb.bpb_byts_per_sec as usize;
            let sectors = self.bpb.bpb_sec_per_clus as usize;
            let mut buffer = vec![0u8; bytes_per_sector * sectors];

            for s in 0..sectors {
                self.read_sector(first_sector + s as u32,
                    &mut buffer[s*bytes_per_sector..(s+1)*bytes_per_sector]).unwrap();

                for entry_off in (0..bytes_per_sector).step_by(32) {
                    let entry = &buffer[s*bytes_per_sector + entry_off ..][..32];
                    let first = entry[0];

                    if first == 0x00 {
                        return None; // end-of-dir
                    }
                    if first == 0xE5 || entry[11] == 0x0F {
                        continue;
                    }

                    let short = self.parse_short_name(&entry[0..11]);
                    if short.eq_ignore_ascii_case(name) {
                        return Some((cluster, s * bytes_per_sector + entry_off));
                    }
                }
            }

            let next = self.fat[cluster as usize];
            if next >= 0x0FFFFFF8 {
                return None;
            }
            cluster = next;
        }
    }

    pub fn initialize_directory_cluster(&mut self, cluster: u32, parent: u32) {
        let bytes_per_sector = self.bpb.bpb_byts_per_sec as usize;
        let sectors = self.bpb.bpb_sec_per_clus as usize;
        let mut buffer = vec![0u8; bytes_per_sector * sectors];

        // "." entry
        {
            let entry = &mut buffer[0..32];
            self.write_directory_entry(entry, ".", 0x10, cluster, 0);
        }

        // ".." entry
        {
            let entry = &mut buffer[32..64];
            self.write_directory_entry(entry, "..", 0x10, parent, 0);
        }

        // Zero the rest
        for b in &mut buffer[64..] {
            *b = 0;
        }

        let first_sector = self.get_first_sector_of_cluster(cluster);

        for i in 0..sectors {
            let sector = first_sector + i as u32;
            let offset = i * bytes_per_sector;
            self.write_sector(sector, &buffer[offset..offset + bytes_per_sector])
                .expect("Failed to write new directory cluster");
        }
    }

    pub fn append_cluster(&mut self, start: u32) -> Option<u32> {
        let mut cur = start;

        loop {
            let entry = self.fat[cur as usize];
            if entry >= 0x0FFFFFF8 {
                break;
            }
            cur = entry;
        }

        let new_cluster = self.alloc_cluster()?;
        self.fat[cur as usize] = new_cluster;
        self.fat[new_cluster as usize] = 0x0FFFFFF8;
        Some(new_cluster)
    }

    pub fn write_directory_entry(&mut self, entry: &mut [u8], name: &str, attr: u8, first_cluster: u32, file_size: u32,) {
        // Format short name: pad or truncate
        let mut name11 = [b' '; 11];

        if name == "." {
            name11[0] = b'.';
        } else if name == ".." {
            name11[0] = b'.';
            name11[1] = b'.';
        } else {
            let (name_part, ext_part) = if let Some((name, ext)) = name.split_once('.') {
                (name, ext)
            } else {
                (name, "")
            };

            // Copy name (max 8 chars)
            for (i, b) in name_part.bytes().take(8).enumerate() {
                name11[i] = b.to_ascii_uppercase();
            }

            // Copy extension (max 3 chars)
            for (i, b) in ext_part.bytes().take(3).enumerate() {
                name11[8 + i] = b.to_ascii_uppercase();
            }
        }

        entry[..11].copy_from_slice(&name11);
        entry[11] = attr; // attribute

        // Zero time/date fields
        entry[12] = 0;
        entry[13] = 0;
        entry[14..20].fill(0);

        // First cluster
        entry[20..22].copy_from_slice(&((first_cluster >> 16) as u16).to_le_bytes());
        entry[22..26].fill(0);
        entry[26..28].copy_from_slice(&(first_cluster as u16).to_le_bytes());
        // File size
        entry[28..32].copy_from_slice(&file_size.to_le_bytes());
    }

    pub fn flush_fat(&mut self) -> std::io::Result<()> {
        let bytes_per_sector = self.bpb.bpb_byts_per_sec as usize;
        let fat_size_bytes = (self.bpb.bpb_fatsz32 as usize) * bytes_per_sector;

        // Rebuild FAT as raw bytes
        let mut fat_raw = vec![0u8; fat_size_bytes];

        for (i, entry) in self.fat.iter().enumerate() {
            let bytes = entry.to_le_bytes();
            fat_raw[i * 4..i * 4 + 4].copy_from_slice(&bytes);
        }

        // Overwrite FAT#1 (and ideally FAT#2)
        let fat_start_sector = self.first_fat_sector;

        for sector in 0..self.bpb.bpb_fatsz32 {
            let offset = (sector as usize) * bytes_per_sector;
            self.write_sector(
                fat_start_sector + sector,
                &fat_raw[offset..offset + bytes_per_sector],
            )?;
        }

        Ok(())
    }

    pub fn parse_short_name(&self, raw_name: &[u8]) -> String {
        let name = String::from_utf8_lossy(&raw_name[0..8]).trim().to_string();
        let ext = String::from_utf8_lossy(&raw_name[8..11]).trim().to_string();
        if ext.is_empty() {
            name
        } else {
            format!("{}.{}", name, ext)
        }
    }

    pub fn read_sector(&mut self, sector: u32, buf: &mut [u8]) -> std::io::Result<()> {
        let offset = sector as u64 * self.bpb.bpb_byts_per_sec as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(buf)?;
        Ok(())
    }

    pub fn write_sector(&mut self, sector: u32, buf: &[u8]) -> std::io::Result<()> {
        let bytes_per_sector = self.bpb.bpb_byts_per_sec as usize;

        // Safety check: buffer must be exactly 1 sector
        if buf.len() != bytes_per_sector {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "write_sector: buffer size {} does not match sector size {}",
                    buf.len(),
                    bytes_per_sector
                ),
            ));
        }

        // Convert sector number into byte offset inside the disk image
        let offset = sector as u64 * bytes_per_sector as u64;

        // Move write pointer
        self.file.seek(SeekFrom::Start(offset))?;

        // Write entire sector
        self.file.write_all(buf)?;

        // Optional but safer:
        // Ensure data is on disk (doesn't stay in OS write buffer)
        self.file.flush()?;

        Ok(())
    }

}