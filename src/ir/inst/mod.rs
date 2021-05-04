mod fmt;
mod serde;

use std::mem;

#[derive(Clone, Debug, PartialEq)]
pub enum Inst {
    /// 0x00, nop
    Nop,

    /// 0x02, ldarg.0
    ///
    /// Load argument at index 0
    ///
    /// `... -> ..., val`
    LdArg0,
    /// 0x03, ldarg.1
    LdArg1,
    /// 0x04, ldarg.2
    LdArg2,
    /// 0x05, ldarg.3
    LdArg3,
    /// 0x0E, ldarg.s idx
    LdArgS(u8),

    /// 0x10, starg.s idx
    ///
    /// Store argument to **idx**
    ///
    /// `..., val -> ...,`
    StArgS(u8),

    /// 0x06, ldloc.0
    ///
    /// Load local var 0 onto stack
    ///
    /// `..., -> ..., val`
    LdLoc0,
    /// 0x07, ldloc.1
    LdLoc1,
    /// 0x08, ldloc.2
    LdLoc2,
    /// 0x09, ldloc.3
    LdLoc3,
    /// 0x11, ldloc.s idx
    LdLocS(u8),
    /// 0xFE0C, ldloc idx
    LdLoc(u16),

    /// 0x0A, stloc.0
    ///
    /// Store val to local var 0
    ///
    /// `... val -> ...`
    StLoc0,
    /// 0x0B, stloc.1
    StLoc1,
    /// 0x0C, stloc.2
    StLoc2,
    /// 0x0D, stloc.3
    StLoc3,
    /// 0x13, stloc.s idx
    StLocS(u8),
    /// 0xFE0E, stloc idx
    StLoc(u16),

    /// 0x14, ldnull
    ///
    /// Push a null ref on the stack
    ///
    /// `... -> ..., null`
    LdNull,
    /// 0x15, ldc.i4.m1
    ///
    /// Push -1 onto the stack as i32
    ///
    /// `... -> ..., num`
    LdCM1,
    /// 0x16, ldc.i4.0
    LdC0,
    /// 0x17, ldc.i4.1
    LdC1,
    /// 0x18, ldc.i4.2
    LdC2,
    /// 0x19, ldc.i4.3
    LdC3,
    /// 0x1A, ldc.i4.4
    LdC4,
    /// 0x1B, ldc.i4.5
    LdC5,
    /// 0x1C, ldc.i4.6
    LdC6,
    /// 0x1D, ldc.i4.7
    LdC7,
    /// 0x1E, ldc.i4.8
    LdC8,
    /// 0x1F, ldc.i4.s num
    LdCI4S(i8),
    /// 0x20, ldc.i4 num
    LdCI4(i32),

    /// 0x25, dup
    ///
    /// Dup top stack value
    ///
    /// `..., val -> ..., val, val`
    Dup,
    /// 0x26, pop
    ///
    /// Pop top val from stack
    ///
    /// `..., val -> ...`
    Pop,

    /// 0x28, call func
    ///
    /// Call a func, See ECMA-335 page 368.
    /// tok is MethodDef/MemberRef/MethodSpec
    ///
    /// `..., arg0, arg1 ... argN -> ..., retVal`
    Call(u32),
    /// 0x2A, ret
    ///
    /// return from current method
    ///
    /// `retVal -> ..., retVal`
    Ret,

    /// 0x38, br
    Br(i32),

    /// 0x39, brfalse
    BrFalse(i32),

    /// 0x3A, brtrue
    BrTrue(i32),

    /// 0x3B, beq
    BEq(i32),

    /// 0x3C, bge
    BGe(i32),

    /// 0x3D, bgt
    BGt(i32),

    /// 0x3E, ble
    BLe(i32),

    /// 0x3F, blt
    BLt(i32),

    /// 0xFE01, ceq
    CEq,

    /// 0xFE02 cgt
    CGt,

    /// 0xFE04 clt
    CLt,

    /// 0x58, add
    ///
    /// Add two numeric values without overflow check
    ///
    /// `..., val1, val2 -> ..., res`
    Add,
    /// 0x59, sub
    Sub,
    /// 0x5A, mul
    Mul,
    /// 0x5B, div
    Div,
    /// 0x5D, rem
    ///
    /// a % b
    ///
    /// `..., val1, val2 -> ..., res`
    Rem,

    /// 0x65, neg
    ///
    /// neg int or float
    Neg,

