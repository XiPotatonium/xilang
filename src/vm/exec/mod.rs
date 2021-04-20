use super::data::{BuiltinType, Method, MethodILImpl, MethodImpl, MethodNativeImpl};
use super::mem::{addr_addu, to_relative, MemTag, SharedMem, Slot, SlotTag, Stack};

use xir::attrib::MethodAttribFlag;
use xir::tok::{get_tok_tag, TokTag};

use std::mem::transmute;

struct MethodState<'m> {
    ip: usize,
    method: &'m Method,
    method_impl: &'m MethodILImpl,
    stack: Stack,
    locals: Vec<Slot>,
    args: Vec<Slot>,
}

impl<'m> MethodState<'m> {
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
                SlotTag::F => panic!("Cannot add between float and int"),
                SlotTag::INative => {
                    $lhs.data.inative_ = $lhs.data.i32_ as isize $op $rhs.data.inative_;
                    $lhs.tag = SlotTag::INative;
                }
                SlotTag::Ref => panic!("Cannot add ref"),
                SlotTag::Uninit => panic!("Cannot add unint data"),
            },
            SlotTag::I64 => unimplemented!(),
            SlotTag::F => unimplemented!(),
            SlotTag::INative => match $rhs.tag {
                SlotTag::I32 => {
                    $lhs.data.inative_ = $lhs.data.inative_ $op $rhs.data.i32_ as isize;
                }
                SlotTag::I64 => panic!("Cannot add between i32 and i64"),
                SlotTag::F => panic!("Cannot add between float and int"),
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
                SlotTag::I64 => panic!("Cannot cmp between i32 and i64"),
                SlotTag::F => panic!("Cannot cmp between float and int"),
                SlotTag::INative => {
                    ($lhs.data.i32_ as isize) $op $rhs.data.inative_
                }
                SlotTag::Ref => panic!("Cannot cmp ref"),
                SlotTag::Uninit => panic!("Cannot cmp unint data"),
            },
            SlotTag::I64 => unimplemented!(),
            SlotTag::F => unimplemented!(),
            SlotTag::INative => match $rhs.tag {
                SlotTag::I32 => {
                    $lhs.data.inative_ $op $rhs.data.i32_ as isize
                }
                SlotTag::I64 => panic!("Cannot cmp between i32 and i64"),
                SlotTag::F => panic!("Cannot cmp between float and int"),
                SlotTag::INative => {
                    $lhs.data.inative_ $op $rhs.data.inative_
                }
                SlotTag::Ref => panic!("Cannot cmp ref"),
                SlotTag::Uninit => panic!("Cannot cmp unint data"),
            },
            SlotTag::Ref => panic!("Cannot cmp ref"),
            SlotTag::Uninit => panic!("Cannot cmp unint data"),
        }
    };
}

