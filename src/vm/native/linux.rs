use std::ffi::{CStr, CString};
use std::ops::Deref;

use libc::{c_void, dlclose, dlerror, dlopen, dlsym, RTLD_LAZY, RTLD_LOCAL};

pub struct ExternalDll {
    name: CString,
    handle: *mut c_void,
}

impl ExternalDll {
    pub fn new_ascii(libpath: &str) -> Result<ExternalDll, String> {
        let name = CString::new(libpath).unwrap();
        unsafe {
            let handle = dlopen(name.as_ptr(), RTLD_LAZY | RTLD_LOCAL);
            if handle.is_null() {
                return Err(format!(
                    "{}: Fail to load lib {}",
                    CStr::from_ptr(dlerror()).to_str().unwrap(),
                    libpath
                ));
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
            let handle = dlsym(self.handle, name.as_ptr());

            if handle.is_null() {
                Err(format!(
                    "{}: Fail to load symbol {} in {}",
                    CStr::from_ptr(dlerror()).to_str().unwrap(),
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
            dlclose(self.handle);
        }
    }
}

pub struct ExternalFn<T> {
    handle: *mut c_void,
    lifetime_marker: std::marker::PhantomData<T>,
}

impl<T> Deref for ExternalFn<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(&self.handle as *const *mut _ as *const T) }
    }
}
