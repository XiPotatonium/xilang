use std::marker::PhantomData;
use std::ptr::{null, null_mut};

struct Node<T> {
    val: T,
    prev: *mut Node<T>,
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
    fn new(val: T, prev: *mut Node<T>, next: Option<Box<Node<T>>>) -> Node<T> {
        Node { val, prev, next }
    }
}

pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    tail: *mut Node<T>,
    size: usize,
}

impl<T> LinkedList<T> {
    pub fn new() -> LinkedList<T> {
        LinkedList {
            head: None,
            tail: null_mut(),
            size: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn front(&self) -> Option<&T> {
        match &self.head {
            Some(head) => Some(head.as_ref().as_ref()),
            None => None,
        }
    }

    pub fn back(&self) -> Option<&T> {
        unsafe {
            match self.tail.as_ref() {
                Some(tail) => Some(tail.as_ref()),
                None => None,
            }
        }
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        match &mut self.head {
            Some(head) => Some(head.as_mut().as_mut()),
            None => None,
        }
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        unsafe {
            match self.tail.as_mut() {
                Some(tail) => Some(tail.as_mut()),
                None => None,
            }
        }
    }

    pub fn push_back(&mut self, val: T) {
        let mut node = Box::new(Node::new(
            val,
            if self.tail.is_null() {
                self.tail
            } else {
                null_mut()
            },
            None,
        ));
        if self.tail.is_null() {
            self.tail = node.as_mut() as *mut Node<T>;
            self.head = Some(node);
        } else {
            unsafe {
                (*self.tail).next = Some(node);
            }
        }
    }

    pub fn iter(&self) -> LinkedListIter<'_, T> {
        LinkedListIter {
            cur: match &self.head {
                Some(head) => head.as_ref() as *const Node<T>,
                None => null(),
            },
            phantom: PhantomData,
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

pub struct LinkedListIter<'i, T: 'i> {
    cur: *const Node<T>,
    // SB rust
    phantom: PhantomData<&'i T>,
}

impl<'i, T> Iterator for LinkedListIter<'i, T> {
    // we will be counting with usize
    type Item = &'i T;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            match self.cur.as_ref() {
                Some(ret) => {
                    self.cur = match &ret.next {
                        Some(next) => next.as_ref() as *const Node<T>,
                        None => null(),
                    };
                    Some(&ret.val)
                }
                None => None,
            }
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