impl<'m> TExecutor<'m> {
    pub unsafe fn new(entry: *const Method) -> TExecutor<'m> {
        let mut ret = TExecutor { states: Vec::new() };
        // currently executor entry has no arguments
        let entry_ref = entry.as_ref().unwrap();
        ret.call(vec![], entry_ref, entry_ref.method_impl.expect_il());
        ret
    }

    unsafe fn call(&mut self, args: Vec<Slot>, method: &'m Method, il_impl: &'m MethodILImpl) {
        // Currently there is no verification of the arg type
        // TODO: Generate locals with type info set
        self.states.push(MethodState {
            stack: Stack::new(),
            locals: vec![Slot::default(); il_impl.locals.len()],
            args,
            ip: 0,
            method,
            method_impl: il_impl,
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
                    let tok = cur_state.consume_u32();
                    let ctx = cur_state.method.ctx.as_ref().unwrap().expect_il();

                    let (tag, idx) = get_tok_tag(tok);

                    let callee = match tag {
                        TokTag::MethodDef => ctx.methods[idx as usize - 1].as_ref(),
                        TokTag::MemberRef => ctx.memberref[idx as usize - 1]
                            .expect_method()
                            .as_ref()
                            .unwrap(),
                        _ => unimplemented!(),
                    };

                    let args = self.states.last_mut().unwrap().stack.pop_n(callee.ps.len());
                    match &callee.method_impl {
                        MethodImpl::IL(il_impl) => {
                            self.call(args, callee, il_impl);
                        }
                        MethodImpl::Native(MethodNativeImpl { scope, .. }) => {
                            // currently there is no multi-slot user defined type
                            let mut ret: Vec<Slot> = Vec::new();
                            let callee_ctx = callee.ctx.as_ref().unwrap().expect_il();
                            callee_ctx.modref[*scope]
                                .as_ref()
                                .unwrap()
                                .expect_dll()
                                .call(mem.get_str(callee.name), &args, &mut ret);
                            self.states.last_mut().unwrap().stack.append(ret);
                        }
                    }
                }
                // ret
                0x2A => {
                    let cur_state = self.states.last_mut().unwrap();
                    match cur_state.method.ret.ty {
                        BuiltinType::Void => {
                            self.states.pop();
                            if self.states.is_empty() {
                                return 0;
                            }
                        }
                        BuiltinType::Bool
                        | BuiltinType::Char
                        | BuiltinType::U1
                        | BuiltinType::I1
                        | BuiltinType::U2
                        | BuiltinType::I2
                        | BuiltinType::U4
                        | BuiltinType::I4
                        | BuiltinType::U8
                        | BuiltinType::I8
                        | BuiltinType::UNative
                        | BuiltinType::INative
                        | BuiltinType::R4
                        | BuiltinType::R8
                        | BuiltinType::ByRef(_)
                        | BuiltinType::Array(_) => {
                            let ret_v = cur_state.stack.pop();
                            self.states.pop();
                            if self.states.is_empty() {
                                return ret_v.as_u32();
                            }
                            self.states.last_mut().unwrap().stack.push(ret_v);
                        }
                        BuiltinType::Class(_) => unreachable!(),
                        BuiltinType::Unk => unreachable!(),
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
                // sub
                0x59 => {
                    let stack = &mut self.states.last_mut().unwrap().stack;
                    let rhs = stack.pop();
                    let lhs = stack.peek_mut();
                    exec_numeric_op!(-, lhs, rhs);
                }
                // mul
                0x5A => {
                    let stack = &mut self.states.last_mut().unwrap().stack;
                    let rhs = stack.pop();
                    let lhs = stack.peek_mut();
                    exec_numeric_op!(*, lhs, rhs);
                }
                // div
                0x5B => {
                    let stack = &mut self.states.last_mut().unwrap().stack;
                    let rhs = stack.pop();
                    let lhs = stack.peek_mut();
                    exec_numeric_op!(/, lhs, rhs);
                }
                // rem
                0x5D => {
                    let stack = &mut self.states.last_mut().unwrap().stack;
                    let rhs = stack.pop();
                    let lhs = stack.peek_mut();
                    exec_numeric_op!(%, lhs, rhs);
                }
                // neg
                0x65 => {
                    let lhs = self.states.last_mut().unwrap().stack.peek_mut();
                    match lhs.tag {
                        SlotTag::I32 => {
                            lhs.data.i32_ = -lhs.data.i32_;
                        }
                        SlotTag::I64 => {
                            lhs.data.i64_ = -lhs.data.i64_;
                        }
                        SlotTag::F => {
                            unimplemented!();
                        }
                        SlotTag::INative => {
                            lhs.data.inative_ = -lhs.data.inative_;
                        }
                        SlotTag::Ref => panic!("Cannot neg ref type"),
                        SlotTag::Uninit => panic!("Cannot neg uinit slot"),
                    }
                }
                // callvirt
                0x6F => {
                    let cur_state = self.states.last_mut().unwrap();
                    let tok = cur_state.consume_u32();
                    let ctx = cur_state.method.ctx.as_ref().unwrap().expect_il();

                    let (tag, idx) = get_tok_tag(tok);

                    let callee = match tag {
                        TokTag::MethodDef => ctx.methods[idx as usize - 1].as_ref(),
                        TokTag::MemberRef => ctx.memberref[idx as usize - 1]
                            .expect_method()
                            .as_ref()
                            .unwrap(),
                        _ => unimplemented!(),
                    };
                    // TODO: make sure callee is virtual

                    let args = cur_state.stack.pop_n(callee.ps.len() + 1);
                    self.call(args, callee, callee.method_impl.expect_il());
                }
                // newobj
                0x73 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let tok = cur_state.consume_u32();
                    let ctx = cur_state.method.ctx.as_ref().unwrap().expect_il();

                    let (tag, idx) = get_tok_tag(tok);

                    let callee = match tag {
                        TokTag::MethodDef => ctx.methods[idx as usize - 1].as_ref(),
                        TokTag::MemberRef => ctx.memberref[idx as usize - 1]
                            .expect_method()
                            .as_ref()
                            .unwrap(),
                        _ => unimplemented!(),
                    };
                    // TODO: make sure callee is .ctor

                    let mut args: Vec<Slot> = Vec::new();
                    if callee.flag.is(MethodAttribFlag::Static) {
                        panic!(".ctor should be a static method");
                    }
                    // TODO: more strict check

                    // Alloc space at heap
                    let offset = mem.heap.new_obj(callee.parent_class.unwrap());
                    args.push(Slot::new_ref(MemTag::HeapMem, offset));
                    args.append(&mut cur_state.stack.pop_n(callee.ps.len()));
                    cur_state.stack.push(Slot::new_ref(MemTag::HeapMem, offset));

                    self.call(args, callee, callee.method_impl.expect_il());
                }
                // ldfld
                0x7B => {
                    let cur_state = self.states.last_mut().unwrap();
                    let tok = cur_state.consume_u32();
                    let (mem_tag, offset) = cur_state.stack.pop().as_addr();
                    if let MemTag::HeapMem = mem_tag {
                    } else {
                        panic!("Operand of ldfld is not a heap addr");
                    }

                    let (tag, idx) = get_tok_tag(tok);
                    let f = match tag {
                        TokTag::Field => cur_state.method.ctx.as_ref().unwrap().expect_il().fields
                            [idx as usize - 1]
                            .as_ref(),
                        TokTag::MemberRef => {
                            cur_state.method.ctx.as_ref().unwrap().expect_il().memberref
                                [idx as usize - 1]
                                .expect_field()
                                .as_ref()
                                .unwrap()
                        }
                        _ => unimplemented!(),
                    };

                    let field_addr = addr_addu(f.addr, offset);

                    match f.ty {
                        BuiltinType::Void | BuiltinType::Unk | BuiltinType::Class(_) => {
                            unreachable!()
                        }
                        BuiltinType::Bool => unimplemented!(),
                        BuiltinType::Char => unimplemented!(),
                        BuiltinType::U1 => unimplemented!(),
                        BuiltinType::I1 => unimplemented!(),
                        BuiltinType::U2 => unimplemented!(),
                        BuiltinType::I2 => unimplemented!(),
                        BuiltinType::U4 => unimplemented!(),
                        BuiltinType::I4 => {
                            cur_state.stack.push_i32(*mem.heap.access(field_addr));
                        }
                        BuiltinType::U8 => unimplemented!(),
                        BuiltinType::I8 => unimplemented!(),
                        BuiltinType::UNative => unimplemented!(),
                        BuiltinType::INative => unimplemented!(),
                        BuiltinType::R4 => unimplemented!(),
                        BuiltinType::R8 => unimplemented!(),
                        BuiltinType::ByRef(_) => unimplemented!(),
                        BuiltinType::Array(_) => unimplemented!(),
                    }
                }
                // stfld
                0x7D => {
                    let cur_state = self.states.last_mut().unwrap();
                    let tok = cur_state.consume_u32();
                    let v = cur_state.stack.pop();
                    let (mem_tag, offset) = cur_state.stack.pop().as_addr();
                    if let MemTag::HeapMem = mem_tag {
                    } else {
                        panic!("Operand of stfld is not a heap addr");
                    }

                    let (tag, idx) = get_tok_tag(tok);
                    let f = match tag {
                        TokTag::Field => cur_state.method.ctx.as_ref().unwrap().expect_il().fields
                            [idx as usize - 1]
                            .as_ref(),
                        TokTag::MemberRef => {
                            cur_state.method.ctx.as_ref().unwrap().expect_il().memberref
                                [idx as usize - 1]
                                .expect_field()
                                .as_ref()
                                .unwrap()
                        }
                        _ => unimplemented!(),
                    };

                    let field_addr = addr_addu(f.addr, offset);

                    match f.ty {
                        BuiltinType::Void | BuiltinType::Unk | BuiltinType::Class(_) => {
                            unreachable!()
                        }
                        BuiltinType::Bool => unimplemented!(),
                        BuiltinType::Char => unimplemented!(),
                        BuiltinType::U1 => unimplemented!(),
                        BuiltinType::I1 => unimplemented!(),
                        BuiltinType::U2 => unimplemented!(),
                        BuiltinType::I2 => unimplemented!(),
                        BuiltinType::U4 => unimplemented!(),
                        BuiltinType::I4 => {
                            *mem.heap.access_mut(field_addr) = v.data.i32_;
                        }
                        BuiltinType::U8 => unimplemented!(),
                        BuiltinType::I8 => unimplemented!(),
                        BuiltinType::UNative => unimplemented!(),
                        BuiltinType::INative => unimplemented!(),
                        BuiltinType::R4 => unimplemented!(),
                        BuiltinType::R8 => unimplemented!(),
                        BuiltinType::ByRef(_) => unimplemented!(),
                        BuiltinType::Array(_) => unimplemented!(),
                    }
                }
                // ldsfld
                0x7E => {
                    let cur_state = self.states.last_mut().unwrap();
                    let tok = cur_state.consume_u32();

                    let (tag, idx) = get_tok_tag(tok);
                    let f = match tag {
                        TokTag::Field => cur_state.method.ctx.as_ref().unwrap().expect_il().fields
                            [idx as usize - 1]
                            .as_ref(),
                        TokTag::MemberRef => {
                            cur_state.method.ctx.as_ref().unwrap().expect_il().memberref
                                [idx as usize - 1]
                                .expect_field()
                                .as_ref()
                                .unwrap()
                        }
                        _ => unimplemented!(),
                    };

                    let (mem_tag, offset) = to_relative(f.addr);
                    if let MemTag::StaticMem = mem_tag {
                    } else {
                        panic!("Operand of ldsfld is not a static addr");
                    }

                    match f.ty {
                        BuiltinType::Void | BuiltinType::Unk | BuiltinType::Class(_) => {
                            unreachable!()
                        }
                        BuiltinType::Bool => unimplemented!(),
                        BuiltinType::Char => unimplemented!(),
                        BuiltinType::U1 => unimplemented!(),
                        BuiltinType::I1 => unimplemented!(),
                        BuiltinType::U2 => unimplemented!(),
                        BuiltinType::I2 => unimplemented!(),
                        BuiltinType::U4 => unimplemented!(),
                        BuiltinType::I4 => {
                            cur_state.stack.push_i32(*mem.heap.access(offset));
                        }
                        BuiltinType::U8 => unimplemented!(),
                        BuiltinType::I8 => unimplemented!(),
                        BuiltinType::UNative => unimplemented!(),
                        BuiltinType::INative => unimplemented!(),
                        BuiltinType::R4 => unimplemented!(),
                        BuiltinType::R8 => unimplemented!(),
                        BuiltinType::ByRef(_) => unimplemented!(),
                        BuiltinType::Array(_) => unimplemented!(),
                    }
                }
                // stsfld
                0x80 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let tok = cur_state.consume_u32();
                    let v = cur_state.stack.pop();

                    let (tag, idx) = get_tok_tag(tok);
                    let f = match tag {
                        TokTag::Field => cur_state.method.ctx.as_ref().unwrap().expect_il().fields
                            [idx as usize - 1]
                            .as_ref(),
                        TokTag::MemberRef => {
                            cur_state.method.ctx.as_ref().unwrap().expect_il().memberref
                                [idx as usize - 1]
                                .expect_field()
                                .as_ref()
                                .unwrap()
                        }
                        _ => unimplemented!(),
                    };

                    let (mem_tag, offset) = to_relative(f.addr);
                    if let MemTag::StaticMem = mem_tag {
                    } else {
                        panic!("Operand of stsfld is not a static addr");
                    }

                    match f.ty {
                        BuiltinType::Void | BuiltinType::Unk | BuiltinType::Class(_) => {
                            unreachable!()
                        }
                        BuiltinType::Bool => unimplemented!(),
                        BuiltinType::Char => unimplemented!(),
                        BuiltinType::U1 => unimplemented!(),
                        BuiltinType::I1 => unimplemented!(),
                        BuiltinType::U2 => unimplemented!(),
                        BuiltinType::I2 => unimplemented!(),
                        BuiltinType::U4 => unimplemented!(),
                        BuiltinType::I4 => {
                            *mem.heap.access_mut(offset) = v.data.i32_;
                        }
                        BuiltinType::U8 => unimplemented!(),
                        BuiltinType::I8 => unimplemented!(),
                        BuiltinType::UNative => unimplemented!(),
                        BuiltinType::INative => unimplemented!(),
                        BuiltinType::R4 => unimplemented!(),
                        BuiltinType::R8 => unimplemented!(),
                        BuiltinType::ByRef(_) => unimplemented!(),
                        BuiltinType::Array(_) => unimplemented!(),
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
