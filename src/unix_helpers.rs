use std::ffi::CStr;

// just wraps libc's if_indextoname() - likely suboptimal
pub fn interface_name(index: u32) -> String {
    let mut c_buf: [libc::c_char; libc::IF_NAMESIZE] = [0; libc::IF_NAMESIZE];
    let ret = unsafe { libc::if_indextoname(index, c_buf.as_mut_ptr()) };
    if ret.is_null() {
        return "".to_string();
    }
    let c_str: &CStr = unsafe { CStr::from_ptr(c_buf.as_ptr()) };

    c_str.to_str().unwrap().to_owned()
}
