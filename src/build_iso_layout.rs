use std::fs;
use std::path::Path;

/// Generates an Alpine-compatible rootfs directory tree skeleton and copies
/// the compiled octos-init binary to /sbin/init.
pub fn generate_rootfs_layout(base_path: &str) -> Result<(), std::io::Error> {
    let base = Path::new(base_path);

    // Replicate essential directories
    let dirs = [
        "bin", "sbin", "etc", "proc", "sys", "dev", "root", "usr/bin"
    ];

    for dir in &dirs {
        let dir_path = base.join(dir);
        fs::create_dir_all(&dir_path)?;
        println!("[SYSTEM LOG] [BUILD] Created directory: {:?}", dir_path);
    }

    // Locate and copy compiled init binary
    let possible_bins = [
        "target/debug/octos-init.exe",
        "target/debug/octos-init",
        "target/release/octos-init.exe",
        "target/release/octos-init",
        "../target/debug/octos-init.exe",
        "../target/debug/octos-init",
        "../../target/debug/octos-init.exe",
        "../../target/debug/octos-init",
    ];

    let mut found = false;
    let dest_init = base.join("sbin/init");

    for bin in &possible_bins {
        if Path::new(bin).exists() {
            fs::copy(bin, &dest_init)?;
            println!("[SYSTEM LOG] [BUILD] Copied init binary from {} to {:?}", bin, dest_init);
            found = true;
            break;
        }
    }

    if !found {
        // Fallback placeholder shell initialization script
        fs::write(&dest_init, "#!/bin/sh\necho 'Octos PID 1 Init'")?;
        println!("[SYSTEM LOG] [BUILD] Init binary not found yet. Created fallback script at {:?}", dest_init);
    }

    Ok(())
}
