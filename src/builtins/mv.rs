use crate::models::ShellCore;

pub fn mv(shell: &mut ShellCore, src: &str, dest: &str) {
    // 1. Validate args
    if src.is_empty() || dest.is_empty() {
        eprintln!("mv: missing operand");
        return;
    }
    if src.eq_ignore_ascii_case(dest) {
        eprintln!("mv: source and destination are the same");
        return;
    }

    // 2. Ensure source is not open
    for of in shell.open_files.iter() {
        if of.name.eq_ignore_ascii_case(src) {
            eprintln!("mv: cannot move open file '{}'", src);
            return;
        }
    }

    let cwd = shell.cwd_cluster;

    // 3. Find the source entry
    let (src_cluster, src_offset, src_entry) =
        match shell.vol.find_entry_in_directory(cwd, src)
    {
        Some((cl, off)) => {
            let entry = shell.vol.read_raw_entry(cl, off).unwrap();
            (cl, off, entry)
        }
        None => {
            eprintln!("mv: cannot stat '{}': No such file or directory", src);
            return;
        }
    };

    let src_attr = src_entry[11];
    let src_is_dir = (src_attr & 0x10) != 0;

    // Extract cluster + size
    let hi = u16::from_le_bytes([src_entry[20], src_entry[21]]) as u32;
    let lo = u16::from_le_bytes([src_entry[26], src_entry[27]]) as u32;
    let src_start_cluster = (hi << 16) | lo;
    // let src_size = u32::from_le_bytes([
    //     src_entry[28], src_entry[29], src_entry[30], src_entry[31],
    // ]);

    // ----------------------------------------------
    // DESTINATION LOGIC
    // ----------------------------------------------

    // Check if dest exists in CWD
    let dest_entry = shell.vol.find_entry_in_directory(cwd, dest);

    match dest_entry {
        // CASE 1: destination exists → must be a directory
        Some((dest_cl, dest_off)) => {
            // read dest entry to see if it's directory
            let entry = shell.vol.read_raw_entry(dest_cl, dest_off).unwrap();
            let attr = entry[11];

            if (attr & 0x10) == 0 {
                eprintln!("mv: cannot overwrite '{}': not a directory", dest);
                return;
            }

            // Destination is directory → move src inside this directory
            let dest_dir_cluster = {
                let hi = u16::from_le_bytes([entry[20], entry[21]]) as u32;
                let lo = u16::from_le_bytes([entry[26], entry[27]]) as u32;
                let cl = (hi << 16) | lo;
                if cl == 0 { shell.vol.bpb.bpb_root_clus } else { cl }
            };

            // Ensure not moving a directory into itself
            if src_is_dir && dest_dir_cluster == src_start_cluster {
                eprintln!("mv: cannot move directory into itself");
                return;
            }

            // Find free slot in dest directory
            let (free_cl, free_off) = match shell.vol.find_free_directory_entry(dest_dir_cluster) {
                Some(v) => v,
                None => {
                    if let Some(new_cl) = shell.vol.append_cluster(dest_dir_cluster) {
                        shell.vol.find_free_directory_entry(new_cl).unwrap()
                    } else {
                        eprintln!("mv: destination directory is full");
                        return;
                    }
                }
            };

            // Write src entry into dest directory
            let new_entry = src_entry.clone();
            // Keep the SAME NAME as src
            shell.vol.write_raw_entry(free_cl, free_off, &new_entry)
                .expect("mv: failed to write destination entry");

            // Mark original as deleted
            shell.vol.mark_entry_deleted(src_cluster, src_offset)
                .expect("mv: failed to delete old entry");

            // Done
            println!("moved '{}' into directory '{}'", src, dest);
        }

        // CASE 2: destination does not exist → simple rename
        None => {
            // Just rename entry IN PLACE
            let mut new_entry = src_entry.clone();
            shell.vol.set_entry_name(&mut new_entry, dest);

            // Write updated 32-byte entry back
            shell.vol.write_raw_entry(src_cluster, src_offset, &new_entry)
                .expect("mv: failed to update directory entry");

            println!("renamed '{}' → '{}'", src, dest);
        }
    }

    shell.vol.flush_fat().ok();
}