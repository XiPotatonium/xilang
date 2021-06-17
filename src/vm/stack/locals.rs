use super::super::data::{BuiltinType, Local, MethodDesc, Param, TypedAddr};
use super::{EvalStack, Slot, SlotTag};

use std::mem;

pub trait ILocals {
    fn load(&self, i: usize, stack: &mut EvalStack);
    fn loada(&self, i: usize, stack: &mut EvalStack);
    fn store(&mut self, i: usize, stack: &mut EvalStack);
    fn store_slot(&mut self, i: usize, slot: Slot);
}

// load from addr into stack
unsafe fn load(ty: &BuiltinType, addr: *const u8, stack: &mut EvalStack) {
    match ty {
        BuiltinType::Void => panic!("Cannot store void"),
        BuiltinType::U1 | BuiltinType::Bool => {
            stack.push_i32(mem::transmute::<u32, i32>(*addr as u32));
        }
        BuiltinType::Char => unimplemented!(),
        BuiltinType::I1 => unimplemented!(),
        BuiltinType::U4 => unimplemented!(),
        BuiltinType::I4 => stack.push_i32(*(addr as *const i32)),
        BuiltinType::U8 => unimplemented!(),
        BuiltinType::I8 => unimplemented!(),
        BuiltinType::UNative => unimplemented!(),
        BuiltinType::INative => unimplemented!(),
        BuiltinType::R4 => unimplemented!(),
        BuiltinType::R8 => unimplemented!(),
        BuiltinType::String
        | BuiltinType::Class(_)
        | BuiltinType::ByRef(_)
        | BuiltinType::SZArray(_) => stack.push_ptr(*(addr as *const *mut u8)),
        BuiltinType::Value(_) => unimplemented!(),
        BuiltinType::GenericInst(_, _, _) => todo!(),
        BuiltinType::Unk => unreachable!(),
    }
}

// store slot into addr
unsafe fn store_slot(ty: &BuiltinType, addr: *mut u8, slot: Slot) {
    match ty {
        BuiltinType::Void => panic!("Cannot store void"),
        BuiltinType::Unk => unreachable!(),
        BuiltinType::Bool | BuiltinType::U1 => {
            slot.expect(SlotTag::I32);
            *addr = mem::transmute::<i32, u32>(slot.data.i32_) as u8;
        }
        BuiltinType::Char => unimplemented!(),
        BuiltinType::I1 => unimplemented!(),
        BuiltinType::U4 => unimplemented!(),
        BuiltinType::I4 => {
            slot.expect(SlotTag::I32);
            *(addr as *mut i32) = slot.data.i32_;
        }
        BuiltinType::U8 => unimplemented!(),
        BuiltinType::I8 => unimplemented!(),
        BuiltinType::UNative => unimplemented!(),
        BuiltinType::INative => unimplemented!(),
        BuiltinType::R4 => unimplemented!(),
        BuiltinType::R8 => unimplemented!(),
        BuiltinType::String
        | BuiltinType::Class(_)
        | BuiltinType::ByRef(_)
        | BuiltinType::SZArray(_) => {
            *(addr as *mut *mut u8) = slot.expect_ref();
        }
        BuiltinType::Value(_) => unimplemented!(),
        BuiltinType::GenericInst(_, _, _) => todo!(),
    }
}

/// locals is fix-sized stack location which stores local vars
pub struct Locals<'m> {
    data: Vec<u8>,
    map: &'m Vec<Local>,
}

impl<'m> Locals<'m> {
    pub fn new(parent: &'m MethodDesc) -> Locals<'m> {
        let method_impl = parent.method_impl.expect_il();
        Locals {
            data: vec![0; method_impl.locals_size],
            map: &method_impl.locals,
        }
    }
}

impl<'m> ILocals for Locals<'m> {
    fn load(&self, i: usize, stack: &mut EvalStack) {
        assert!(i < self.map.len());
        unsafe {
            load(
                &self.map[i].ty,
                &self.data[self.map[i].offset] as *const u8,
                stack,
            )
        };
    }

    fn loada(&self, i: usize, stack: &mut EvalStack) {
        assert!(i < self.map.len());
        stack.push_managed(&self.data[self.map[i].offset] as *const u8 as *mut u8);
    }

    fn store(&mut self, i: usize, stack: &mut EvalStack) {
        assert!(i < self.map.len());
        if let BuiltinType::Value(ty) = self.map[i].ty {
            stack.pop(Some(TypedAddr {
                ty,
                addr: &mut self.data[self.map[i].offset] as *mut u8,
            }));
            return;
        }

        self.store_slot(i, stack.pop(None));
    }

