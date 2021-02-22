use super::{Slot, SlotData, SlotTag};

pub struct Stack {
    data: Vec<Slot>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack { data: Vec::new() }
    }

    pub fn peek(&self) -> &Slot {
        self.data.last().unwrap()
    }

    pub fn peek_mut(&mut self) -> &mut Slot {
        self.data.last_mut().unwrap()
    }

    pub fn dup(&mut self) {
        self.data.push(self.data.last().unwrap().clone());
    }

    pub fn pop(&mut self) -> Slot {
        self.data.pop().unwrap()
    }

    pub fn pop_n(&mut self, n: usize) -> Vec<Slot> {
        let ret = self.data[self.data.len() - n..].to_vec();
        (0..n).into_iter().for_each(|_| {
            self.data.pop();
        });
        ret
    }

    pub fn push(&mut self, v: Slot) {
        self.data.push(v);
    }

    pub fn push_i32(&mut self, v: i32) {
        self.data.push(Slot {
            tag: SlotTag::I32,
            data: SlotData { i32_: v },
        });
    }
}
