use crate::models::ShellCore;

pub fn mv(shell: &mut ShellCore, src: &str, dest: &str) {
    if src.is_empty() || dest.is_empty() {
        eprintln!("mv: missing operand");
        return;
    }
    if src.eq_ignore_ascii_case(dest) {
        eprintln!("mv: source and destination are the same");
        return;
    }

    for of in shell.open_files.iter() {
        if of.name.eq_ignore_ascii_case(src) {
            eprintln!("mv: cannot move open file '{}'", src);
            return;
        }
    }

    let cwd = shell.cwd_cluster;

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

    let hi = u16::from_le_bytes([src_entry[20], src_entry[21]]) as u32;
    let lo = u16::from_le_bytes([src_entry[26], src_entry[27]]) as u32;
    let src_start_cluster = (hi << 16) | lo;

    let dest_entry = shell.vol.find_entry_in_directory(cwd, dest);

    match dest_entry {
        Some((dest_cl, dest_off)) => {
            let entry = shell.vol.read_raw_entry(dest_cl, dest_off).unwrap();
            let attr = entry[11];

            if (attr & 0x10) == 0 {
                eprintln!("mv: cannot overwrite '{}': not a directory", dest);
                return;
            }

            let dest_dir_cluster = {
                let hi = u16::from_le_bytes([entry[20], entry[21]]) as u32;
                let lo = u16::from_le_bytes([entry[26], entry[27]]) as u32;
                let cl = (hi << 16) | lo;
                if cl == 0 { shell.vol.bpb.bpb_root_clus } else { cl }
            };

            if src_is_dir && dest_dir_cluster == src_start_cluster {
                eprintln!("mv: cannot move directory into itself");
                return;
            }

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

            let new_entry = src_entry.clone();
            // Keep the SAME NAME as src
            shell.vol.write_raw_entry(free_cl, free_off, &new_entry)
                .expect("mv: failed to write destination entry");

            shell.vol.mark_entry_deleted(src_cluster, src_offset)
                .expect("mv: failed to delete old entry");

            // Done
            println!("moved '{}' into directory '{}'", src, dest);
        }

        None => {
            let mut new_entry = src_entry.clone();
            shell.vol.set_entry_name(&mut new_entry, dest);

            shell.vol.write_raw_entry(src_cluster, src_offset, &new_entry)
                .expect("mv: failed to update directory entry");

            println!("renamed '{}' → '{}'", src, dest);
        }
    }

    shell.vol.flush_fat().ok();
}