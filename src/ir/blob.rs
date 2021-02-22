/// None CLR standard
#[derive(Debug)]
pub enum IrBlob {
    Void,
    Bool,
    Char,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    UNative,
    INative,
    F32,
    F64,
    Obj(u32),
    Func(Vec<u32>, u32),
    Array(u32),
}
