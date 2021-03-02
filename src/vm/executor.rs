use super::data::{VMMethod, VMType};
use super::mem::{to_relative, MemTag, SharedMem, Slot, SlotTag, Stack};

use crate::ir::inst::Inst;
use crate::ir::ir_file::{TBL_FIELD_TAG, TBL_MEMBERREF_TAG, TBL_METHOD_TAG, TBL_TAG_MASK};

use std::mem::transmute;

struct MethodState<'m> {
    ip: usize,
    method: &'m VMMethod,
    stack: Stack,
    locals: Vec<Slot>,
    args: Vec<Slot>,
}

impl<'m> MethodState<'m> {
    pub fn consume_u8(&mut self) -> u8 {
        self.ip += 1;
        self.method.insts[self.ip - 1]
    }

    pub fn consume_u16(&mut self) -> u16 {
        self.ip += 2;
        ((self.method.insts[self.ip - 2] as u16) << 8) + (self.method.insts[self.ip - 1] as u16)
    }

    pub fn consume_u32(&mut self) -> u32 {
        self.ip += 4;
        ((self.method.insts[self.ip - 4] as u32) << 24)
            + ((self.method.insts[self.ip - 3] as u32) << 16)
            + ((self.method.insts[self.ip - 2] as u32) << 8)
            + (self.method.insts[self.ip - 1] as u32)
    }

    pub fn consume_i8(&mut self) -> i8 {
        self.ip += 1;
        unsafe { transmute(self.method.insts[self.ip - 1]) }
    }

    pub fn consume_i32(&mut self) -> i32 {
        self.ip += 4;
        let bytes = [
            self.method.insts[self.ip - 4],
            self.method.insts[self.ip - 3],
            self.method.insts[self.ip - 2],
            self.method.insts[self.ip - 1],
        ];
        i32::from_be_bytes(bytes)
    }
}

pub struct TExecutor<'m> {
    states: Vec<MethodState<'m>>,
}

macro_rules! exec_numeric_op {
    ($op: tt, $lhs: ident, $rhs: ident) => {
        match $lhs.tag {
            SlotTag::I32 => match $rhs.tag {
                SlotTag::I32 => {
                    $lhs.data.i32_ = $lhs.data.i32_ $op $rhs.data.i32_;
                }
                SlotTag::I64 => panic!("Cannot add between i32 and i64"),
                SlotTag::F64 => panic!("Cannot add between float and int"),
                SlotTag::INative => {
                    $lhs.data.inative_ = $lhs.data.i32_ as isize $op $rhs.data.inative_;
                    $lhs.tag = SlotTag::INative;
                }
                SlotTag::Ref => panic!("Cannot add ref"),
                SlotTag::Uninit => panic!("Cannot add unint data"),
            },
            SlotTag::I64 => unimplemented!(),
            SlotTag::F64 => unimplemented!(),
            SlotTag::INative => match $rhs.tag {
                SlotTag::I32 => {
                    $lhs.data.inative_ = $lhs.data.inative_ $op $rhs.data.i32_ as isize;
                }
                SlotTag::I64 => panic!("Cannot add between i32 and i64"),
                SlotTag::F64 => panic!("Cannot add between float and int"),
                SlotTag::INative => {
                    $lhs.data.inative_ = $lhs.data.inative_ $op $rhs.data.inative_;
                }
                SlotTag::Ref => panic!("Cannot add ref"),
                SlotTag::Uninit => panic!("Cannot add unint data"),
            },
            SlotTag::Ref => panic!("Cannot add ref"),
            SlotTag::Uninit => panic!("Cannot add unint data"),
        }
    };
}

