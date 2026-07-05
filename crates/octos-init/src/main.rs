use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

use octos_init::{mount_vfs, model_loader};

#[tokio::main]
async fn main() {
    println!("[SYSTEM LOG] [INIT] Initializing Octos Custom PID 1 Init Subsystem...");

    // Mount VFS
    if let Err(e) = mount_vfs() {
        eprintln!("[SYSTEM LOG] [INIT] [ERROR] VFS mount failed: {}", e);
    }

    // Early stage model weights (e.g. 512 MB ONNX space)
    if let Err(e) = model_loader::stage_model_weights("llama-3b-q4.gguf", 512 * 1024 * 1024) {
        eprintln!("[SYSTEM LOG] [INIT] [ERROR] Weight staging failed: {}", e);
    }

    // Supervision of octos-core process
    println!("[SYSTEM LOG] [INIT] Launching core service daemon 'octos-core'...");
    
    let mut child = match Command::new("./octos-core").spawn() {
        Ok(c) => c,
        Err(_) => {
            match Command::new("target/debug/octos-core")
                .spawn()
                .or_else(|_| Command::new("../target/debug/octos-core").spawn())
                .or_else(|_| Command::new("../../target/debug/octos-core").spawn())
            {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[SYSTEM LOG] [INIT] [WARNING] Could not locate 'octos-core' executable: {}", e);
                    loop {
                        sleep(Duration::from_secs(60)).await;
                    }
                }
            }
        }
    };

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                println!("[SYSTEM LOG] [INIT] 'octos-core' exited with status: {}. Restarting...", status);
                if let Ok(c) = Command::new("target/debug/octos-core")
                    .spawn()
                    .or_else(|_| Command::new("../target/debug/octos-core").spawn())
                    .or_else(|_| Command::new("./octos-core").spawn())
                {
                    child = c;
                }
            }
            Ok(None) => {
                // Running normally
            }
            Err(e) => {
                eprintln!("[SYSTEM LOG] [INIT] [ERROR] Child supervision process query error: {}", e);
            }
        }
        sleep(Duration::from_secs(5)).await;
    }
}
