use std::marker::PhantomData;
use std::mem::swap;
use std::ptr::null_mut;

use crate::ir::inst::Inst;

pub struct BasicBlock {
    pub insts: Vec<Inst>,
    pub offset: i32,
    pub size: usize,
    /// Branch target of the last inst,
    pub target: Option<LLCursor<BasicBlock>>,
}

impl BasicBlock {
    pub fn new() -> BasicBlock {
        BasicBlock {
            insts: Vec::new(),
            offset: 0,
            size: 0,
            target: None,
        }
    }

    pub fn push(&mut self, inst: Inst) {
        self.size += inst.size();
        self.insts.push(inst);
    }
}

pub struct LLCursor<T> {
    node: *mut Node<T>,
}

impl<T> LLCursor<T> {
    pub fn as_mut(&mut self) -> Option<&mut T> {
        unsafe {
            if let Some(n) = self.node.as_mut() {
                Some(n.as_mut())
            } else {
                None
            }
        }
    }

    pub fn as_ref(&self) -> Option<&T> {
        unsafe {
            if let Some(n) = self.node.as_ref() {
                Some(n.as_ref())
            } else {
                None
            }
        }
    }
}

impl<T> Clone for LLCursor<T> {
    fn clone(&self) -> Self {
        LLCursor { node: self.node }
    }
}

struct Node<T> {
    val: T,
    next: Option<Box<Node<T>>>,
}

impl<T> AsRef<T> for Node<T> {
    fn as_ref(&self) -> &T {
        &self.val
    }
}

impl<T> AsMut<T> for Node<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.val
    }
}

impl<T> Node<T> {
    fn new(val: T, next: Option<Box<Node<T>>>) -> Node<T> {
        Node { val, next }
    }
}

pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    tail: *mut Node<T>,
}

impl<T> LinkedList<T> {
    pub fn new() -> LinkedList<T> {
        LinkedList {
            head: None,
            tail: null_mut(),
        }
    }

    pub fn cursor_back_mut(&mut self) -> LLCursor<T> {
        let node = unsafe { self.tail.as_mut().unwrap() as *mut Node<T> };
        LLCursor { node }
    }

    pub fn insert_after_cursor(&mut self, cursor: &LLCursor<T>, val: T) -> LLCursor<T> {
        let mut node = Box::new(Node::new(val, None));
        unsafe {
            let cur_ptr = cursor.node.as_mut().unwrap();
            swap(&mut node.next, &mut cur_ptr.next);
            let ret = LLCursor {
                node: node.as_mut() as *mut Node<T>,
            };
            cur_ptr.next = Some(node);
            ret
        }
    }

    pub fn push_back(&mut self, val: T) {
        let mut node = Box::new(Node::new(val, None));
        if self.tail.is_null() {
            self.tail = node.as_mut() as *mut Node<T>;
            self.head = Some(node);
        } else {
            unsafe {
                (*self.tail).next = Some(node);
            }
        }
    }

    pub fn iter_mut(&mut self) -> LinkedListIterMut<'_, T> {
        LinkedListIterMut {
            cur: match self.head {
                Some(ref mut head) => head.as_mut() as *mut Node<T>,
                None => null_mut(),
            },
            phantom: PhantomData,
        }
    }
}

pub struct LinkedListIterMut<'i, T: 'i> {
    cur: *mut Node<T>,
    // SB rust
    phantom: PhantomData<&'i mut T>,
}

impl<'i, T> Iterator for LinkedListIterMut<'i, T> {
    // we will be counting with usize
    type Item = &'i mut T;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            match self.cur.as_mut() {
                Some(ret) => {
                    self.cur = match &mut ret.next {
                        Some(next) => next.as_mut() as *mut Node<T>,
                        None => null_mut(),
                    };
                    Some(&mut ret.val)
                }
                None => None,
            }
        }
    }
}
