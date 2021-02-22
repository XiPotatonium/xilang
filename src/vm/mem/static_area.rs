pub struct StaticArea {
    data: Vec<u8>,
}

impl StaticArea {
    pub fn new(size: usize) -> StaticArea {
        StaticArea {
            data: vec![0; size],
        }
    }
}
