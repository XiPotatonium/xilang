use super::super::data::{Method, MethodILImpl};
use super::{Args, EvalStack, Locals, Slot};

use std::mem::transmute;

pub struct ActivationRecord<'m> {
    pub ip: usize,
    pub ret_addr: *mut Slot,
    pub method: &'m Method,
    pub method_impl: &'m MethodILImpl,
    pub eval_stack: EvalStack,
    pub locals: Locals<'m>,
    pub args: Args<'m>,
}

impl<'m> ActivationRecord<'m> {
    pub fn consume_u8(&mut self) -> u8 {
        self.ip += 1;
        self.method_impl.insts[self.ip - 1]
    }

    pub fn consume_u16(&mut self) -> u16 {
        self.ip += 2;
        ((self.method_impl.insts[self.ip - 2] as u16) << 8)
            + (self.method_impl.insts[self.ip - 1] as u16)
    }

    pub fn consume_u32(&mut self) -> u32 {
        self.ip += 4;
        ((self.method_impl.insts[self.ip - 4] as u32) << 24)
            + ((self.method_impl.insts[self.ip - 3] as u32) << 16)
            + ((self.method_impl.insts[self.ip - 2] as u32) << 8)
            + (self.method_impl.insts[self.ip - 1] as u32)
    }

    pub fn consume_i8(&mut self) -> i8 {
        self.ip += 1;
        unsafe { transmute(self.method_impl.insts[self.ip - 1]) }
    }

    pub fn consume_i32(&mut self) -> i32 {
        self.ip += 4;
        let bytes = [
            self.method_impl.insts[self.ip - 4],
            self.method_impl.insts[self.ip - 3],
            self.method_impl.insts[self.ip - 2],
            self.method_impl.insts[self.ip - 1],
        ];
        i32::from_be_bytes(bytes)
    }
}
