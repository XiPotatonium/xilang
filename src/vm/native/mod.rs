#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "linux")]
use linux as os;
#[cfg(target_os = "windows")]
use win as os;

use super::stack::{Args, Slot};

use std::ffi::CString;
use std::os::raw::{c_char, c_uchar};

static NATIVE_BRIDGE_FNAME: &str = "native_bridge";

/// Return state of dll function
#[repr(C)]
enum NativeState {
    Ok,
    /// Function not found
    NoFunc,
}

pub struct VMDll {
    dll: os::ExternalDll,
    /// TODO: use hashmap to optimize funtion dispatch
    bridge_fn: os::ExternalFn<
        unsafe extern "C" fn(*const c_char, *const c_uchar, *mut Slot) -> NativeState,
    >,
}

impl VMDll {
    pub fn new_ascii(libpath: &str) -> Result<VMDll, String> {
        let dll = os::ExternalDll::new_ascii(libpath)?;
        let bridge_fn = dll.load_fn(NATIVE_BRIDGE_FNAME)?;
        Ok(VMDll { dll, bridge_fn })
    }

    pub fn call(&self, name: &str, args: Args, ret: *mut Slot) {
        let fname = CString::new(name).unwrap();
        let state =
            unsafe { (*self.bridge_fn)(fname.as_ptr(), args.as_ptr() as *const c_uchar, ret) };
        match state {
            NativeState::Ok => {}
            NativeState::NoFunc => {
                panic!("No function {} in dll {}", name, self.dll.get_name());
            }
        }
    }
}
