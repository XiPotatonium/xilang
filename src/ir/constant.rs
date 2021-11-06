#[derive(Clone, Debug, PartialEq)]
pub enum Constant {
    Utf8(String),          //  1
    Class(u16),            //  7
    String(u16),           //  8
    Fieldref(u16, u16),    //  9
    Methodref(u16, u16),   // 10
    NameAndType(u16, u16), // 12
}
