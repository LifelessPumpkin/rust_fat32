use crate::models::ShellCore;


pub fn close(file_descriptor: usize, shell: &mut ShellCore) {
    if let Some(pos) = shell.open_files.iter().position(|of| of.file_descriptor == file_descriptor) {
        shell.open_files.remove(pos);
    } else {
        eprintln!("close: file not open: {}", file_descriptor);
    }
}
