use super::Slot;

pub struct Stack {
    data: Vec<Slot>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack { data: Vec::new() }
    }
}
