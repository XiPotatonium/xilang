use std::ffi::CString;
use std::ops::Deref;

use winapi::shared::minwindef::{FARPROC, HMODULE};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryA};

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

    pub fn get_name(&self) -> &str {
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
                    self.get_name(),
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
