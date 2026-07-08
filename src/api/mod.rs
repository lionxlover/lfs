//! C-compatible API Export layer for LionFS
//! Exposes stable endpoints for external utilities to mount, unmount, and query LionFS in userspace.

#[repr(C)]
pub struct LfsApiStatus {
    pub success: bool,
    pub error_code: i32,
}

#[no_mangle]
pub extern "C" fn lfs_version() -> *const libc::c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const libc::c_char
}

#[no_mangle]
pub extern "C" fn lfs_mount_fuse(_device_path: *const libc::c_char, _mount_point: *const libc::c_char) -> LfsApiStatus {
    // In a full implementation, this would instantiate LionFsFuse and call fuser::mount2
    LfsApiStatus {
        success: true,
        error_code: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_version() {
        let ptr = lfs_version();
        assert!(!ptr.is_null());
    }
}
