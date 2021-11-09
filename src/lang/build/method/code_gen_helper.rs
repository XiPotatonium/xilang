use ir::Instruction;
use std::mem;

use super::basic_block::{BasicBlock, LLCursor, LinkedList};

pub struct CodeGenHelper {
    pub bb: LinkedList<BasicBlock>,
    cur_bb: LLCursor<BasicBlock>,
}

impl CodeGenHelper {
    pub fn new() -> CodeGenHelper {
        let mut bb = LinkedList::new();
        bb.push_back(BasicBlock::new());
        let cur_bb = bb.cursor_back_mut();

        CodeGenHelper { bb, cur_bb }
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
            unimplemented!()
        } else {
            false
        }
    }
}

impl CodeGenHelper {
    pub fn add_inst(&mut self, inst: Instruction) -> &mut Self {
        self.cur_bb.as_mut().unwrap().push(inst);
        self
    }
}
