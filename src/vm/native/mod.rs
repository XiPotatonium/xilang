use super::mem::slot::Slot;
use core::panic;
use std::ffi::CString;
use std::ops::Deref;

#[cfg(windows)]
use winapi::shared::minwindef::{FARPROC, HMODULE};
#[cfg(windows)]
use winapi::um::errhandlingapi::GetLastError;
#[cfg(windows)]
use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryA};

static NATIVE_BRIDGE_FNAME: &str = "native_bridge";

/// Return state of dll function
#[repr(C)]
enum NativeState {
    Ok,
    /// Function not found
    NoFunc,
    /// Wrong argument number
    WrongArgc,
    /// Wrong argument type
    WrongArgTy,
}

pub struct VMDll {
    dll: ExternalDll,
    /// TODO: use hashmap to optimize funtion dispatch
    bridge_fn:
        ExternalFn<unsafe extern "C" fn(*const i8, i32, *const Slot, *mut Slot) -> NativeState>,
}

impl VMDll {
    pub fn new_ascii(libpath: &str) -> Result<VMDll, String> {
        let dll = ExternalDll::new_ascii(libpath)?;
        let bridge_fn = dll.load_fn(NATIVE_BRIDGE_FNAME)?;
        Ok(VMDll { dll, bridge_fn })
    }

    pub fn call(&self, name: &str, args: &Vec<Slot>, ret: &mut [Slot]) {
        let fname = CString::new(name).unwrap();
        let state = unsafe {
            (*self.bridge_fn)(
                fname.as_ptr(),
                args.len() as i32,
                args.as_ptr(),
                ret.as_mut_ptr(),
            )
        };
        match state {
            NativeState::Ok => {}
            NativeState::NoFunc => {
                panic!("No function {} in dll {}", name, self.dll.rust_name());
            }
            NativeState::WrongArgc => {
                panic!("Mismatch arg number");
            }
            NativeState::WrongArgTy => {
                panic!("Mismatch arg type");
            }
        }
    }
}

pub struct ExternalDll {
    name: CString,
    handle: HMODULE,
}

impl ExternalDll {
    pub fn new_ascii(libpath: &str) -> Result<ExternalDll, String> {
        let name = CString::new(libpath).unwrap();
        unsafe {
            let handle = LoadLibraryA(name.as_ptr());
            if handle.is_null() {
                return Err(format!("{}: Fail to load lib {}", GetLastError(), libpath));
            }

            Ok(ExternalDll { name, handle })
        }
    }

    pub fn rust_name(&self) -> &str {
        self.name.to_str().unwrap()
    }

    pub fn load_fn<T>(&self, fn_name: &str) -> Result<ExternalFn<T>, String> {
        let name = CString::new(fn_name).unwrap();
        unsafe {
            let handle = GetProcAddress(self.handle, name.as_ptr());

            if handle.is_null() {
                Err(format!(
                    "{}: Fail to load symbol {} in {}",
                    GetLastError(),
                    self.rust_name(),
                    fn_name
                ))
            } else {
                Ok(ExternalFn {
                    handle,
                    lifetime_marker: std::marker::PhantomData,
                })
            }
        }
    }
}

impl Drop for ExternalDll {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.handle);
        }
    }
}

pub struct ExternalFn<T> {
    handle: FARPROC,
    lifetime_marker: std::marker::PhantomData<T>,
}

impl<T> Deref for ExternalFn<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(&self.handle as *const *mut _ as *const T) }
    }
}
