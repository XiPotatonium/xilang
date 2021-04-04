/// Similar to fat format
pub struct CorILMethod {
    /// max stack
    pub max_stack: u16,
    /// max locals
    pub local: u16,
    pub insts: Vec<u8>,
}
