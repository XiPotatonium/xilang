use super::data::{VMField, VMMethod, VMType};
use super::mem::{to_relative, MemTag, SharedMem, Slot, SlotTag, Stack};
use crate::ir::inst::Inst;
use crate::ir::ir_file::{
    TBL_CLASSREF_TAG, TBL_CLASS_TAG, TBL_FIELD_TAG, TBL_MEMBERREF_TAG, TBL_METHOD_TAG, TBL_TAG_MASK,
};

struct MethodState<'m> {
    ip: usize,
    method: &'m VMMethod,
    stack: Stack,
    locals: Vec<Slot>,
    args: Vec<Slot>,
}

pub struct TExecutor<'m> {
    states: Vec<MethodState<'m>>,
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

    fn fetch(&mut self) -> &'m Inst {
        let state = self.states.last_mut().unwrap();
        state.ip += 1;
        &state.method.insts[state.ip - 1]
    }

    pub unsafe fn run(&mut self, mem: &'m mut SharedMem) -> u32 {
        loop {
            match self.fetch() {
                Inst::Nop => {}
                Inst::LdArg0 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.args[0]);
                }
                Inst::LdArg1 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.args[1]);
                }
                Inst::LdArg2 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.args[2]);
                }
                Inst::LdArg3 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.args[3]);
                }
                Inst::LdArgS(idx) => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.args[*idx as usize]);
                }
                Inst::StArgS(idx) => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.args[*idx as usize] = cur_state.stack.pop();
                }
                Inst::LdLoc0 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.locals[0]);
                }
                Inst::LdLoc1 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.locals[1]);
                }
                Inst::LdLoc2 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.locals[2]);
                }
                Inst::LdLoc3 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.locals[3]);
                }
                Inst::LdLocS(idx) => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.locals[*idx as usize]);
                }
                Inst::LdLoc(idx) => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.stack.push(cur_state.locals[*idx as usize]);
                }
                Inst::StLoc0 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals[0] = cur_state.stack.pop();
                }
                Inst::StLoc1 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals[1] = cur_state.stack.pop();
                }
                Inst::StLoc2 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals[2] = cur_state.stack.pop();
                }
                Inst::StLoc3 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals[3] = cur_state.stack.pop();
                }
                Inst::StLocS(idx) => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals[*idx as usize] = cur_state.stack.pop();
                }
                Inst::StLoc(idx) => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals[*idx as usize] = cur_state.stack.pop();
                }
                Inst::LdNull => {
                    self.states.last_mut().unwrap().stack.push(Slot::null());
                }
                Inst::LdCM1 => {
                    self.states.last_mut().unwrap().stack.push_i32(-1);
                }
                Inst::LdC0 => {
                    self.states.last_mut().unwrap().stack.push_i32(0);
                }
                Inst::LdC1 => {
                    self.states.last_mut().unwrap().stack.push_i32(1);
                }
                Inst::LdC2 => {
                    self.states.last_mut().unwrap().stack.push_i32(2);
                }
                Inst::LdC3 => {
                    self.states.last_mut().unwrap().stack.push_i32(3);
                }
                Inst::LdC4 => {
                    self.states.last_mut().unwrap().stack.push_i32(4);
                }
                Inst::LdC5 => {
                    self.states.last_mut().unwrap().stack.push_i32(5);
                }
                Inst::LdC6 => {
                    self.states.last_mut().unwrap().stack.push_i32(6);
                }
                Inst::LdC7 => {
                    self.states.last_mut().unwrap().stack.push_i32(7);
                }
                Inst::LdC8 => {
                    self.states.last_mut().unwrap().stack.push_i32(8);
                }
                Inst::LdCI4S(v) => {
                    self.states.last_mut().unwrap().stack.push_i32(*v as i32);
                }
                Inst::LdCI4(v) => {
                    self.states.last_mut().unwrap().stack.push_i32(*v);
                }
                Inst::Dup => {
                    self.states.last_mut().unwrap().stack.dup();
                }
                Inst::Pop => {
                    self.states.last_mut().unwrap().stack.pop();
                }
                Inst::Call(idx) => {
                    let tag = *idx & TBL_TAG_MASK;
                    let idx = (*idx & !TBL_TAG_MASK) as usize - 1;
                    let ctx = self.states.last().unwrap().method.ctx.as_ref().unwrap();

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

                    self.call(self.states.last().unwrap().stack.clone_top(arg_len), callee);
                }
                Inst::Ret => {
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
                Inst::Add => {
                    let stack = &mut self.states.last_mut().unwrap().stack;
                    let rhs = stack.pop();
                    let lhs = stack.peek_mut();
                    match lhs.tag {
                        SlotTag::I32 => match rhs.tag {
                            SlotTag::I32 => {
                                lhs.data.i32_ += rhs.data.i32_;
                            }
                            SlotTag::I64 => unimplemented!(),
                            SlotTag::F64 => panic!("Cannot add between float and int"),
                            SlotTag::Ref => panic!("Cannot add ref"),
                            SlotTag::Uninit => panic!("Cannot add unint data"),
                        },
                        SlotTag::I64 => unimplemented!(),
                        SlotTag::F64 => unimplemented!(),
                        SlotTag::Ref => panic!("Cannot add ref"),
                        SlotTag::Uninit => panic!("Cannot add unint data"),
                    }
                }
                Inst::CallVirt(idx) => {
                    let tag = *idx & TBL_TAG_MASK;
                    let idx = (*idx & !TBL_TAG_MASK) as usize - 1;
                    let ctx = self.states.last().unwrap().method.ctx.as_ref().unwrap();

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

                    self.call(self.states.last().unwrap().stack.clone_top(arg_len), callee);
                }
                Inst::NewObj(idx) => {
                    let tag = *idx & TBL_TAG_MASK;
                    let idx = (*idx & !TBL_TAG_MASK) as usize - 1;
                    let cur_state = self.states.last_mut().unwrap();

                    let c = match tag {
                        TBL_CLASS_TAG => {
                            cur_state.method.ctx.as_ref().unwrap().classes[idx].as_ref()
                        }
                        TBL_CLASSREF_TAG => cur_state.method.ctx.as_ref().unwrap().classref[idx]
                            .as_ref()
                            .unwrap(),
                        _ => unreachable!(),
                    };

                    let offset = mem.heap.new_obj(c.obj_size);
                }
                Inst::LdFld(idx) => {
                    let tag = *idx & TBL_TAG_MASK;
                    let idx = (*idx & !TBL_TAG_MASK) as usize - 1;
                    let cur_state = self.states.last_mut().unwrap();
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
                Inst::StFld(idx) => {
                    let tag = *idx & TBL_TAG_MASK;
                    let idx = (*idx & !TBL_TAG_MASK) as usize - 1;
                    let cur_state = self.states.last_mut().unwrap();
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
                Inst::LdSFld(idx) => {
                    let tag = *idx & TBL_TAG_MASK;
                    let idx = (*idx & !TBL_TAG_MASK) as usize - 1;
                    let cur_state = self.states.last_mut().unwrap();

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
                Inst::StSFld(idx) => {
                    let tag = *idx & TBL_TAG_MASK;
                    let idx = (*idx & !TBL_TAG_MASK) as usize - 1;
                    let cur_state = self.states.last_mut().unwrap();
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
            }
        }
    }
}
