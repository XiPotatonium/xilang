use super::serde::{IDeserializer, ISerializable};

pub enum Inst {
    /// 0x00, nop
    ///
    /// Do nothing
    NOp,
    /// 0x01, aconst_null
    ///
    /// Push null
    ///
    /// ... -> ...,null
    AConstNull,
    /// 0x02, iconst_m1
    IConstM1,
    /// 0x03, iconst_0
    IConst0,
    /// 0x04, iconst_1
    IConst1,
    /// 0x05, iconst_2
    IConst2,
    /// 0x06, iconst_3
    IConst3,
    /// 0x07, iconst_4
    IConst4,
    /// 0x08, iconst_5
    IConst5,
    /// 0x09, lconst_0
    LConst0,
    /// 0x0a, lconst_1
    LConst1,
    /// 0x0b, fconst_0
    FConst0,
    /// 0x0c, fconst_1
    FConst1,
    /// 0x0d, fconst_2
    FConst2,
    /// 0x0e, dconst_0
    ///
    /// Push double
    DConst0,
    /// 0x0f, dconst_1
    DConst1,
    /// 0x10, bipush byte
    ///
    /// Push byte
    ///
    /// ... -> ...,value
    BiPush(i8),
    /// 0x11, sipush byte1 byte2
    SiPush(i16),
    /// 0x12, ldc index
    ///
    /// Push item from constant pool
    LdC(u8),
    /// 0x13, ldc_w indexbyte1 indexbyte2
    LdCW(u16),
    /// 0x14, ldc2_w indexbyte1 indexbyte2
    ///
    /// Push long/double from constant pool
    LdC2W(u16),
    /// 0x15, iload index
    ILoad(u8),
    /// 0x16, lload index
    LLoad(u8),
    /// 0x17, fload index
    FLoad(u8),
    /// 0x18, dload index
    ///
    /// Load double from local vars
    ///
    /// ... -> ...,value
    DLoad(u8),
    /// 0x19, aload index
    ///
    /// Load reference from local vars
    ///
    /// ... -> ...,objref
    ALoad(u8),
    /// 0x1a, iload_0
    ILoad0,
    /// 0x1b, iload_1
    ILoad1,
    /// 0x1c, iload_2
    ILoad2,
    /// 0x1d, iload_3
    ILoad3,
    /// 0x1e, lload_0
    LLoad0,
    /// 0x1f, lload_1
    LLoad1,
    /// 0x20, lload_2
    LLoad2,
    /// 0x21, lload_3
    LLoad3,
    /// 0x22, fload_0
    FLoad0,
    /// 0x23, fload_1
    FLoad1,
    /// 0x24, fload_2
    FLoad2,
    /// 0x25, fload_3
    FLoad3,
    /// 0x26, dload_0
    DLoad0,
    /// 0x27, dload_1
    DLoad1,
    /// 0x28, dload_2
    DLoad2,
    /// 0x29, dload_3
    DLoad3,
    /// 0x2a, aload_0
    ALoad0,
    /// 0x2b, aload_1
    ALoad1,
    /// 0x2c, aload_2
    ALoad2,
    /// 0x2d, aload_3
    ALoad3,
    /// 0x2e, iaload
    ///
    /// Load int from array
    IALoad,
    /// 0x2f, laload
    LALoad,
    /// 0x30, faload
    ///
    /// Load float from array
    FALoad,
    /// 0x31, daload
    ///
    /// Load double from array
    ///
    /// ...,arrayref,index -> ...,value
    DALoad,
    /// 0x32, aaload
    ///
    /// Load reference from array
    ///
    /// ...,arrayref,index -> ...,value
    AALoad,
    /// 0x33, baload
    ///
    /// Load byte/boolean from array
    ///
    /// ...,arrayref,index -> ...,value
    BALoad,
    /// 0x34, caload
    ///
    /// Load char from array
    ///
    /// ...,arrayref,index -> ...,value
    CALoad,
    /// 0x35, saload
    SALoad,
    /// 0x36, istore index
    IStore(u8),
    /// 0x37, lstore index
    LStore(u8),
    /// 0x38, fstore index
    FStore(u8),
    /// 0x39, dstore index
    ///
    /// Store double into local vars
    ///
    /// ...,value -> ...
    DStore(u8),
    /// 0x3a, astore index
    ///
    /// Store reference into local vars
    ///
    /// ...,value -> ...
    AStore(u8),
    /// 0x3b, istore_0
    IStore0,
    /// 0x3c, istore_1
    IStore1,
    /// 0x3d, istore_2
    IStore2,
    /// 0x3e, istore_3
    IStore3,
    /// 0x3f, lstore_0
    LStore0,
    /// 0x40, lstore_1
    LStore1,
    /// 0x41, lstore_2
    LStore2,
    /// 0x42, lstore_3
    LStore3,
    /// 0x43, fstore_0
    FStore0,
    /// 0x44, fstore_1
    FStore1,
    /// 0x45, fstore_2
    FStore2,
    /// 0x46, fstore_3
    FStore3,
    /// 0x47, dstore_0
    DStore0,
    /// 0x48, dstore_1
    DStore1,
    /// 0x49, dstore_2
    DStore2,
    /// 0x4a, dstore_3
    DStore3,
    /// 0x4b, astore_0
    AStore0,
    /// 0x4c, astore_1
    AStore1,
    /// 0x4d, astore_2
    AStore2,
    /// 0x4e, astore_3
    AStore3,
    /// 0x4f, iastore
    ///
    /// Store into int array
    IAStore,
    /// 0x50, lastore
    LAStore,
    /// 0x51, fastore
    ///
    /// Store into float array
    FAStore,
    /// 0x52, dastore
    ///
    /// Store into double array
    ///
    /// ...,arrayref,index,value -> ...
    DAStore,
    /// 0x53, aastore
    ///
    /// Store into reference array
    ///
    /// ...,arrayref,index,value -> ...
    AAStore,
    /// 0x54, bastore
    ///
    /// Store into byte/boolean array
    ///
    /// ...,arrayref,index,value -> ...
    BAStore,
    /// 0x55, castore
    ///
    /// Store into char array
    ///
    /// ...,arrayref,index,value -> ...
    CAStore,
    /// 0x56, sastore
    SAStore,
    /// 0x57, pop
    ///
    /// Pop top stack value
    ///
    /// ...,val -> ...
    Pop,
    /// 0x58, pop2
    ///
    /// Pop top 2 stack value, if top value is category 2, pop 1 stack value
    /// https://docs.oracle.com/javase/specs/jvms/se17/html/jvms-6.html#jvms-6.5.pop2
    Pop2,
    /// 0x59, dup
    ///
    /// Duplicate the top stack value
    ///
    /// ...,val -> ...,val,val
    Dup,
    /// 0x5a, dup_x1
    ///
    /// Duplicate the top operand stack value and insert two values down.
    /// val1 and val2 must be category 1 computational type
    ///
    /// ...,val2,val1 -> ...,val1,val2,val1
    DupX1,
    /// 0x5b, dup_x2
    ///
    /// https://docs.oracle.com/javase/specs/jvms/se17/html/jvms-6.html#jvms-6.5.dup_x2
    DupX2,
    /// 0x5c, dup2
    ///
    /// Duplicate the top one or two operand stack values, see jvm doc for more detail
    Dup2,
    /// 0x5d, dup2_x1
    Dup2X1,
    /// 0x5e, dup2_x2
    Dup2X2,
    /// 0x5f, swap
    ///
    /// Swap top 2 stack values
    ///
    /// ...,val2,val1 -> ...,val1,val2
    Swap,
    /// 0x60, iadd
    IAdd,
    /// 0x61, ladd
    LAdd,
    /// 0x62, fadd
    FAdd,
    /// 0x63, dadd
    ///
    /// Add double
    ///
    /// ...,val1,val2 -> ...,result
    DAdd,
    /// 0x64, isub
    ISub,
    /// 0x65, lsub
    LSub,
    /// 0x66, fsub
    FSub,
    /// 0x67, dsub
    ///
    /// Subtract double
    ///
    /// ...,val1,val2 -> ...,result
    DSub,
    /// 0x68, imul
    IMul,
    /// 0x69, lmul
    LMul,
    /// 0x6a, fmul
    FMul,
    /// 0x6b, dmul
    ///
    /// Multiple double
    ///
    /// ...,val1,val2 -> ...,result
    DMul,
    /// 0x6c, idiv
    IDiv,
    /// 0x6d, ldiv
    LDiv,
    /// 0x6e, fdiv
    FDiv,
    /// 0x6f, ddiv
    ///
    /// Divide double
    ///
    /// ...,val1,val2 -> ...,result
    DDiv,
    /// 0x70, irem
    IRem,
    /// 0x71, lrem
    LRem,
    /// 0x72, frem
    FRem,
    /// 0x73, drem
    ///
    /// Remainder double
    ///
    /// ...,val1,val2 -> ...,result
    DRem,
    /// 0x74, ineg
    INeg,
    /// 0x75, lneg
    LNeg,
    /// 0x76, fneg
    FNeg,
    /// 0x77, dneg
    ///
    /// Negate double
    ///
    /// ...,val -> ...,result
    DNeg,
    /// 0x78, ishl
    ///
    /// Shift left int
    IShl,
    /// 0x79, lshl
    LShl,
    /// 0x7a, ishr
    ///
    /// Arithmetic shift right int
    IShr,
    /// 0x7b, lshr
    LShr,
    /// 0x7c, iushr
    ///
    /// Logical shift right int
    IUShr,
    /// 0x7d, lushr
    ///
    /// logical shift right long
    LUShr,
    /// 0x7e, iand
    IAnd,
    /// 0x7f, land
    LAnd,
    /// 0x80, ior
    IOr,
    /// 0x81, lor
    LOr,
    /// 0x82, ixor
    IXor,
    /// 0x83, lxor
    LXor,
    /// 0x84, iinc index const
    ///
    /// Increment local var by constant
    IInc(u8, i32),
    /// 0x85, i2l
    I2L,
    /// 0x86, i2f
    I2F,
    /// 0x87, i2d
    I2D,
    /// 0x88, l2i
    L2I,
    /// 0x89, l2f
    L2F,
    /// 0x8a, l2d
    L2D,
    /// 0x8b, f2i
    F2I,
    /// 0x8c, f2l
    F2L,
    /// 0x8d, f2d
    F2D,
    /// 0x8e, d2i
    ///
    /// Convert double to int
    ///
    /// ...,value -> ...,result
    D2I,
    /// 0x8f, d2l
    ///
    /// Convert double to long
    ///
    /// ...,value -> ...,result
    D2L,
    /// 0x90, d2f
    ///
    /// Convert double to float
    ///
    /// ...,value -> ...,result
    D2F,
    /// 0x91, i2b
    I2B,
    /// 0x92, i2c
    I2C,
    /// 0x93, i2s
    I2S,
    /// 0x94, lcmp
    ///
    /// Compare long values
    ///
    /// ...,val1,val2 -> ...,result
    LCmp,
    /// 0x95, fcmpl
    FCmpL,
    /// 0x96, fcmpg
    FCmpG,
    /// 0x97, dcmpl
    ///
    /// Compare double, return int
    ///
    /// ...,val1,val2 -> result
    DCmpL,
    /// 0x98, dcmpg
    DCmpG,
    /// 0x99, ifeq branchbyte1 branchbyte2
    ///
    /// Branch if int comparison with 0 succeeds
    IfEq(i16),
    /// 0x9a, ifne
    IfNe(i16),
    /// 0x9b, iflt
    IfLt(i16),
    /// 0x9c, ifge
    IfGe(i16),
    /// 0x9d, ifgt
    IfGt(i16),
    /// 0x9e, ifle
    IfLe(i16),
    /// 0x9f, if_icmpeq branchbyte1 branchbyte2
    IfICmpEq(i16),
    /// 0xa0, if_icmpne branchbyte1 branchbyte2
    IfICmpNe(i16),
    /// 0xa1, if_icmplt branchbyte1 branchbyte2
    IfICmpLt(i16),
    /// 0xa2, if_icmpge branchbyte1 branchbyte2
    IfICmpGe(i16),
    /// 0xa3, if_icmpgt branchbyte1 branchbyte2
    IfICmpGt(i16),
    /// 0xa4, if_icmple branchbyte1 branchbyte2
    IfICmpLe(i16),
    /// 0xa5, if_acmpeq branchbyte1 branchbyte2
    ///
    /// Branch if reference comparison succeeds
    ///
    /// ...,val1,val2 -> ...
    IfACmpEq(i16),
    /// 0xa6, if_acmpne branchbyte1 branchbyte2
    IfACmpNe(i16),
    /// 0xa7, got branchbyte1 branchbyte2
    ///
    /// Branch to a signed branch offset
    Goto(i16),
    /// 0xa8, jsr branchbyte1 branchbyte2
    ///
    /// jump subroutine, push pc into stack and jump
    ///
    /// ... -> ...,address
    Jsr(i16),
    /// 0xa9, ret index
    ///
    /// Return from subroutinue
    /// https://docs.oracle.com/javase/specs/jvms/se17/html/jvms-6.html#jvms-6.5.ret
    Ret(u8),
    /// 0xaa, tableswitch defaultbyte1/2/3/4 lobyte1/2/3/4 hibyte1/2/3/4
    ///
    /// https://docs.oracle.com/javase/specs/jvms/se17/html/jvms-6.html#jvms-6.5.tableswitch
    ///
    /// ...,index -> ...
    TableSwitch(i32, i32, i32, Vec<i32>),
    /// 0xab, loopupswitch defaultbyte1/2/3/4 npairs1/2/3/4
    ///
    /// https://docs.oracle.com/javase/specs/jvms/se17/html/jvms-6.html#jvms-6.5.lookupswitch
    ///
    /// ...,key -> ...
    LookUpSwitch(i32, Vec<i32>),
    /// 0xac, ireturn
    IReturn,
    /// 0xad, lreturn
    LReturn,
    /// 0xae, freturn
    FReturn,
    /// 0xaf, dreturn
    ///
    /// Return double from method
    ///
    /// ...,value -> [empty]
    DReturn,
    /// 0xb0, areturn
    ///
    /// Return reference from method
    ///
    /// ...,objectref -> [empty]
    AReturn,
    /// 0xb1, return
    ///
    /// Return void
    ///
    /// ... -> [empty]
    Return,
    /// 0xb2, getstatic indexbyte1 indexbyte2
    ///
    /// Fetch static field from class
    ///
    /// ... -> ...,value
    GetStatic(u16),
    /// 0xb3, putstatic indexbyte1 indexbyte2
    ///
    /// Set static field in class
    ///
    /// ...,val -> ...
    PutStatic(u16),
    /// 0xb4, getfield indexbyte1 indexbyte2
    ///
    /// Fetch field from object
    ///
    /// ...,objectref -> ...,value
    GetField(u16),
    /// 0xb5, putfield indexbyte1 indexbyte2
    ///
    /// Set field in object
    ///
    /// ...,objectref,val -> ...
    PutField(u16),
    /// 0xb6, invokevirtual indexbyte1 indexbyte2
    ///
    /// Invoke instance method; dispatch based on class
    ///
    /// ...,objectref,[arg1,[arg2...]] -> ...
    InvokeVirtual(u16),
    /// 0xb7, invokespecial indexbyte1 indexbyte2
    ///
    /// Invoke instance method;
    /// direct invocation of instance initialization methods and methods of the current class and its supertypes
    ///
    /// ...,objectref,[arg1,[arg2...]] -> ...
    InvokeSpecial(u16),
    /// 0xb8, invokestatic indexbyte1 indexbyte2
    ///
    /// Invoke a class static method
    ///
    /// ...,[arg1,[arg2...]] -> ...
    InvokeStatic(u16),
    /// 0xb9, invokeinterface indexbyte1 indexbyte2 count 0
    ///
    /// Invoke a interface method
    ///
    /// ...,objectref,[arg1,[arg2...]] -> ...
    InvokeInterface(u16, u8),
    /// 0xba, invokedynamic indexbyte1 indexbyte2 0 0
    ///
    /// Invoke a dynamic computed call-site
    ///
    /// ...,[arg1,[arg2...]] -> ...
    InvokeDynamic(u16),
    /// 0xbb, new indexbyte1 indexbyte2
    ///
    /// Create new object
    ///
    /// ... -> ...,objectref
    New(u16),
    /// 0xbc, newarray atype
    ///
    /// Create new array, atype is a byte
    ///
    /// ...,count -> ...,arrayref
    NewArray(ArrayType),
    /// 0xbd, anewarray indexbyte1 indexbyte2
    ///
    /// Create new array of reference, index is (indexbyte1 << 8) | indexbyte2.
    ///
    /// ...,count -> ...,arrayref
    ANewArray(u16),
    /// 0xbe, arraylength
    ///
    /// Get length of array
    ///
    /// ...,arrayref -> ...,length
    ArrayLength,
    /// 0xbf, athrow
    ///
    /// ...,objectref -> objectref
    AThrow,
    /// 0xc0, checkcast indexbyte1 indexbyte2
    ///
    /// Check whether object is of a given type
    ///
    /// ...,objectref -> ...,objectref
    CheckCast(u16),
    /// 0xc1, instanceof indexbyte1 indexbyte2
    ///
    /// Determine if object is a given type
    ///
    /// ...,objectref -> ...,result
    InstanceOf(u16),
    /// 0xc2, moniterenter
    MonitorEnter,
    /// 0xc3, moniterexit
    MonitorExit,
    /// 0xc4, wide WidePayload
    ///
    /// Extend local var index into u16
    Wide(WidePayload),
    /// 0xc5, multianewarray indexbyte1 indexbyte2 dimensions
    ///
    /// ...,count1,[count2,...] -> arrayref
    MultiANewArray(u16, u8),
    /// 0xc6, ifnull branchbyte1 branchbyte2
    ///
    /// branch if reference is null
    ///
    /// ...,val -> ...
    IfNull(i16),
    /// 0xc7, ifnonnull branchbyte1 branchbyte2
    ///
    /// branch if reference is not Null
    ///
    /// ...,val -> ...
    IfNonNull(i16),
    /// 0xc8, goto_w branchbyte1 branchbyte2 branchbyte3 branchbyte4
    GotoW(i32),
    /// 0xc9, jsr_w branchbyte1 branchbyte2 branchbyte3 branchbyte4
    JsrW(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidePayload {
    ILoad(u16),
    FLoad(u16),
    ALoad(u16),
    LLoad(u16),
    DLoad(u16),
    IStore(u16),
    FStore(u16),
    AStore(u16),
    LStore(u16),
    DStore(u16),
    Ret(u16),
    IInc(u16, u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArrayType {
    Boolean,
    Char,
    Float,
    Double,
    Byte,
    Short,
    Int,
    Long,
}

impl TryFrom<u8> for ArrayType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            4 => Ok(ArrayType::Boolean),
            5 => Ok(ArrayType::Char),
            6 => Ok(ArrayType::Float),
            7 => Ok(ArrayType::Double),
            8 => Ok(ArrayType::Byte),
            9 => Ok(ArrayType::Short),
            10 => Ok(ArrayType::Int),
            11 => Ok(ArrayType::Long),
            _ => Err("Invalid array type"),
        }
    }
}

impl From<ArrayType> for u8 {
    fn from(value: ArrayType) -> Self {
        match value {
            ArrayType::Boolean => 4,
            ArrayType::Char => 5,
            ArrayType::Float => 6,
            ArrayType::Double => 7,
            ArrayType::Byte => 8,
            ArrayType::Short => 9,
            ArrayType::Int => 10,
            ArrayType::Long => 11,
        }
    }
}

impl ISerializable for ArrayType {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(u8::from(*self));
    }

