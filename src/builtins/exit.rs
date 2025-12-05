
pub fn exit() {
    println!("Exiting core module.");
    // Close all open files and clean up resources if necessary
    // shell.vol.file.sync_all().expect("Failed to sync file");
    std::process::exit(0);
}
