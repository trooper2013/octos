pub mod model_loader;

#[cfg(target_os = "linux")]
pub fn mount_vfs() -> Result<(), std::io::Error> {
    use std::ffi::CString;
    // Proc mount
    let source = CString::new("proc").unwrap();
    let target = CString::new("/proc").unwrap();
    let fstype = CString::new("proc").unwrap();
    let res = unsafe { libc::mount(source.as_ptr(), target.as_ptr(), fstype.as_ptr(), 0, std::ptr::null()) };
    if res != 0 {
        return Err(std::io::Error::last_os_error());
    }

    // Sys mount
    let source = CString::new("sysfs").unwrap();
    let target = CString::new("/sys").unwrap();
    let fstype = CString::new("sysfs").unwrap();
    let res = unsafe { libc::mount(source.as_ptr(), target.as_ptr(), fstype.as_ptr(), 0, std::ptr::null()) };
    if res != 0 {
        return Err(std::io::Error::last_os_error());
    }

    // Dev mount (devtmpfs with fallback)
    let source = CString::new("devtmpfs").unwrap();
    let target = CString::new("/dev").unwrap();
    let fstype = CString::new("devtmpfs").unwrap();
    let res = unsafe { libc::mount(source.as_ptr(), target.as_ptr(), fstype.as_ptr(), 0, std::ptr::null()) };
    if res != 0 {
        let source_tmp = CString::new("tmpfs").unwrap();
        let fstype_tmp = CString::new("tmpfs").unwrap();
        let res_tmp = unsafe { libc::mount(source_tmp.as_ptr(), target.as_ptr(), fstype_tmp.as_ptr(), 0, std::ptr::null()) };
        if res_tmp != 0 {
            return Err(std::io::Error::last_os_error());
        }
    }
    
    println!("[SYSTEM LOG] [INIT] VFS mounts (/proc, /sys, /dev) completed successfully.");
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn mount_vfs() -> Result<(), std::io::Error> {
    println!("[SYSTEM LOG] [INIT] [MOCK] Simulating POSIX VFS mounts (/proc, /sys, /dev) on non-Linux host.");
    Ok(())
}