    fn deserialize(_: &mut dyn IDeserializer) -> Self {
        todo!()
    }
}

impl Inst {
    pub fn size(&self) -> usize {
        match self {
            Inst::NOp => 1,
            Inst::AConstNull
            | Inst::IConstM1
            | Inst::IConst0
            | Inst::IConst1
            | Inst::IConst2
            | Inst::IConst3
            | Inst::IConst4
            | Inst::IConst5
            | Inst::LConst0
            | Inst::LConst1
            | Inst::FConst0
            | Inst::FConst1
            | Inst::FConst2
            | Inst::DConst0
            | Inst::DConst1 => 1,
            Inst::BiPush(_) => 2,
            Inst::SiPush(_) => 3,
            Inst::LdC(_) => 2,
            Inst::LdCW(_) => 3,
            Inst::LdC2W(_) => 3,
            Inst::ILoad(_) | Inst::LLoad(_) | Inst::FLoad(_) | Inst::DLoad(_) | Inst::ALoad(_) => 2,
            Inst::ILoad0
            | Inst::ILoad1
            | Inst::ILoad2
            | Inst::ILoad3
            | Inst::LLoad0
            | Inst::LLoad1
            | Inst::LLoad2
            | Inst::LLoad3
            | Inst::FLoad0
            | Inst::FLoad1
            | Inst::FLoad2
            | Inst::FLoad3
            | Inst::DLoad0
            | Inst::DLoad1
            | Inst::DLoad2
            | Inst::DLoad3
            | Inst::ALoad0
            | Inst::ALoad1
            | Inst::ALoad2
            | Inst::ALoad3 => 1,
            Inst::IALoad
            | Inst::LALoad
            | Inst::FALoad
            | Inst::DALoad
            | Inst::AALoad
            | Inst::BALoad
            | Inst::CALoad
            | Inst::SALoad => 1,
            Inst::IStore(_)
            | Inst::LStore(_)
            | Inst::FStore(_)
            | Inst::DStore(_)
            | Inst::AStore(_) => 2,
            Inst::IStore0
            | Inst::IStore1
            | Inst::IStore2
            | Inst::IStore3
            | Inst::LStore0
            | Inst::LStore1
            | Inst::LStore2
            | Inst::LStore3
            | Inst::FStore0
            | Inst::FStore1
            | Inst::FStore2
            | Inst::FStore3
            | Inst::DStore0
            | Inst::DStore1
            | Inst::DStore2
            | Inst::DStore3
            | Inst::AStore0
            | Inst::AStore1
            | Inst::AStore2
            | Inst::AStore3
            | Inst::IAStore
            | Inst::LAStore
            | Inst::FAStore
            | Inst::DAStore
            | Inst::AAStore
            | Inst::BAStore
            | Inst::CAStore
            | Inst::SAStore => 1,
            Inst::Pop
            | Inst::Pop2
            | Inst::Dup
            | Inst::DupX1
            | Inst::DupX2
            | Inst::Dup2
            | Inst::Dup2X1
            | Inst::Dup2X2
            | Inst::Swap => 1,
            Inst::IAdd
            | Inst::LAdd
            | Inst::FAdd
            | Inst::DAdd
            | Inst::ISub
            | Inst::LSub
            | Inst::FSub
            | Inst::DSub
            | Inst::IMul
            | Inst::LMul
            | Inst::FMul
            | Inst::DMul
            | Inst::IDiv
            | Inst::LDiv
            | Inst::FDiv
            | Inst::DDiv
            | Inst::IRem
            | Inst::LRem
            | Inst::FRem
            | Inst::DRem
            | Inst::INeg
            | Inst::LNeg
            | Inst::FNeg
            | Inst::DNeg
            | Inst::IShl
            | Inst::LShl
            | Inst::IShr
            | Inst::LShr
            | Inst::IUShr
            | Inst::LUShr
            | Inst::IAnd
            | Inst::LAnd
            | Inst::IOr
            | Inst::LOr
            | Inst::IXor
            | Inst::LXor => 1,
            Inst::IInc(_, _) => 6,
            Inst::I2L
            | Inst::I2F
            | Inst::I2D
            | Inst::L2I
            | Inst::L2F
            | Inst::L2D
            | Inst::F2I
            | Inst::F2L
            | Inst::F2D
            | Inst::D2I
            | Inst::D2L
            | Inst::D2F
            | Inst::I2B
            | Inst::I2C
            | Inst::I2S => 1,
            Inst::LCmp | Inst::FCmpL | Inst::FCmpG | Inst::DCmpL | Inst::DCmpG => 1,
            Inst::IfEq(_)
            | Inst::IfNe(_)
            | Inst::IfLt(_)
            | Inst::IfGe(_)
            | Inst::IfGt(_)
            | Inst::IfLe(_) => 3,
            Inst::IfICmpEq(_)
            | Inst::IfICmpNe(_)
            | Inst::IfICmpLt(_)
            | Inst::IfICmpGe(_)
            | Inst::IfICmpGt(_)
            | Inst::IfICmpLe(_)
            | Inst::IfACmpEq(_)
            | Inst::IfACmpNe(_) => 3,
            Inst::Goto(_) | Inst::Jsr(_) => 3,
            Inst::Ret(_) => 2,
            Inst::TableSwitch(_, _, _, tbl) => 4 * (4 + tbl.len()),
            Inst::LookUpSwitch(_, tbl) => 4 * (3 + tbl.len()),
            Inst::IReturn
            | Inst::LReturn
            | Inst::FReturn
            | Inst::DReturn
            | Inst::AReturn
            | Inst::Return => 1,
            Inst::GetStatic(_) | Inst::PutStatic(_) | Inst::GetField(_) | Inst::PutField(_) => 3,
            Inst::InvokeVirtual(_) | Inst::InvokeSpecial(_) | Inst::InvokeStatic(_) => 3,
            Inst::InvokeInterface(_, _) => 4,
            Inst::InvokeDynamic(_) => 3,
            Inst::New(_) => 3,
            Inst::NewArray(_) => 2,
            Inst::ANewArray(_) => 3,
            Inst::ArrayLength => 1,
            Inst::AThrow => 1,
            Inst::CheckCast(_) => 3,
            Inst::InstanceOf(_) => 3,
            Inst::MonitorEnter => 1,
            Inst::MonitorExit => 1,
            Inst::Wide(payload) => 1 + payload.size(),
            Inst::MultiANewArray(_, _) => 4,
            Inst::IfNull(_) => 3,
            Inst::IfNonNull(_) => 3,
            Inst::GotoW(_) => 5,
            Inst::JsrW(_) => 5,
        }
    }
}