macro_rules! exec_cmp_op {
    ($op: tt, $lhs: ident, $rhs: ident) => {
        match $lhs.tag {
            SlotTag::I32 => match $rhs.tag {
                SlotTag::I32 => {
                    $lhs.data.i32_ $op $rhs.data.i32_
                }
                SlotTag::I64 => panic!("Cannot add between i32 and i64"),
                SlotTag::F64 => panic!("Cannot add between float and int"),
                SlotTag::INative => {
                    ($lhs.data.i32_ as isize) $op $rhs.data.inative_
                }
                SlotTag::Ref => panic!("Cannot add ref"),
                SlotTag::Uninit => panic!("Cannot add unint data"),
            },
            SlotTag::I64 => unimplemented!(),
            SlotTag::F64 => unimplemented!(),
            SlotTag::INative => match $rhs.tag {
                SlotTag::I32 => {
                    $lhs.data.inative_ $op $rhs.data.i32_ as isize
                }
                SlotTag::I64 => panic!("Cannot add between i32 and i64"),
                SlotTag::F64 => panic!("Cannot add between float and int"),
                SlotTag::INative => {
                    $lhs.data.inative_ $op $rhs.data.inative_
                }
                SlotTag::Ref => panic!("Cannot add ref"),
                SlotTag::Uninit => panic!("Cannot add unint data"),
            },
            SlotTag::Ref => panic!("Cannot add ref"),
            SlotTag::Uninit => panic!("Cannot add unint data"),
        }
    };
}

