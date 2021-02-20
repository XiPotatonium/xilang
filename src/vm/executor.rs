use super::data::VMMethod;
use super::mem::{SharedMem, Slot, Stack};

struct MethodState {
    ip: u32,
    stack: Stack,
    locals: Vec<Slot>,
    args: Vec<Slot>,
}

pub struct TExecutor {
    states: Vec<MethodState>,
}

impl TExecutor {
    pub fn new() -> TExecutor {
        TExecutor { states: Vec::new() }
    }

    pub fn run(&mut self, entry: &VMMethod, mem: &mut SharedMem) {
        // currently executor entry has no arguments
        self.states.push(MethodState::new(vec![], entry));
        unimplemented!();
    }
}

impl MethodState {
    fn new(args: Vec<Slot>, m: &VMMethod) -> MethodState {
        MethodState {
            ip: 0,
            stack: Stack::new(),
            locals: Vec::new(),
            args,
        }
    }
}
