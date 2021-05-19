use std::io::{stdout, Write};
use std::mem;
use std::ptr;

use super::super::data::{BuiltinType, MethodImpl};
use super::super::heap::Heap;
use super::super::shared_mem::SharedMem;
use super::super::stack::{Args, Slot, SlotTag};

// pub type InternalCallType = fn(&mut Args, *mut Slot, &mut SharedMem);

#[derive(Clone)]
pub struct InternalCallWrapper(*mut u8);

impl InternalCallWrapper {
    fn as_ref<'m, T>(&self) -> &T {
        unsafe { &*(&self.0 as *const *mut _ as *const T) }
    }

    pub fn call<'m>(&self, args: Args<'m>, ret: *mut Slot, mem: &'m mut SharedMem) {
        self.as_ref::<fn(Args<'m>, *mut Slot, &'m mut SharedMem)>()(args, ret, mem);
    }
}

impl Default for InternalCallWrapper {
    fn default() -> Self {
        Self(ptr::null_mut())
    }
}

pub fn register_internal_calls(m: &mut SharedMem) {
    let std_mod = m.mods.get_mut(&m.std_str_idx).unwrap().expect_il_mut();
    for method in std_mod.methods.iter_mut() {
        let is_static = method.is_static();
        if let MethodImpl::Runtime(runtime_impl) = &mut method.method_impl {
            if let Some(ty) = unsafe { method.parent.as_ref() } {
                let ty_name = &m.str_pool[ty.name];
                let method_name = &m.str_pool[method.name];
                if ty_name == "IO" {
                    if method_name == "write" {
                        if method.ps.len() == 1 && is_static {
                            if let BuiltinType::String = method.ps[0].ty {
                                runtime_impl.func =
                                    InternalCallWrapper(std_io_write_string as *mut u8);
                            }
                        }
                    }
                } else if ty_name == "String" {
                    if method_name == "len" && !is_static {
                        // is check needed here?
                        runtime_impl.func = InternalCallWrapper(std_string_len as *mut u8);
                    }
                }
            }

            if runtime_impl.func.0.is_null() {
                panic!(
                    "No matched internal function for {}",
                    method.str_desc_with_fullname(&m.str_pool)
                );
            }
        }
    }
}

pub fn std_io_write_string<'m>(args: Args<'m>, _: *mut Slot, _: &'m mut SharedMem) {
    let s = unsafe { *(args.as_ptr() as *const *mut u8) };
    let chars = Heap::get_chars(s);
    for ch in chars {
        print!("{}", ch);
    }
    stdout().flush().unwrap();
}

pub fn std_string_len<'m>(args: Args<'m>, ret: *mut Slot, _: &'m mut SharedMem) {
    let s = args.get_self().unwrap();
    let chars = Heap::get_chars(s);
    unsafe {
        let ret = ret.as_mut().unwrap();
        ret.tag = SlotTag::I32;
        ret.data.i32_ = mem::transmute::<u32, i32>(chars.total_len() as u32);
    }
}
