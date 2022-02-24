use crate::lang::sym::Func;
use crate::lang::sym::RValType;

use std::fmt;
use std::mem;
use std::ptr;

pub struct ActivationRecord<'m> {
    pub method: &'m Func,
    pub eval_stack: EvalStack,
    pub ret_addr: *mut Slot,
    pub locals: Vec<Slot>,
}

#[derive(Clone)]
#[repr(C)]
pub struct Slot {
    // TODO: make tag and data priv
    pub tag: SlotTag,
    pub data: SlotData,
}

#[derive(Clone, PartialEq, Debug)]
#[repr(C)]
pub enum SlotTag {
    I32,
    I64,
    INative,
    F32,
    F64,
    Ref,
    UnInit,
}

impl fmt::Display for SlotTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SlotTag::I32 => write!(f, "i32"),
            SlotTag::I64 => write!(f, "i64"),
            SlotTag::INative => write!(f, "inative"),
            SlotTag::F32 => write!(f, "f32"),
            SlotTag::F64 => write!(f, "f64"),
            SlotTag::Ref => write!(f, "ref"),
            SlotTag::UnInit => write!(f, "UnInit"),
        }
    }
}

/// Not CLI standard, see I.12.1
#[derive(Clone, Copy)]
#[repr(C)]
pub union SlotData {
    pub i32_: i32,
    pub i64_: i64,
    pub inative_: isize,
    pub f32_: f32,
    pub f64_: f64,
    pub ptr_: *mut u8,
}

impl Slot {
    pub fn new(ty: &RValType) -> Self {
        match ty {
            RValType::None => todo!(),
            RValType::Bool | RValType::U8 | RValType::Char | RValType::I32 => Slot {
                tag: SlotTag::I32,
                data: SlotData { i32_: 0 },
            },
            RValType::F64 => Slot {
                tag: SlotTag::F64,
                data: SlotData { f64_: 0.0 },
            },
            RValType::ISize | RValType::USize => Slot {
                tag: SlotTag::INative,
                data: SlotData { inative_: 0 },
            },
            RValType::ClassRef(_) => Slot {
                tag: SlotTag::Ref,
                data: SlotData {
                    ptr_: ptr::null_mut(),
                },
            },
            RValType::Array(_) => todo!(),
            RValType::UnInit => unreachable!(),
        }
    }

    pub fn expect(&self, tag: SlotTag) {
        if tag != self.tag {
            panic!("Expect {} but found {} in stack", tag, self.tag);
        }
    }

    pub fn null() -> Self {
        Slot {
            tag: SlotTag::Ref,
            data: SlotData {
                ptr_: ptr::null_mut(),
            },
        }
    }
}

pub struct EvalStack {
    data: Vec<Slot>,
}

impl EvalStack {
    pub fn new() -> EvalStack {
        EvalStack { data: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// peek top slot body
    pub fn peek(&self) -> Option<&Slot> {
        self.data.last()
    }

    pub fn peek_at(&self, at: usize) -> Option<&Slot> {
        if at >= self.data.len() {
            None
        } else {
            Some(&self.data[at])
        }
    }

    pub fn peek_at_mut(&mut self, at: usize) -> Option<&mut Slot> {
        if at >= self.data.len() {
            None
        } else {
            Some(&mut self.data[at])
        }
    }

    pub fn peek_mut(&mut self) -> Option<&mut Slot> {
        self.data.last_mut()
    }

    pub fn pop(&mut self) -> Option<Slot> {
        self.data.pop()
    }
}

// push
impl EvalStack {
    /// dup top slot, including appendix (value)
    pub fn dup(&mut self) {
        // STABLE TODO: use extend_from_within after it becomes stable
        let slot = self.peek().unwrap().clone();
        self.push_slot(slot);
    }

    pub fn push_slot(&mut self, slot: Slot) {
        self.data.push(slot);
    }

    pub fn push_ref(&mut self, ptr: *mut u8) {
        self.push_slot(Slot {
            tag: SlotTag::Ref,
            data: SlotData { ptr_: ptr },
        });
    }

    pub fn push_i32(&mut self, v: i32) {
        self.push_slot(Slot {
            tag: SlotTag::I32,
            data: SlotData { i32_: v },
        });
    }

    pub fn push_usize(&mut self, v: usize) {
        self.push_slot(Slot {
            tag: SlotTag::INative,
            data: SlotData {
                inative_: unsafe { mem::transmute::<usize, isize>(v) },
            },
        })
    }
}