impl<'m> TExecutor<'m> {
    pub unsafe fn new(entry: *const VMMethod) -> TExecutor<'m> {
        let mut ret = TExecutor { states: Vec::new() };
        // currently executor entry has no arguments
        ret.call(vec![], entry);
        ret
    }

    unsafe fn call(&mut self, args: Vec<Slot>, method: *const VMMethod) {
        // Currently there is no verification of the arg type
        let method = method.as_ref().unwrap();
        self.states.push(MethodState {
            stack: Stack::new(),
            locals: vec![Slot::default(); method.locals],
            args,
            ip: 0,
            method,
        });
    }

    pub unsafe fn run(&mut self, mem: &'m mut SharedMem) -> u32 {
        loop {
            let code = self.states.last_mut().unwrap().consume_u8();
            match code {
                // nop
                0x00 => {}
                // ldarg0
                0x02 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.args[0]);
                }
                // ldarg1
                0x03 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.args[1]);
                }
                // ldarg2
                0x04 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.args[2]);
                }
                // ldarg3
                0x05 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.args[3]);
                }
                // ldloc0
                0x06 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.locals[0]);
                }
                // ldloc1
                0x07 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.locals[1]);
                }
                // ldloc2
                0x08 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.locals[2]);
                }
                // ldloc3
                0x09 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.locals[3]);
                }
                // stloc0
                0x0A => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals[0] = cur_state.stack.pop();
                }
                // stloc1
                0x0B => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals[1] = cur_state.stack.pop();
                }
                // stloc2
                0x0C => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals[2] = cur_state.stack.pop();
                }
                // stloc3
                0x0D => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals[3] = cur_state.stack.pop();
                }
                // ldarg.s
                0x0E => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u8();
                    cur_state.stack.push(cur_state.args[idx as usize]);
                }
                // starg.s
                0x10 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u8();
                    cur_state.args[idx as usize] = cur_state.stack.pop();
                }
                // ldloc.s
                0x11 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u8();
                    cur_state.stack.push(cur_state.locals[idx as usize]);
                }
                // stloc.s
                0x13 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u8();
                    cur_state.locals[idx as usize] = cur_state.stack.pop();
                }
                // ldnull
                0x14 => {
                    self.states.last_mut().unwrap().stack.push(Slot::null());
                }
                // ldc.i4.m1
                0x15 => {
                    self.states.last_mut().unwrap().stack.push_i32(-1);
                }
                // ldc.i4.0
                0x16 => {
                    self.states.last_mut().unwrap().stack.push_i32(0);
                }
                // ldc.i4.1
                0x17 => {
                    self.states.last_mut().unwrap().stack.push_i32(1);
                }
                // ldc.i4.2
                0x18 => {
                    self.states.last_mut().unwrap().stack.push_i32(2);
                }
                // ldc.i4.3
                0x19 => {
                    self.states.last_mut().unwrap().stack.push_i32(3);
                }
                // ldc.i4.4
                0x1A => {
                    self.states.last_mut().unwrap().stack.push_i32(4);
                }
                // ldc.i4.5
                0x1B => {
                    self.states.last_mut().unwrap().stack.push_i32(5);
                }
                // ldc.i4.6
                0x1C => {
                    self.states.last_mut().unwrap().stack.push_i32(6);
                }
                // ldc.i4.7
                0x1D => {
                    self.states.last_mut().unwrap().stack.push_i32(7);
                }
                // ldc.i4.8
                0x1E => {
                    self.states.last_mut().unwrap().stack.push_i32(8);
                }
                // ldc.i4.s
                0x1F => {
                    let cur_state = self.states.last_mut().unwrap();
                    let v = cur_state.consume_i8();
                    cur_state.stack.push_i32(v as i32);
                }
                // ldc.i4
                0x20 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let v = cur_state.consume_i32();
                    cur_state.stack.push_i32(v);
                }
                // dup
                0x25 => {
                    self.states.last_mut().unwrap().stack.dup();
                }
                // pop
                0x26 => {
                    self.states.last_mut().unwrap().stack.pop();
                }
                // call
                0x28 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u32();
                    let tag = idx & TBL_TAG_MASK;
                    let idx = (idx & !TBL_TAG_MASK) as usize - 1;
                    let ctx = cur_state.method.ctx.as_ref().unwrap();

                    let (arg_len, callee) = match tag {
                        TBL_METHOD_TAG => (
                            ctx.methods[idx].ps_ty.len(),
                            ctx.methods[idx].as_ref() as *const VMMethod,
                        ),
                        TBL_MEMBERREF_TAG => {
                            let callee = ctx.memberref[idx].expect_method();
                            (callee.as_ref().unwrap().ps_ty.len(), callee)
                        }
                        _ => unreachable!(),
                    };

                    let args = self.states.last_mut().unwrap().stack.pop_n(arg_len);
                    self.call(args, callee);
                }
                // ret
                0x2A => {
                    let cur_state = self.states.last_mut().unwrap();
                    match cur_state.method.ret_ty {
                        VMType::Void => {
                            self.states.pop();
                            if self.states.is_empty() {
                                return 0;
                            }
                        }
                        VMType::Bool
                        | VMType::Char
                        | VMType::U8
                        | VMType::I8
                        | VMType::U16
                        | VMType::I16
                        | VMType::U32
                        | VMType::I32
                        | VMType::U64
                        | VMType::I64
                        | VMType::UNative
                        | VMType::INative
                        | VMType::F32
                        | VMType::F64
                        | VMType::Obj(_)
                        | VMType::Array(_) => {
                            let ret_v = cur_state.stack.pop();
                            self.states.pop();
                            if self.states.is_empty() {
                                return ret_v.as_u32();
                            }
                            self.states.last_mut().unwrap().stack.push(ret_v);
                        }
                        VMType::Unk => unreachable!(),
                    }
                }
                // br
                0x38 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let offset = cur_state.consume_i32();
                    cur_state.ip = (cur_state.ip as i32 + offset) as usize;
                }
                // brfalse
                0x39 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let offset = cur_state.consume_i32();
                    let v = cur_state.stack.pop();
                    if v.data.inative_ == 0 {
                        // false
                        cur_state.ip = (cur_state.ip as i32 + offset) as usize;
                    }
                }
                // brtrue
                0x3A => {
                    let cur_state = self.states.last_mut().unwrap();
                    let offset = cur_state.consume_i32();
                    let v = cur_state.stack.pop();
                    if v.data.inative_ != 0 {
                        // true
                        cur_state.ip = (cur_state.ip as i32 + offset) as usize;
                    }
                }
                // beq
                0x3B => {
                    let cur_state = self.states.last_mut().unwrap();
                    let offset = cur_state.consume_i32();
                    let rhs = cur_state.stack.pop();
                    let lhs = cur_state.stack.pop();
                    let b = exec_cmp_op!(==, lhs, rhs);
                    if b {
                        cur_state.ip = (cur_state.ip as i32 + offset) as usize;
                    }
                }
                // bge
                0x3C => {
                    let cur_state = self.states.last_mut().unwrap();
                    let offset = cur_state.consume_i32();
                    let rhs = cur_state.stack.pop();
                    let lhs = cur_state.stack.pop();
                    let b = exec_cmp_op!(>=, lhs, rhs);
                    if b {
                        cur_state.ip = (cur_state.ip as i32 + offset) as usize;
                    }
                }
                // bgt
                0x3D => {
                    let cur_state = self.states.last_mut().unwrap();
                    let offset = cur_state.consume_i32();
                    let rhs = cur_state.stack.pop();
                    let lhs = cur_state.stack.pop();
                    let b = exec_cmp_op!(>, lhs, rhs);
                    if b {
                        cur_state.ip = (cur_state.ip as i32 + offset) as usize;
                    }
                }
                // ble
                0x3E => {
                    let cur_state = self.states.last_mut().unwrap();
                    let offset = cur_state.consume_i32();
                    let rhs = cur_state.stack.pop();
                    let lhs = cur_state.stack.pop();
                    let b = exec_cmp_op!(<=, lhs, rhs);
                    if b {
                        cur_state.ip = (cur_state.ip as i32 + offset) as usize;
                    }
                }
                // blt
                0x3F => {
                    let cur_state = self.states.last_mut().unwrap();
                    let offset = cur_state.consume_i32();
                    let rhs = cur_state.stack.pop();
                    let lhs = cur_state.stack.pop();
                    let b = exec_cmp_op!(<, lhs, rhs);
                    if b {
                        cur_state.ip = (cur_state.ip as i32 + offset) as usize;
                    }
                }
                // add
                0x58 => {
                    let stack = &mut self.states.last_mut().unwrap().stack;
                    let rhs = stack.pop();
                    let lhs = stack.peek_mut();
                    exec_numeric_op!(+, lhs, rhs);
                }
                // rem
                0x5D => {
                    let stack = &mut self.states.last_mut().unwrap().stack;
                    let rhs = stack.pop();
                    let lhs = stack.peek_mut();
                    exec_numeric_op!(%, lhs, rhs);
                }
                // callvirt
                0x6F => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u32();
                    let tag = idx & TBL_TAG_MASK;
                    let idx = (idx & !TBL_TAG_MASK) as usize - 1;
                    let ctx = cur_state.method.ctx.as_ref().unwrap();

                    let (arg_len, callee) = match tag {
                        TBL_METHOD_TAG => (
                            ctx.methods[idx].ps_ty.len(),
                            ctx.methods[idx].as_ref() as *const VMMethod,
                        ),
                        TBL_MEMBERREF_TAG => {
                            let callee = ctx.memberref[idx].expect_method();
                            (callee.as_ref().unwrap().ps_ty.len(), callee)
                        }
                        _ => unreachable!(),
                    };

                    if callee.as_ref().unwrap().offset != 0 {
                        // virtual method
                        unimplemented!("Calling a virtual method is not implemented");
                    }

                    let args = cur_state.stack.pop_n(arg_len + 1);
                    self.call(args, callee);
                }
                // newobj
                0x73 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u32();
                    let tag = idx & TBL_TAG_MASK;
                    let idx = (idx & !TBL_TAG_MASK) as usize - 1;
                    let ctx = cur_state.method.ctx.as_ref().unwrap();

                    let (arg_len, callee) = match tag {
                        TBL_METHOD_TAG => (
                            ctx.methods[idx].ps_ty.len(),
                            ctx.methods[idx].as_ref() as *const VMMethod,
                        ),
                        TBL_MEMBERREF_TAG => {
                            let callee = ctx.memberref[idx].expect_method();
                            (callee.as_ref().unwrap().ps_ty.len(), callee)
                        }
                        _ => unreachable!(),
                    };

                    let mut args: Vec<Slot> = Vec::new();
                    if let VMType::Obj(class) = &callee.as_ref().unwrap().ps_ty[0] {
                        let offset = mem.heap.new_obj(*class);
                        args.push(Slot::new_ref(MemTag::HeapMem, offset));
                        args.append(&mut cur_state.stack.pop_n(arg_len - 1));
                        cur_state.stack.push(Slot::new_ref(MemTag::HeapMem, offset));
                    } else {
                        panic!("Creator's first param is not a class type");
                    }

                    self.call(args, callee);
                }
                // ldfld
                0x7B => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u32();
                    let tag = idx & TBL_TAG_MASK;
                    let idx = (idx & !TBL_TAG_MASK) as usize - 1;
                    let (mem_tag, offset) = cur_state.stack.pop().as_addr();
                    if let MemTag::HeapMem = mem_tag {
                    } else {
                        panic!("Operand of ldfld is not a heap addr");
                    }

                    let f = match tag {
                        TBL_FIELD_TAG => {
                            cur_state.method.ctx.as_ref().unwrap().fields[idx].as_ref()
                        }
                        TBL_MEMBERREF_TAG => cur_state.method.ctx.as_ref().unwrap().memberref[idx]
                            .expect_field()
                            .as_ref()
                            .unwrap(),
                        _ => unreachable!(),
                    };

                    let offset = offset + f.addr;

                    match f.ty {
                        VMType::Void | VMType::Unk => unreachable!(),
                        VMType::Bool => unimplemented!(),
                        VMType::Char => unimplemented!(),
                        VMType::U8 => unimplemented!(),
                        VMType::I8 => unimplemented!(),
                        VMType::U16 => unimplemented!(),
                        VMType::I16 => unimplemented!(),
                        VMType::U32 => unimplemented!(),
                        VMType::I32 => {
                            cur_state.stack.push_i32(*mem.heap.access(offset));
                        }
                        VMType::U64 => unimplemented!(),
                        VMType::I64 => unimplemented!(),
                        VMType::UNative => unimplemented!(),
                        VMType::INative => unimplemented!(),
                        VMType::F32 => unimplemented!(),
                        VMType::F64 => unimplemented!(),
                        VMType::Obj(_) => unimplemented!(),
                        VMType::Array(_) => unimplemented!(),
                    }
                }
                // stfld
                0x7D => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u32();
                    let tag = idx & TBL_TAG_MASK;
                    let idx = (idx & !TBL_TAG_MASK) as usize - 1;
                    let v = cur_state.stack.pop();
                    let (mem_tag, offset) = cur_state.stack.pop().as_addr();
                    if let MemTag::HeapMem = mem_tag {
                    } else {
                        panic!("Operand of stfld is not a heap addr");
                    }

                    let f = match tag {
                        TBL_FIELD_TAG => {
                            cur_state.method.ctx.as_ref().unwrap().fields[idx].as_ref()
                        }
                        TBL_MEMBERREF_TAG => cur_state.method.ctx.as_ref().unwrap().memberref[idx]
                            .expect_field()
                            .as_ref()
                            .unwrap(),
                        _ => unreachable!(),
                    };

                    let offset = offset + f.addr;

                    match f.ty {
                        VMType::Void | VMType::Unk => unreachable!(),
                        VMType::Bool => unimplemented!(),
                        VMType::Char => unimplemented!(),
                        VMType::U8 => unimplemented!(),
                        VMType::I8 => unimplemented!(),
                        VMType::U16 => unimplemented!(),
                        VMType::I16 => unimplemented!(),
                        VMType::U32 => unimplemented!(),
                        VMType::I32 => {
                            *mem.heap.access_mut(offset) = v.data.i32_;
                        }
                        VMType::U64 => unimplemented!(),
                        VMType::I64 => unimplemented!(),
                        VMType::UNative => unimplemented!(),
                        VMType::INative => unimplemented!(),
                        VMType::F32 => unimplemented!(),
                        VMType::F64 => unimplemented!(),
                        VMType::Obj(_) => unimplemented!(),
                        VMType::Array(_) => unimplemented!(),
                    }
                }
                // ldsfld
                0x7E => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u32();
                    let tag = idx & TBL_TAG_MASK;
                    let idx = (idx & !TBL_TAG_MASK) as usize - 1;

                    let f = match tag {
                        TBL_FIELD_TAG => {
                            cur_state.method.ctx.as_ref().unwrap().fields[idx].as_ref()
                        }
                        TBL_MEMBERREF_TAG => cur_state.method.ctx.as_ref().unwrap().memberref[idx]
                            .expect_field()
                            .as_ref()
                            .unwrap(),
                        _ => unreachable!(),
                    };

                    let (mem_tag, offset) = to_relative(f.addr);
                    if let MemTag::StaticMem = mem_tag {
                    } else {
                        panic!("Operand of ldsfld is not a static addr");
                    }

                    match f.ty {
                        VMType::Void | VMType::Unk => unreachable!(),
                        VMType::Bool => unimplemented!(),
                        VMType::Char => unimplemented!(),
                        VMType::U8 => unimplemented!(),
                        VMType::I8 => unimplemented!(),
                        VMType::U16 => unimplemented!(),
                        VMType::I16 => unimplemented!(),
                        VMType::U32 => unimplemented!(),
                        VMType::I32 => {
                            cur_state.stack.push_i32(*mem.heap.access(offset));
                        }
                        VMType::U64 => unimplemented!(),
                        VMType::I64 => unimplemented!(),
                        VMType::UNative => unimplemented!(),
                        VMType::INative => unimplemented!(),
                        VMType::F32 => unimplemented!(),
                        VMType::F64 => unimplemented!(),
                        VMType::Obj(_) => unimplemented!(),
                        VMType::Array(_) => unimplemented!(),
                    }
                }
                // stfld
                0x80 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u32();
                    let tag = idx & TBL_TAG_MASK;
                    let idx = (idx & !TBL_TAG_MASK) as usize - 1;
                    let v = cur_state.stack.pop();

                    let f = match tag {
                        TBL_FIELD_TAG => {
                            cur_state.method.ctx.as_ref().unwrap().fields[idx].as_ref()
                        }
                        TBL_MEMBERREF_TAG => cur_state.method.ctx.as_ref().unwrap().memberref[idx]
                            .expect_field()
                            .as_ref()
                            .unwrap(),
                        _ => unreachable!(),
                    };

                    let (mem_tag, offset) = to_relative(f.addr);
                    if let MemTag::StaticMem = mem_tag {
                    } else {
                        panic!("Operand of stsfld is not a static addr");
                    }

                    match f.ty {
                        VMType::Void | VMType::Unk => unreachable!(),
                        VMType::Bool => unimplemented!(),
                        VMType::Char => unimplemented!(),
                        VMType::U8 => unimplemented!(),
                        VMType::I8 => unimplemented!(),
                        VMType::U16 => unimplemented!(),
                        VMType::I16 => unimplemented!(),
                        VMType::U32 => unimplemented!(),
                        VMType::I32 => {
                            *mem.heap.access_mut(offset) = v.data.i32_;
                        }
                        VMType::U64 => unimplemented!(),
                        VMType::I64 => unimplemented!(),
                        VMType::UNative => unimplemented!(),
                        VMType::INative => unimplemented!(),
                        VMType::F32 => unimplemented!(),
                        VMType::F64 => unimplemented!(),
                        VMType::Obj(_) => unimplemented!(),
                        VMType::Array(_) => unimplemented!(),
                    }
                }

                0xFE => {
                    let inner_code = self.states.last_mut().unwrap().consume_u8();
                    match inner_code {
                        // ceq
                        0x01 => {
                            let cur_state = self.states.last_mut().unwrap();
                            let rhs = cur_state.stack.pop();
                            let lhs = cur_state.stack.peek_mut();
                            let t = exec_cmp_op!(==, lhs, rhs);
                            lhs.data.inative_ = if t { 1 } else { 0 };
                            lhs.tag = SlotTag::INative;
                        }
                        // cgt
                        0x02 => {
                            let cur_state = self.states.last_mut().unwrap();
                            let rhs = cur_state.stack.pop();
                            let lhs = cur_state.stack.peek_mut();
                            let t = exec_cmp_op!(>, lhs, rhs);
                            lhs.data.inative_ = if t { 1 } else { 0 };
                            lhs.tag = SlotTag::INative;
                        }
                        // clt
                        0x04 => {
                            let cur_state = self.states.last_mut().unwrap();
                            let rhs = cur_state.stack.pop();
                            let lhs = cur_state.stack.peek_mut();
                            let t = exec_cmp_op!(<, lhs, rhs);
                            lhs.data.inative_ = if t { 1 } else { 0 };
                            lhs.tag = SlotTag::INative;
                        }
                        // ldloc
                        0x0C => {
                            let cur_state = self.states.last_mut().unwrap();
                            let idx = cur_state.consume_u16();
                            cur_state.stack.push(cur_state.locals[idx as usize]);
                        }
                        // stloc
                        0x0E => {
                            let cur_state = self.states.last_mut().unwrap();
                            let idx = cur_state.consume_u16();
                            cur_state.locals[idx as usize] = cur_state.stack.pop();
                        }
                        _ => panic!("Unknown inst 0xFE{:X}", inner_code),
                    }
                }

                _ => panic!("Unknown inst: 0x{:X}", code),
            }
        }
    }
}
