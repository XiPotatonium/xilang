pub struct Heap {
    data: Vec<u8>,
}

impl Heap {
    pub fn new(size: usize) -> Heap {
        Heap {
            data: vec![0; size],
        }
    }
}
