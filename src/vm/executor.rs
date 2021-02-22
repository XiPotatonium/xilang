use super::data::{VMMethod, VMType};
use super::mem::{SharedMem, Slot, SlotTag, Stack};
use crate::ir::inst::Inst;

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
    pub fn new(entry: &'m VMMethod) -> TExecutor<'m> {
        let mut ret = TExecutor { states: Vec::new() };
        // currently executor entry has no arguments
        ret.call(vec![], entry);
        ret
    }

    fn call(&mut self, args: Vec<Slot>, method: &'m VMMethod) {
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

    pub fn run(&mut self, mem: &mut SharedMem) -> u32 {
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
                Inst::Call(_) => {}
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
                            SlotTag::I32 => unsafe {
                                lhs.data.i32_ += rhs.data.i32_;
                            },
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
                Inst::CallVirt(_) => {}
                Inst::New(_) => {}
                Inst::LdFld(_) => {}
                Inst::StFld(_) => {}
                Inst::LdSFld(_) => {}
                Inst::StSFld(_) => {}
            }
        }
    }
}
