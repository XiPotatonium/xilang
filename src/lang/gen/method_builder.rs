use xir::inst::Inst;

use std::mem;

use super::basic_block::{BasicBlock, LLCursor, LinkedList};

pub struct MethodBuilder {
    pub bb: LinkedList<BasicBlock>,
    cur_bb: LLCursor<BasicBlock>,
}

impl MethodBuilder {
    pub fn new() -> MethodBuilder {
        let mut bb = LinkedList::new();
        bb.push_back(BasicBlock::new());
        let cur_bb = bb.cursor_back_mut();

        MethodBuilder { bb, cur_bb }
    }

    pub fn insert_after_cur(&mut self) -> LLCursor<BasicBlock> {
        self.bb
            .insert_after_cursor(&mut self.cur_bb, BasicBlock::new())
    }

    pub fn set_cur_bb(&mut self, cur_bb: LLCursor<BasicBlock>) -> LLCursor<BasicBlock> {
        let mut cur_bb = cur_bb;
        mem::swap(&mut cur_bb, &mut self.cur_bb);
        cur_bb
    }

    pub fn cur_bb_last_is_branch(&self) -> bool {
        if let Some(inst) = self.cur_bb.as_ref().unwrap().insts.last() {
            match inst {
                Inst::BEq(_)
                | Inst::BGe(_)
                | Inst::BGt(_)
                | Inst::BLe(_)
                | Inst::BLt(_)
                | Inst::Br(_)
                | Inst::BrFalse(_)
                | Inst::BrTrue(_)
                | Inst::Ret => true,
                _ => false,
            }
        } else {
            false
        }
    }
}

impl MethodBuilder {
    pub fn add_inst(&mut self, inst: Inst) -> &mut Self {
        self.cur_bb.as_mut().unwrap().push(inst);
        self
    }

    pub fn add_brfalse(&mut self, target: LLCursor<BasicBlock>) -> &mut Self {
        let cur_bb = self.cur_bb.as_mut().unwrap();
        cur_bb.push(Inst::BrFalse(0));
        if let Some(_) = cur_bb.target {
            unreachable!();
        } else {
            cur_bb.target = Some(target);
        }
        self
    }

    pub fn add_brtrue(&mut self, target: LLCursor<BasicBlock>) -> &mut Self {
        let cur_bb = self.cur_bb.as_mut().unwrap();
        cur_bb.push(Inst::BrTrue(0));
        if let Some(_) = cur_bb.target {
            unreachable!();
        } else {
            cur_bb.target = Some(target);
        }
        self
    }

    pub fn add_br(&mut self, target: LLCursor<BasicBlock>) -> &mut Self {
        let cur_bb = self.cur_bb.as_mut().unwrap();
        cur_bb.push(Inst::Br(0));
        if let Some(_) = cur_bb.target {
            unreachable!();
        } else {
            cur_bb.target = Some(target);
        }
        self
    }

    pub fn add_inst_stloc(&mut self, local_idx: u16) -> &mut Self {
        self.add_inst(match local_idx {
            0 => Inst::StLoc0,
            1 => Inst::StLoc1,
            2 => Inst::StLoc2,
            3 => Inst::StLoc3,
            _ => {
                if local_idx >= u8::MIN as u16 && local_idx <= u8::MAX as u16 {
                    Inst::StLocS(local_idx as u8)
                } else {
                    Inst::StLoc(local_idx)
                }
            }
        })
    }

    pub fn add_inst_ldloc(&mut self, local_idx: u16) -> &mut Self {
        self.add_inst(match local_idx {
            0 => Inst::LdLoc0,
            1 => Inst::LdLoc1,
            2 => Inst::LdLoc2,
            3 => Inst::LdLoc3,
            _ => {
                if local_idx >= u8::MIN as u16 && local_idx <= u8::MAX as u16 {
                    Inst::LdLocS(local_idx as u8)
                } else {
                    Inst::LdLoc(local_idx)
                }
            }
        })
    }

    pub fn add_inst_ldarg(&mut self, arg_offset: u16) -> &mut Self {
        self.add_inst(match arg_offset {
            0 => Inst::LdArg0,
            1 => Inst::LdArg1,
            2 => Inst::LdArg2,
            3 => Inst::LdArg3,
            _ => {
                if arg_offset >= u8::MIN as u16 && arg_offset <= u8::MAX as u16 {
                    Inst::LdArgS(arg_offset as u8)
                } else {
                    unimplemented!("ldarg is not implemeneted");
                }
            }
        })
    }

    pub fn add_inst_starg(&mut self, arg_offset: u16) -> &mut Self {
        self.add_inst(
            if arg_offset >= u8::MIN as u16 && arg_offset <= u8::MAX as u16 {
                Inst::StArgS(arg_offset as u8)
            } else {
                unimplemented!("ldarg is not implemeneted");
            },
        )
    }

    /// Push an int value to the stack
    pub fn add_inst_ldc(&mut self, value: i32) -> &mut Self {
        self.add_inst(match value {
            -1 => Inst::LdCM1,
            0 => Inst::LdC0,
            1 => Inst::LdC1,
            2 => Inst::LdC2,
            3 => Inst::LdC3,
            4 => Inst::LdC4,
            5 => Inst::LdC5,
            6 => Inst::LdC6,
            7 => Inst::LdC7,
            8 => Inst::LdC8,
            _ => {
                if value >= i8::MIN as i32 && value <= i8::MAX as i32 {
                    Inst::LdCI4S(value as i8)
                } else {
                    Inst::LdCI4(value)
                }
            }
        })
    }
}