    /// 0x6F, callvirt method
    ///
    /// Call a virtual method associate with an obj
    ///
    /// `..., obj, arg1, ..., argN -> ..., retVal`
    CallVirt(u32),
    /// 0x73, new ctor
    ///
    /// Call a creator, return obj addr
    ///
    /// `..., arg0, ..., argN -> ..., obj`
    NewObj(u32),
    /// 0x7B, ldfld field
    ///
    /// Load a field onto the stack, **field** is Field/MemberRef token
    ///
    /// `..., obj -> ..., val`
    LdFld(u32),
    /// 0x7D, stfld field
    ///
    /// Store a value to field
    ///
    /// `..., obj, val -> ...,`
    StFld(u32),
    /// 0x7E, ldsfld field
    ///
    /// Load a static field onto the stack
    ///
    /// `..., -> ..., val`
    LdSFld(u32),
    /// 0x80, stsfld field
    ///
    /// Store a value to field
    ///
    /// `..., val -> ...,`
    StSFld(u32),

    /// 0x72, ldstr literal
    ///
    /// load **str**: std::String from with **literal**: idx into usr str heap.
    /// String interning will be used to eliminate duplication
    ///
    /// ..., -> ..., str
    LdStr(u32),

    /// 0x8D, newarr ty
    ///
    /// create arr of **size**: unative|i32 size
    ///
    /// ..., size -> ..., arr
    NewArr(u32),
    /// 0x8E, ldlen
    ///
    /// load the **len**: unative of the **arr**: O
    ///
    /// ..., arr -> len
    LdLen,
    /// 0x94, ldelem.i4
    LdElemI4,
    /// 0x9E, stelem.i4
    StElemI4,
    /// 0xA3, ldelem ty
    ///
    /// load elem at **idx**: i32|inative onto the stack
    ///
    /// ..., arr, idx -> ..., val
    LdElem(u32),
    /// 0xA4, stelem ty
    ///
    /// ..., arr, idx, val -> ...
    StElem(u32),
}

const INST_SIZE: usize = 1;
// 0xFEXX
const FAT_INST_SIZE: usize = 2;

impl Inst {
    pub fn size(&self) -> usize {
        match self {
            Inst::Nop => INST_SIZE,

            Inst::LdArg0 | Inst::LdArg1 | Inst::LdArg2 | Inst::LdArg3 => INST_SIZE,
            Inst::LdArgS(_) | Inst::StArgS(_) => INST_SIZE + mem::size_of::<u8>(),

            Inst::LdLoc0 | Inst::LdLoc1 | Inst::LdLoc2 | Inst::LdLoc3 => INST_SIZE,
            Inst::LdLocS(_) => INST_SIZE + mem::size_of::<u8>(),
            Inst::LdLoc(_) => FAT_INST_SIZE + mem::size_of::<u16>(),

            Inst::StLoc0 | Inst::StLoc1 | Inst::StLoc2 | Inst::StLoc3 => INST_SIZE,
            Inst::StLocS(_) => INST_SIZE + mem::size_of::<u8>(),
            Inst::StLoc(_) => FAT_INST_SIZE + mem::size_of::<u16>(),

            Inst::LdNull
            | Inst::LdCM1
            | Inst::LdC0
            | Inst::LdC1
            | Inst::LdC2
            | Inst::LdC3
            | Inst::LdC4
            | Inst::LdC5
            | Inst::LdC6
            | Inst::LdC7
            | Inst::LdC8 => INST_SIZE,
            Inst::LdCI4S(_) => INST_SIZE + mem::size_of::<u8>(),
            Inst::LdCI4(_) => INST_SIZE + mem::size_of::<i32>(),

            Inst::Dup => INST_SIZE,
            Inst::Pop => INST_SIZE,

            Inst::Call(_) => INST_SIZE + mem::size_of::<u32>(),
            Inst::Ret => INST_SIZE,

            Inst::Br(_)
            | Inst::BrFalse(_)
            | Inst::BrTrue(_)
            | Inst::BEq(_)
            | Inst::BGe(_)
            | Inst::BGt(_)
            | Inst::BLe(_)
            | Inst::BLt(_) => INST_SIZE + mem::size_of::<i32>(),

            Inst::CEq | Inst::CGt | Inst::CLt => FAT_INST_SIZE,

            Inst::Add | Inst::Sub | Inst::Mul | Inst::Div | Inst::Rem => INST_SIZE,

            Inst::Neg => INST_SIZE,

            Inst::CallVirt(_)
            | Inst::NewObj(_)
            | Inst::LdFld(_)
            | Inst::StFld(_)
            | Inst::LdSFld(_)
            | Inst::StSFld(_) => INST_SIZE + mem::size_of::<u32>(),

            Inst::LdStr(_) => INST_SIZE + mem::size_of::<u32>(),

            Inst::NewArr(_) => INST_SIZE + mem::size_of::<u32>(),

            Inst::LdLen => INST_SIZE,

            Inst::LdElemI4 | Inst::StElemI4 => INST_SIZE,

            Inst::LdElem(_) | Inst::StElem(_) => INST_SIZE + mem::size_of::<u32>(),
        }
    }
}