    fn store_slot(&mut self, i: usize, slot: Slot) {
        assert!(i < self.map.len());
        unsafe {
            store_slot(
                &self.map[i].ty,
                &mut self.data[self.map[i].offset] as *mut u8,
                slot,
            )
        };
    }
}
pub struct Args<'m> {
    data: Vec<u8>,
    map: &'m Vec<Param>,
    has_self: bool,
}

impl<'m> Args<'m> {
    pub fn new(parent: &'m MethodDesc) -> Args<'m> {
        Args {
            data: vec![0; parent.ps_size],
            map: &parent.ps,
            has_self: !parent.is_static(),
        }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn get_self(&self) -> Option<*mut u8> {
        if self.has_self {
            Some(unsafe { *(&self.data[0] as *const u8 as *const *mut u8) })
        } else {
            return None;
        }
    }

    pub fn get_self_mut(&mut self) -> Option<&mut *mut u8> {
        if self.has_self {
            Some(unsafe {
                (&mut self.data[0] as *mut u8 as *mut *mut u8)
                    .as_mut()
                    .unwrap()
            })
        } else {
            return None;
        }
    }

    pub fn fill_args(&mut self, stack: &mut EvalStack) {
        self.fill_args_except_self(stack);
        if let Some(self_mut) = self.get_self_mut() {
            *self_mut = stack.pop(None).expect_ref_or_ptr();
        }
    }

    pub fn fill_args_except_self(&mut self, stack: &mut EvalStack) {
        let self_offset = if self.has_self { 1 } else { 0 };
        for i in (0..self.map.len()).rev() {
            self.store(i + self_offset, stack);
        }
    }

    fn align_arg(&self, i: usize) -> ArgType {
        let aligned = if self.has_self {
            if i == 0 {
                return ArgType::ArgSelf;
            }
            i - 1
        } else {
            i
        };

        if aligned >= self.map.len() {
            panic!("Index arg {} but only has {} args", aligned, self.map.len());
        }

        ArgType::MethodArg(aligned)
    }
}

enum ArgType {
    ArgSelf,
    MethodArg(usize),
}

impl<'m> ILocals for Args<'m> {
    /// if has self ptr, i == 0 means load self ptr
    fn load(&self, i: usize, stack: &mut EvalStack) {
        let i = if self.has_self {
            if i == 0 {
                // load self
                stack.push_ptr(unsafe { *(self.data.as_ptr() as *const *mut u8) });
                return;
            }
            i - 1
        } else {
            i
        };
        assert!(i < self.map.len());
        unsafe {
            load(
                &self.map[i].ty,
                &self.data[self.map[i].offset] as *const u8,
                stack,
            )
        };
    }

    fn loada(&self, i: usize, stack: &mut EvalStack) {
        let i = if self.has_self {
            if i == 0 {
                // load self
                stack.push_ptr(unsafe { *(self.data.as_ptr() as *const *mut u8) });
                return;
            }
            i - 1
        } else {
            i
        };
        assert!(i < self.map.len());
        stack.push_managed(&self.data[self.map[i].offset] as *const u8 as *mut u8);
    }

    /// if method is instance method, args starts at index 1, and i == 0 means self ptr,
    /// if method is static method, args starts at index 0
    fn store(&mut self, i: usize, stack: &mut EvalStack) {
        // same as Locals.store()
        let arg_i = self.align_arg(i);
        if let ArgType::MethodArg(i) = arg_i {
            if let BuiltinType::Value(ty) = self.map[i].ty {
                stack.pop(Some(TypedAddr {
                    ty,
                    addr: &mut self.data[self.map[i].offset] as *mut u8,
                }));
                return;
            }
        }

        self.store_slot(i, stack.pop(None));
    }

    /// if has self ptr, i == 0 means store self ptr
    fn store_slot(&mut self, i: usize, slot: Slot) {
        let arg_i = self.align_arg(i);
        match arg_i {
            ArgType::ArgSelf => {
                // store self
                unsafe {
                    *(&mut self.data[0] as *mut u8 as *mut *mut u8) = slot.expect_ref_or_ptr();
                }
            }
            ArgType::MethodArg(i) => {
                unsafe {
                    store_slot(
                        &self.map[i].ty,
                        &mut self.data[self.map[i].offset] as *mut u8,
                        slot,
                    )
                };
            }
        }
    }
}