impl WidePayload {
    pub fn size(&self) -> usize {
        match self {
            WidePayload::ILoad(_)
            | WidePayload::FLoad(_)
            | WidePayload::ALoad(_)
            | WidePayload::LLoad(_)
            | WidePayload::DLoad(_) => 3,
            WidePayload::IStore(_)
            | WidePayload::FStore(_)
            | WidePayload::AStore(_)
            | WidePayload::LStore(_)
            | WidePayload::DStore(_) => 3,
            WidePayload::Ret(_) => 3,
            WidePayload::IInc(_, _) => 5,
        }
    }
}

impl ISerializable for WidePayload {
    fn serialize(&self, buf: &mut Vec<u8>) {
        match self {
            WidePayload::ILoad(index) => {
                buf.push(0x15);
                index.serialize(buf);
            }
            WidePayload::LLoad(index) => {
                buf.push(0x16);
                index.serialize(buf);
            }
            WidePayload::FLoad(index) => {
                buf.push(0x17);
                index.serialize(buf);
            }
            WidePayload::DLoad(index) => {
                buf.push(0x18);
                index.serialize(buf);
            }
            WidePayload::ALoad(index) => {
                buf.push(0x19);
                index.serialize(buf);
            }
            WidePayload::IStore(index) => {
                buf.push(0x36);
                index.serialize(buf);
            }
            WidePayload::LStore(index) => {
                buf.push(0x37);
                index.serialize(buf);
            }
            WidePayload::FStore(index) => {
                buf.push(0x38);
                index.serialize(buf);
            }
            WidePayload::DStore(index) => {
                buf.push(0x39);
                index.serialize(buf);
            }
            WidePayload::AStore(index) => {
                buf.push(0x3a);
                index.serialize(buf);
            }
            WidePayload::Ret(index) => {
                buf.push(0xa9);
                index.serialize(buf);
            }
            WidePayload::IInc(index, c) => {
                buf.push(0x84);
                index.serialize(buf);
                c.serialize(buf);
            }
        }
    }

