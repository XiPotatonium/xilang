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
    /// 0x0E, ldarg.s <idx>
    LdArgS(u8),

    /// 0x10, starg.s <idx>
    ///
    /// Store argument to index <odx>
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
    /// 0x11, ldloc.s <idx>
    LdLocS(u8),
    /// 0xFE0C, ldloc <idx>
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
    /// 0x13, stloc.s <idx>
    StLocS(u8),
    /// 0xFE0E, stloc <idx>
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
    /// 0x1F, ldc.i4.s <num>
    LdCI4S(i8),
    /// 0x20, ldc.i4 <num>
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

    /// 0x28, call <func>
    ///
    /// Call a func, See ECMA-335 page 368
    ///
    /// `..., arg0, arg1 ... argN -> ..., retVal`
    Call(u32),
    /// 0x2A, ret
    ///
    /// return from current method
    ///
    /// `retVal -> ..., retVal`
    Ret,

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

    /// 0x58, add
    ///
    /// Add two numeric values without overflow check
    ///
    /// `..., val1, val2 -> ..., res`
    Add,
    /// 0x5D, rem
    ///
    /// a % b
    ///
    /// `..., val1, val2 -> ..., res`
    Rem,

    /// 0x6F, callvirt <method>
    ///
    /// Call a virtual method associate with an obj
    ///
    /// `..., obj, arg1, ..., argN -> ..., retVal`
    CallVirt(u32),
    /// 0x73, new <ctor>
    ///
    /// Call a creator, return obj addr
    ///
    /// `..., arg0, ..., argN -> ..., obj`
    NewObj(u32),
    /// 0x7B, ldfld <field>
    ///
    /// Load a field onto the stack
    ///
    /// `..., obj -> ..., val`
    LdFld(u32),
    /// 0x7D, stfld <field>
    ///
    /// Store a value to field
    ///
    /// `..., obj, val -> ...,`
    StFld(u32),
    /// 0x7E, ldsfld <field>
    ///
    /// Load a static field onto the stack
    ///
    /// `..., -> ..., val`
    LdSFld(u32),
    /// 0x80, stsfld <field>
    ///
    /// Store a value to field
    ///
    /// `..., val -> ...,`
    StSFld(u32),
}

impl Inst {
    pub fn size(&self) -> usize {
        match self {
            Inst::Nop => 1,

            Inst::LdArg0 => 1,
            Inst::LdArg1 => 1,
            Inst::LdArg2 => 1,
            Inst::LdArg3 => 1,
            Inst::LdArgS(_) => 2,

            Inst::StArgS(_) => 2,

            Inst::LdLoc0 => 1,
            Inst::LdLoc1 => 1,
            Inst::LdLoc2 => 1,
            Inst::LdLoc3 => 1,
            Inst::LdLocS(_) => 2,
            Inst::LdLoc(_) => 4,
            Inst::StLoc0 => 1,
            Inst::StLoc1 => 1,
            Inst::StLoc2 => 1,
            Inst::StLoc3 => 1,
            Inst::StLocS(_) => 2,
            Inst::StLoc(_) => 4,

            Inst::LdNull => 1,
            Inst::LdCM1 => 1,
            Inst::LdC0 => 1,
            Inst::LdC1 => 1,
            Inst::LdC2 => 1,
            Inst::LdC3 => 1,
            Inst::LdC4 => 1,
            Inst::LdC5 => 1,
            Inst::LdC6 => 1,
            Inst::LdC7 => 1,
            Inst::LdC8 => 1,
            Inst::LdCI4S(_) => 2,
            Inst::LdCI4(_) => 5,

            Inst::Dup => 1,
            Inst::Pop => 1,

            Inst::Call(_) => 5,
            Inst::Ret => 1,

            Inst::BrFalse(_) => 5,
            Inst::BrTrue(_) => 5,
            Inst::BEq(_) => 5,
            Inst::BGe(_) => 5,
            Inst::BGt(_) => 5,
            Inst::BLe(_) => 5,
            Inst::BLt(_) => 5,

            Inst::Add => 1,
            Inst::Rem => 1,

            Inst::CallVirt(_) => 5,
            Inst::NewObj(_) => 5,
            Inst::LdFld(_) => 5,
            Inst::StFld(_) => 5,
            Inst::LdSFld(_) => 5,
            Inst::StSFld(_) => 5,
        }
    }
}
