use std::ffi::CStr;
use tokio::runtime::Handle;

use crate::handle_group_msg;

#[unsafe(no_mangle)]
pub extern "C" fn pmsg_process(_user_id: u64, _msg_ptr: *const libc::c_char) {}

#[unsafe(no_mangle)]
pub extern "C" fn gmsg_process(group_id: u64, user_id: u64, msg_ptr: *const libc::c_char) {
    unsafe {
        let c_str = CStr::from_ptr(msg_ptr);
        let msg = c_str.to_str().unwrap_or("");
        
        handle_group_msg(group_id, user_id, msg);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pevent_process(_json_ptr: *const libc::c_char) {
}

#[unsafe(no_mangle)]
pub extern "C" fn gevent_process(_json_ptr: *const libc::c_char) {
}
