use crate::models::BootSector;


pub fn info(bsb: &BootSector) {
    println!("Boot Sector Information:");
    println!("Root Cluster: {}", bsb.bpb_root_clus);
    println!("Bytes per Sector: {}", bsb.bpb_byts_per_sec);
    println!("Sectors per Cluster: {}", bsb.bpb_sec_per_clus);
    println!("Total Sectors: {}", bsb.bpb_tot_sec32);
    println!("Sectors per FAT: {}", bsb.bpb_fatsz32);
    println!("File Size: {} bytes", bsb.file_size);
}
