use crate::models::ShellCore;


pub fn close(file_descriptor: usize, shell: &mut ShellCore) {
    // Find the file in open_files and remove it
    if let Some(pos) = shell.open_files.iter().position(|of| of.file_descriptor == file_descriptor) {
        shell.open_files.remove(pos);
    } else {
        eprintln!("close: file not open: {}", file_descriptor);
    }
}
