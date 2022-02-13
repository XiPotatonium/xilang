use std::mem;

use super::basic_block::{BasicBlock, LLCursor, LinkedList};

pub struct CodeGenHelper {
    pub bb: LinkedList<BasicBlock>,
    cur_bb: LLCursor<BasicBlock>,
    pub cur_stack_depth: u16,
    pub max_stack_depth: u16,
}

impl CodeGenHelper {
    pub fn new() -> CodeGenHelper {
        let mut bb = LinkedList::new();
        bb.push_back(BasicBlock::new());
        let cur_bb = bb.cursor_back_mut();

        CodeGenHelper {
            bb,
            cur_bb,
            cur_stack_depth: 0,
            max_stack_depth: 0,
        }
    }

    pub fn insert_after_cur(&mut self) -> LLCursor<BasicBlock> {
        self.bb.insert_after_cursor(&self.cur_bb, BasicBlock::new())
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

    fn change_stack_depth(&mut self, slots_delta: i32) {
        // TODO: do overflow or underflow check
        self.cur_stack_depth = ((self.cur_stack_depth as i32) + slots_delta) as u16;
        if self.cur_stack_depth > self.max_stack_depth {
            self.max_stack_depth = self.cur_stack_depth;
        }
    }
}
