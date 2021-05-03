use std::mem;
use std::ptr;

use super::super::data::{BuiltinType, Type};

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
    Value,
    Uninit,
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

impl Default for Slot {
    fn default() -> Self {
        Slot {
            tag: SlotTag::Uninit,
            data: SlotData { i32_: 0 },
        }
    }
}

impl Slot {
    pub fn new(ty: &BuiltinType) -> Self {
        match ty {
            BuiltinType::Void => panic!("Cannot alloc void on stack"),
            BuiltinType::Unk => unreachable!(),
            BuiltinType::Class(_) => panic!("Cannot alloc class on stack"),
            BuiltinType::Bool
            | BuiltinType::Char
            | BuiltinType::U1
            | BuiltinType::U2
            | BuiltinType::U4
            | BuiltinType::I1
            | BuiltinType::I2
            | BuiltinType::I4 => Slot {
                tag: SlotTag::I32,
                data: SlotData { i32_: 0 },
            },
            BuiltinType::U8 | BuiltinType::I8 => Slot {
                tag: SlotTag::I64,
                data: SlotData { i64_: 0 },
            },
            BuiltinType::UNative | BuiltinType::INative => unimplemented!(),
            BuiltinType::R4 => unimplemented!(),
            BuiltinType::R8 => unimplemented!(),
            BuiltinType::ByRef(_) => unimplemented!(),
            BuiltinType::Array(_) => unimplemented!(),
        }
    }

    pub fn expect(&self, tag: SlotTag) {
        if tag != self.tag {
            panic!("Unexpected slot tag");
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

    pub unsafe fn new_ref(addr: *mut u8) -> Slot {
        Slot {
            tag: SlotTag::Ref,
            data: SlotData { ptr_: addr },
        }
    }

    /// interpret as ptr and map into relative address
    ///
    ///
    pub unsafe fn as_addr<T>(&self) -> *mut T {
        if let SlotTag::Ref = self.tag {
            self.data.ptr_ as *mut T
        } else {
            panic!("Slot is not ptr");
        }
    }

    /// return 0 if it is not a value slot entry, else return value size
    pub fn val_size(&self) -> usize {
        if let SlotTag::Value = self.tag {
            unsafe {
                (self.data.ptr_ as *const Type)
                    .as_ref()
                    .unwrap()
                    .instance_field_size
            }
        } else {
            0
        }
    }
}

pub struct EvalStack {
    data: Vec<u8>,
    size: usize,
}

impl EvalStack {
    pub fn new() -> EvalStack {
        EvalStack {
            data: Vec::new(),
            size: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// return top slot body addr
    fn top_ptr(&self) -> *const Slot {
        &self.data[self.data.len() - mem::size_of::<Slot>()] as *const u8 as *const Slot
    }

    fn top_mut_ptr(&mut self) -> *mut Slot {
        let idx = self.data.len() - mem::size_of::<Slot>();
        &mut self.data[idx] as *mut u8 as *mut Slot
    }

    /// peek top slot body
    pub fn peek(&self) -> Option<&Slot> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.top_ptr().as_ref().unwrap() })
        }
    }

    pub fn peek_mut(&mut self) -> Option<&mut Slot> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.top_mut_ptr().as_mut().unwrap() })
        }
    }

    /// caller must guarantee it is a valid space
    pub fn peek_value(&self, val_size: usize) -> &[u8] {
        let tail = self.data.len() - mem::size_of::<Slot>();
        let start = tail - val_size;
        &self.data[start..tail]
    }

    /// dup top slot, including appendix (value)
    pub fn dup(&mut self) {
        // STABLE TODO: use extend_from_within after it becomes stable
        let slot = self.peek().unwrap().clone();
        let val_size = slot.val_size();
        if val_size != 0 {
            let mut data = self.peek_value(val_size).to_vec();
            self.data.append(&mut data);
        }
        self.push_slot(slot);
    }

    /// pop top value, including appendix, but appendix is not returned
    pub fn pop_with_slot(&mut self) -> Slot {
        let ret = self.peek().unwrap().clone();
        self.pop();
        ret
    }

    /// pop top value, including appendix
    pub fn pop(&mut self) {
        let mut pop_size = mem::size_of::<Slot>();
        pop_size += self.peek().unwrap().val_size();
        for _ in 0..pop_size {
            self.data.pop();
        }
        self.size -= 1;
    }

    /// alloc space for certain type as return value, space for value is also allocated
    ///
    /// return slot entry ptr, return null if ret type is void
    pub fn alloc_ret(&mut self, ty: &BuiltinType) -> *mut Slot {
        if let BuiltinType::Void = ty {
            return ptr::null_mut();
        }
        let slot = Slot::new(ty);
        for _ in 0..slot.val_size() {
            self.data.push(0); // any more efficient ways?
        }
        self.push_slot(slot);
        self.top_mut_ptr()
    }

    /// Note: value space will not be allocated
    pub fn push_slot(&mut self, slot: Slot) {
        (0..mem::size_of::<Slot>())
            .into_iter()
            .for_each(|_| self.data.push(0));
        self.size += 1;
        *self.peek_mut().unwrap() = slot;
    }

    pub fn push_ptr(&mut self, v: *mut u8) {
        self.push_slot(Slot {
            tag: SlotTag::Ref,
            data: SlotData { ptr_: v },
        })
    }

    pub fn push_i32(&mut self, v: i32) {
        self.push_slot(Slot {
            tag: SlotTag::I32,
            data: SlotData { i32_: v },
        });
    }
}
