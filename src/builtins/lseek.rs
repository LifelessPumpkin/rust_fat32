use crate::models::ShellCore;


pub fn lseek(file_descriptor: usize, offset: u32, shell: &mut ShellCore) {
    // Find the file in open_files and update its offset
    if let Some(of) = shell.open_files.iter_mut().find(|of| of.file_descriptor == file_descriptor) {
        if offset > of.size {
            // If its greater than file size, set the offset to file size
            of.offset = of.size;
        } else {
            of.offset = offset;
        }
    } else {
        eprintln!("lseek: file not open: {}", file_descriptor);
    }
}
