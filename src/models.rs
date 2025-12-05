use std::{fs::File, io::Read};
#[derive(Copy, Clone)]
pub struct BootSector {
    pub bpb_byts_per_sec: u16,
    pub bpb_sec_per_clus: u8,
    pub bpb_rsvd_sec_cnt: u16,
    pub bpb_num_fats: u8,
    pub bpb_fatsz32: u32,
    pub bpb_tot_sec32: u32,
    pub bpb_root_clus: u32,
    pub file_size: u64,
}
impl BootSector {
    pub fn new(image: &mut File) -> Self {
    let mut buffer = [0; 512];
    match image.read(&mut buffer) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to read from image file: {}", e);
            std::process::exit(1);
        }
    }
    let bpb_byts_per_sec = u16::from_le_bytes([buffer[11], buffer[12]]);
    let bpb_sec_per_clus = buffer[13];
    let bpb_rsvd_sec_cnt = u16::from_le_bytes([buffer[14], buffer[15]]);
    let bpb_num_fats = buffer[16];
    let bpb_tot_sec32 = u32::from_le_bytes([buffer[32], buffer[33], buffer[34], buffer[35]]);
    let bpb_fatsz32 = u32::from_le_bytes([buffer[36], buffer[37], buffer[38], buffer[39]]);
    let bpb_root_clus = u32::from_le_bytes([buffer[44], buffer[45], buffer[46], buffer[47]]);
    let metadata = match image.metadata() {
        Ok(meta) => meta,
        Err(e) => {
            eprintln!("Failed to get metadata for image file: {}", e);
            std::process::exit(1);
        }
    };
    let file_size = metadata.len();

    BootSector {
        bpb_byts_per_sec,
        bpb_sec_per_clus,
        bpb_rsvd_sec_cnt,
        bpb_num_fats,
        bpb_fatsz32,
        bpb_tot_sec32,
        bpb_root_clus,
        file_size,
    }
    }
}
pub struct Volume {
    pub file: File,
    pub bpb: BootSector,
    pub first_fat_sector: u32,
    pub first_data_sector: u32,
    pub fat: Vec<u32>
}

pub struct OpenFile {
    pub name: String,
    pub file_descriptor: usize,
    pub dir_cluster: u32,      // where its dir entry lives
    pub dir_cluster_path: String, // path to the directory containing the file
    pub start_cluster: u32,
    pub size: u32,
    pub offset: u32,
    pub mode: FileMode,
}

pub enum FileMode {
    Read,
    Write,
    ReadWrite,
}

pub struct ShellCore {
    pub vol: Volume,
    pub cwd_cluster: u32,
    pub cwd_path: String,
    pub open_files: Vec<OpenFile>, // max 10
}