    fn deserialize(_: &mut dyn IDeserializer) -> Self {
        todo!()
    }
}

impl ISerializable for Inst {
    fn serialize(&self, buf: &mut Vec<u8>) {
        match self {
            Inst::NOp => buf.push(0x00),
            Inst::AConstNull => buf.push(0x01),
            Inst::IConstM1 => buf.push(0x02),
            Inst::IConst0 => buf.push(0x03),
            Inst::IConst1 => buf.push(0x04),
            Inst::IConst2 => buf.push(0x05),
            Inst::IConst3 => buf.push(0x06),
            Inst::IConst4 => buf.push(0x07),
            Inst::IConst5 => buf.push(0x08),
            Inst::LConst0 => buf.push(0x09),
            Inst::LConst1 => buf.push(0x0a),
            Inst::FConst0 => buf.push(0x0b),
            Inst::FConst1 => buf.push(0x0c),
            Inst::FConst2 => buf.push(0x0d),
            Inst::DConst0 => buf.push(0x0e),
            Inst::DConst1 => buf.push(0x0f),
            Inst::BiPush(c) => {
                buf.push(0x10);
                c.serialize(buf);
            }
            Inst::SiPush(c) => {
                buf.push(0x11);
                c.serialize(buf);
            }
            Inst::LdC(index) => {
                buf.push(0x12);
                index.serialize(buf);
            }
            Inst::LdCW(index) => {
                buf.push(0x13);
                index.serialize(buf);
            }
            Inst::LdC2W(index) => {
                buf.push(0x14);
                index.serialize(buf);
            }
            Inst::ILoad(index) => {
                buf.push(0x15);
                index.serialize(buf);
            }
            Inst::LLoad(index) => {
                buf.push(0x16);
                index.serialize(buf);
            }
            Inst::FLoad(index) => {
                buf.push(0x17);
                index.serialize(buf);
            }
            Inst::DLoad(index) => {
                buf.push(0x18);
                index.serialize(buf);
            }
            Inst::ALoad(index) => {
                buf.push(0x19);
                index.serialize(buf);
            }
            Inst::ILoad0 => buf.push(0x1a),
            Inst::ILoad1 => buf.push(0x1b),
            Inst::ILoad2 => buf.push(0x1c),
            Inst::ILoad3 => buf.push(0x1d),
            Inst::LLoad0 => buf.push(0x1e),
            Inst::LLoad1 => buf.push(0x1f),
            Inst::LLoad2 => buf.push(0x20),
            Inst::LLoad3 => buf.push(0x21),
            Inst::FLoad0 => buf.push(0x22),
            Inst::FLoad1 => buf.push(0x23),
            Inst::FLoad2 => buf.push(0x24),
            Inst::FLoad3 => buf.push(0x25),
            Inst::DLoad0 => buf.push(0x26),
            Inst::DLoad1 => buf.push(0x27),
            Inst::DLoad2 => buf.push(0x28),
            Inst::DLoad3 => buf.push(0x29),
            Inst::ALoad0 => buf.push(0x2a),
            Inst::ALoad1 => buf.push(0x2b),
            Inst::ALoad2 => buf.push(0x2c),
            Inst::ALoad3 => buf.push(0x2d),
            Inst::IALoad => buf.push(0x2e),
            Inst::LALoad => buf.push(0x2f),
            Inst::FALoad => buf.push(0x30),
            Inst::DALoad => buf.push(0x31),
            Inst::AALoad => buf.push(0x32),
            Inst::BALoad => buf.push(0x33),
            Inst::CALoad => buf.push(0x34),
            Inst::SALoad => buf.push(0x35),
            Inst::IStore(index) => {
                buf.push(0x36);
                index.serialize(buf);
            }
            Inst::LStore(index) => {
                buf.push(0x37);
                index.serialize(buf);
            }
            Inst::FStore(index) => {
                buf.push(0x38);
                index.serialize(buf);
            }
            Inst::DStore(index) => {
                buf.push(0x39);
                index.serialize(buf);
            }
            Inst::AStore(index) => {
                buf.push(0x3a);
                index.serialize(buf);
            }
            Inst::IStore0 => buf.push(0x3b),
            Inst::IStore1 => buf.push(0x3c),
            Inst::IStore2 => buf.push(0x3d),
            Inst::IStore3 => buf.push(0x3e),
            Inst::LStore0 => buf.push(0x3f),
            Inst::LStore1 => buf.push(0x40),
            Inst::LStore2 => buf.push(0x41),
            Inst::LStore3 => buf.push(0x42),
            Inst::FStore0 => buf.push(0x43),
            Inst::FStore1 => buf.push(0x44),
            Inst::FStore2 => buf.push(0x45),
            Inst::FStore3 => buf.push(0x46),
            Inst::DStore0 => buf.push(0x47),
            Inst::DStore1 => buf.push(0x48),
            Inst::DStore2 => buf.push(0x49),
            Inst::DStore3 => buf.push(0x4a),
            Inst::AStore0 => buf.push(0x4b),
            Inst::AStore1 => buf.push(0x4c),
            Inst::AStore2 => buf.push(0x4d),
            Inst::AStore3 => buf.push(0x4e),
            Inst::IAStore => buf.push(0x4f),
            Inst::LAStore => buf.push(0x50),
            Inst::FAStore => buf.push(0x51),
            Inst::DAStore => buf.push(0x52),
            Inst::AAStore => buf.push(0x53),
            Inst::BAStore => buf.push(0x54),
            Inst::CAStore => buf.push(0x55),
            Inst::SAStore => buf.push(0x56),
            Inst::Pop => buf.push(0x57),
            Inst::Pop2 => buf.push(0x58),
            Inst::Dup => buf.push(0x59),
            Inst::DupX1 => buf.push(0x5a),
            Inst::DupX2 => buf.push(0x5b),
            Inst::Dup2 => buf.push(0x5c),
            Inst::Dup2X1 => buf.push(0x5d),
            Inst::Dup2X2 => buf.push(0x5e),
            Inst::Swap => buf.push(0x5f),
            Inst::IAdd => buf.push(0x60),
            Inst::LAdd => buf.push(0x61),
            Inst::FAdd => buf.push(0x62),
            Inst::DAdd => buf.push(0x63),
            Inst::ISub => buf.push(0x64),
            Inst::LSub => buf.push(0x65),
            Inst::FSub => buf.push(0x66),
            Inst::DSub => buf.push(0x67),
            Inst::IMul => buf.push(0x68),
            Inst::LMul => buf.push(0x69),
            Inst::FMul => buf.push(0x6a),
            Inst::DMul => buf.push(0x6b),
            Inst::IDiv => buf.push(0x6c),
            Inst::LDiv => buf.push(0x6d),
            Inst::FDiv => buf.push(0x6e),
            Inst::DDiv => buf.push(0x6f),
            Inst::IRem => buf.push(0x70),
            Inst::LRem => buf.push(0x71),
            Inst::FRem => buf.push(0x72),
            Inst::DRem => buf.push(0x73),
            Inst::INeg => buf.push(0x74),
            Inst::LNeg => buf.push(0x75),
            Inst::FNeg => buf.push(0x76),
            Inst::DNeg => buf.push(0x77),
            Inst::IShl => buf.push(0x78),
            Inst::LShl => buf.push(0x79),
            Inst::IShr => buf.push(0x7a),
            Inst::LShr => buf.push(0x7b),
            Inst::IUShr => buf.push(0x7c),
            Inst::LUShr => buf.push(0x7d),
            Inst::IAnd => buf.push(0x7e),
            Inst::LAnd => buf.push(0x7f),
            Inst::IOr => buf.push(0x80),
            Inst::LOr => buf.push(0x81),
            Inst::IXor => buf.push(0x82),
            Inst::LXor => buf.push(0x83),
            Inst::IInc(index, c) => {
                buf.push(0x84);
                index.serialize(buf);
                c.serialize(buf);
            }
            Inst::I2L => buf.push(0x85),
            Inst::I2F => buf.push(0x86),
            Inst::I2D => buf.push(0x87),
            Inst::L2I => buf.push(0x88),
            Inst::L2F => buf.push(0x89),
            Inst::L2D => buf.push(0x8a),
            Inst::F2I => buf.push(0x8b),
            Inst::F2L => buf.push(0x8c),
            Inst::F2D => buf.push(0x8d),
            Inst::D2I => buf.push(0x8e),
            Inst::D2L => buf.push(0x8f),
            Inst::D2F => buf.push(0x90),
            Inst::I2B => buf.push(0x91),
            Inst::I2C => buf.push(0x92),
            Inst::I2S => buf.push(0x93),
            Inst::LCmp => buf.push(0x94),
            Inst::FCmpL => buf.push(0x95),
            Inst::FCmpG => buf.push(0x96),
            Inst::DCmpL => buf.push(0x97),
            Inst::DCmpG => buf.push(0x98),
            Inst::IfEq(offset) => {
                buf.push(0x99);
                offset.serialize(buf);
            }
            Inst::IfNe(offset) => {
                buf.push(0x9a);
                offset.serialize(buf);
            }
            Inst::IfLt(offset) => {
                buf.push(0x9b);
                offset.serialize(buf);
            }
            Inst::IfGe(offset) => {
                buf.push(0x9c);
                offset.serialize(buf);
            }
            Inst::IfGt(offset) => {
                buf.push(0x9d);
                offset.serialize(buf);
            }
            Inst::IfLe(offset) => {
                buf.push(0x9e);
                offset.serialize(buf);
            }
            Inst::IfICmpEq(offset) => {
                buf.push(0x9f);
                offset.serialize(buf);
            }
            Inst::IfICmpNe(offset) => {
                buf.push(0xa0);
                offset.serialize(buf);
            }
            Inst::IfICmpLt(offset) => {
                buf.push(0xa1);
                offset.serialize(buf);
            }
            Inst::IfICmpGe(offset) => {
                buf.push(0xa2);
                offset.serialize(buf);
            }
            Inst::IfICmpGt(offset) => {
                buf.push(0xa3);
                offset.serialize(buf);
            }
            Inst::IfICmpLe(offset) => {
                buf.push(0xa4);
                offset.serialize(buf);
            }
            Inst::IfACmpEq(offset) => {
                buf.push(0xa5);
                offset.serialize(buf);
            }
            Inst::IfACmpNe(offset) => {
                buf.push(0xa6);
                offset.serialize(buf);
            }
            Inst::Goto(offset) => {
                buf.push(0xa7);
                offset.serialize(buf);
            }
            Inst::Jsr(offset) => {
                buf.push(0xa8);
                offset.serialize(buf);
            }
            Inst::Ret(index) => {
                buf.push(0xa9);
                index.serialize(buf);
            }
            Inst::TableSwitch(default, lo, high, tbl) => {
                buf.push(0xaa);
                for _ in 0..3 {
                    // padding 3 byte 0x00
                    buf.push(0x00);
                }
                default.serialize(buf);
                lo.serialize(buf);
                high.serialize(buf);
                for entry in tbl.iter() {
                    entry.serialize(buf);
                }
            }
            Inst::LookUpSwitch(default, tbl) => {
                buf.push(0xab);
                for _ in 0..3 {
                    // padding 3 byte 0x00
                    buf.push(0x00);
                }
                default.serialize(buf);
                (tbl.len() as u32).serialize(buf);
                for entry in tbl.iter() {
                    entry.serialize(buf);
                }
            }
            Inst::IReturn => buf.push(0xac),
            Inst::LReturn => buf.push(0xad),
            Inst::FReturn => buf.push(0xae),
            Inst::DReturn => buf.push(0xaf),
            Inst::AReturn => buf.push(0xb0),
            Inst::Return => buf.push(0xb1),
            Inst::GetStatic(index) => {
                buf.push(0xb2);
                index.serialize(buf);
            }
            Inst::PutStatic(index) => {
                buf.push(0xb3);
                index.serialize(buf);
            }
            Inst::GetField(index) => {
                buf.push(0xb4);
                index.serialize(buf);
            }
            Inst::PutField(index) => {
                buf.push(0xb5);
                index.serialize(buf);
            }
            Inst::InvokeVirtual(index) => {
                buf.push(0xb6);
                index.serialize(buf);
            }
            Inst::InvokeSpecial(index) => {
                buf.push(0xb7);
                index.serialize(buf);
            }
            Inst::InvokeStatic(index) => {
                buf.push(0xb8);
                index.serialize(buf);
            }
            Inst::InvokeInterface(index, count) => {
                buf.push(0xb9);
                index.serialize(buf);
                count.serialize(buf);
            }
            Inst::InvokeDynamic(index) => {
                buf.push(0xba);
                index.serialize(buf);
            }
            Inst::New(index) => {
                buf.push(0xbb);
                index.serialize(buf);
            }
            Inst::NewArray(atype) => {
                buf.push(0xbc);
                atype.serialize(buf);
            }
            Inst::ANewArray(index) => {
                buf.push(0xbd);
                index.serialize(buf);
            }
            Inst::ArrayLength => buf.push(0xbe),
            Inst::AThrow => buf.push(0xbf),
            Inst::CheckCast(index) => {
                buf.push(0xc0);
                index.serialize(buf);
            }
            Inst::InstanceOf(index) => {
                buf.push(0xc1);
                index.serialize(buf);
            }
            Inst::MonitorEnter => buf.push(0xc2),
            Inst::MonitorExit => buf.push(0xc3),
            Inst::Wide(payload) => {
                buf.push(0xc4);
                payload.serialize(buf);
            }
            Inst::MultiANewArray(index, dim) => {
                buf.push(0xc5);
                index.serialize(buf);
                dim.serialize(buf);
            }
            Inst::IfNull(offset) => {
                buf.push(0xc6);
                offset.serialize(buf);
            }
            Inst::IfNonNull(offset) => {
                buf.push(0xc7);
                offset.serialize(buf);
            }
            Inst::GotoW(offset) => {
                buf.push(0xc8);
                offset.serialize(buf);
            }
            Inst::JsrW(offset) => {
                buf.push(0xc9);
                offset.serialize(buf);
            }
        }
    }

    fn deserialize(_: &mut dyn IDeserializer) -> Self {
        todo!()
    }
}
