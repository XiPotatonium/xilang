use std::fmt;
use std::mem;
use std::mem::size_of;
use std::ptr;

use super::super::data::{BuiltinType, Type, TypedAddr};

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
    Managed,
    Ref,
    /// SlotData.ptr_ stores *mut Type
    Value,
    Uninit,
}

impl fmt::Display for SlotTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SlotTag::I32 => write!(f, "i32"),
            SlotTag::I64 => write!(f, "i64"),
            SlotTag::INative => write!(f, "inative"),
            SlotTag::F32 => write!(f, "f32"),
            SlotTag::F64 => write!(f, "f64"),
            SlotTag::Managed => write!(f, "&"),
            SlotTag::Ref => write!(f, "O"),
            SlotTag::Value => write!(f, "val"),
            SlotTag::Uninit => write!(f, "UINIT"),
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
            BuiltinType::Bool
            | BuiltinType::Char
            | BuiltinType::U1
            | BuiltinType::U4
            | BuiltinType::I1
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
            BuiltinType::Class(ty) => {
                let ty_ref = unsafe { ty.as_ref() };
                if ty_ref.ee_class.is_value {
                    Slot {
                        tag: SlotTag::Value,
                        data: SlotData {
                            ptr_: ty.as_ptr() as *mut u8,
                        },
                    }
                } else {
                    Slot {
                        tag: SlotTag::Ref,
                        data: SlotData {
                            ptr_: ptr::null_mut(),
                        },
                    }
                }
            }
            BuiltinType::String | BuiltinType::ByRef(_) | BuiltinType::SZArray(_) => Slot {
                tag: SlotTag::Ref,
                data: SlotData {
                    ptr_: ptr::null_mut(),
                },
            },
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

    pub unsafe fn new_managed(addr: *mut u8) -> Slot {
        Slot {
            tag: SlotTag::Managed,
            data: SlotData { ptr_: addr },
        }
    }

    pub unsafe fn expect_ref(&self) -> *mut u8 {
        if let SlotTag::Ref = self.tag {
            self.data.ptr_ as *mut u8
        } else {
            panic!("Expect O but found {}", self.tag);
        }
    }

    pub fn expect_ref_or_ptr(&self) -> *mut u8 {
        match self.tag {
            SlotTag::INative => unsafe { mem::transmute::<isize, *mut u8>(self.data.inative_) },
            SlotTag::Managed | SlotTag::Ref => unsafe { self.data.ptr_ },
            _ => panic!("Expect O or ptr but found {}", self.tag),
        }
    }

    /// managed or unmanaged
    pub unsafe fn expect_ptr(&self) -> *mut u8 {
        match self.tag {
            SlotTag::INative => mem::transmute::<isize, *mut u8>(self.data.inative_),
            SlotTag::Managed => self.data.ptr_,
            _ => panic!("Expect ptr but found {}", self.tag),
        }
    }

    /// return 0 if it is not a value slot entry, else return value size
    pub fn val_size(&self) -> usize {
        if let SlotTag::Value = self.tag {
            unsafe {
                (self.data.ptr_ as *const Type)
                    .as_ref()
                    .unwrap()
                    .basic_instance_size
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

    pub fn peek_at(&self, at: usize) -> Option<&Slot> {
        if self.size < at + 1 {
            None
        } else {
            let mut addr = self.top_ptr();
            for _ in 0..at {
                let val_size = unsafe { addr.as_ref().unwrap() }.val_size();
                addr = addr.wrapping_sub(1);
                addr = (addr as *const u8).wrapping_sub(val_size) as *const Slot;
            }
            Some(unsafe { addr.as_ref().unwrap() })
        }
    }

    pub fn peek_mut(&mut self) -> Option<&mut Slot> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.top_mut_ptr().as_mut().unwrap() })
        }
    }

    pub fn peek_value(&self, ty: &Type) -> *mut u8 {
        let header_slot = self.peek().unwrap();
        if let SlotTag::Value = header_slot.tag {
            let slot_ty = unsafe { header_slot.data.ptr_ as *const Type };
            if slot_ty != ty {
                panic!("Incompatible type");
            }
            &self.data[self.data.len() - mem::size_of::<Slot>() - ty.basic_instance_size]
                as *const u8 as *mut u8
        } else {
            panic!("Stack top slot is not a value");
        }
    }

    /// pop top value, including appendix.
    /// If target is not None, value type is expected and value will be stored at target.addr, type compliance will be checked
    pub fn pop(&mut self, target: Option<TypedAddr>) -> Slot {
        let ret = self.peek().unwrap().clone();

        for _ in 0..mem::size_of::<Slot>() {
            self.data.pop();
        }
        if let Some(target) = target {
            ret.expect(SlotTag::Value);
            unsafe {
                let slot_ty = ret.data.ptr_ as *const Type;
                if slot_ty != target.ty.as_ptr() {
                    panic!("Incompatible value type");
                }
                for i in (0..target.ty.as_ref().basic_instance_size).rev() {
                    *target.addr.wrapping_add(i) = self.data.pop().unwrap();
                }
            }
            if let SlotTag::Value = ret.tag {
            } else {
                unreachable!();
            }
        } else {
            for _ in 0..ret.val_size() {
                self.data.pop();
            }
        }

        self.size -= 1;

        ret
    }
}

// push
impl EvalStack {
    /// dup top slot, including appendix (value)
    pub fn dup(&mut self) {
        // STABLE TODO: use extend_from_within after it becomes stable
        let slot = self.peek().unwrap().clone();
        let val_size = slot.val_size();
        if val_size != 0 {
            let tail = self.data.len() - mem::size_of::<Slot>();
            let start = tail - val_size;
            // TODO: use extend_from_within
            self.data.append(&mut self.data[start..tail].to_vec());
        }
        self.push_slot(slot);
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

    /// return value addr, this addr should not be hold for long since eval stack is rapidly changing
    ///
    /// Drop that before mutate (push/pop) eval stack
    ///
    /// caller must guarantee init has same size with ty, or something unexpected will happen
    pub unsafe fn alloc_value(&mut self, ty: &Type, init: *const u8) -> *mut u8 {
        if init.is_null() {
            for _ in 0..ty.basic_instance_size {
                self.data.push(0); // any more efficient ways?
            }
        } else {
            for i in 0..ty.basic_instance_size {
                self.data.push(*init.wrapping_add(i)); // any more efficient ways?
            }
        }
        self.push_slot(Slot {
            tag: SlotTag::Value,
            data: SlotData {
                ptr_: ty as *const Type as *mut u8,
            },
        });
        &self.data[self.data.len() - ty.basic_instance_size - size_of::<Slot>()] as *const u8
            as *mut u8
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

    pub fn push_managed(&mut self, v: *mut u8) {
        self.push_slot(Slot {
            tag: SlotTag::Managed,
            data: SlotData { ptr_: v },
        })
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
