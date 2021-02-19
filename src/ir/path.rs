use core::panic;
use std::{ops::Index, usize};

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // Same as identifier
    static ref SEG_RULE : Regex = Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*").unwrap();
}

pub trait IModPath {
    fn len(&self) -> usize;
    fn get_self_name(&self) -> Option<&str>;
    fn get_super_name(&self) -> Option<&str>;
    fn get_root_name(&self) -> Option<&str>;
    fn to_string(self) -> String;
    fn as_str(&self) -> &str;
    fn iter(&self) -> ModPathIter;
    fn range(&self, start: usize, end: usize) -> ModPathSlice;
}

#[derive(Clone)]
pub struct ModPath {
    path: String,
    seg_tails: Vec<usize>,
}

pub struct ModPathSlice<'p> {
    path: &'p ModPath,
    root: usize,
    tail: usize,
}

impl ModPath {
    pub fn new() -> ModPath {
        ModPath {
            path: String::new(),
            seg_tails: Vec::new(),
        }
    }

    pub fn push(&mut self, seg: &str) {
        if !SEG_RULE.is_match(seg) {
            panic!("Invalid segment {} in path", seg);
        }

        if self.len() != 0 {
            self.path.push_str("/");
        }
        self.path.push_str(seg);
        self.seg_tails.push(self.path.len());
    }

    pub fn as_slice(&self) -> ModPathSlice {
        ModPathSlice {
            path: self,
            root: 0,
            tail: self.len(),
        }
    }

    pub fn get_super(&self) -> ModPathSlice {
        if self.len() <= 1 {
            panic!("Path {} has no super", self.as_str());
        } else {
            ModPathSlice {
                path: self,
                root: 0,
                tail: self.len() - 1,
            }
        }
    }

    /// Canonicalize path
    ///
    /// A valid canonicalized path: `("crate"? | "super"*) ~ Id*`
    ///
    /// Return:
    pub fn canonicalize(&self) -> (bool, usize, ModPath) {
        let mut segs: Vec<&str> = Vec::new();
        let mut has_crate: bool = false;
        let mut super_count = 0;
        for i in (0..self.len()).into_iter() {
            let seg = &self[i];
            if seg == "crate" {
                if i == 0 {
                    has_crate = true;
                    segs.push(seg);
                } else {
                    panic!(
                        "crate should be the first segment in path {}",
                        self.as_str()
                    );
                }
            } else if seg == "super" {
                if segs.len() == 0 {
                    super_count += 1;
                    segs.push(seg);
                } else if *segs.last().unwrap() == "crate" {
                    panic!("Super of crate is invalid");
                } else if *segs.last().unwrap() == "super" {
                    super_count += 1;
                    segs.push(seg);
                } else {
                    // remove last
                    segs.remove(segs.len() - 1);
                }
            } else if seg == "self" {
                continue;
            } else {
                segs.push(seg);
            }
        }
        let mut path = ModPath::new();
        for seg in segs.into_iter() {
            path.push(seg);
        }
        (has_crate, super_count, path)
    }
}

impl IModPath for ModPath {
    fn len(&self) -> usize {
        self.seg_tails.len()
    }

    fn get_self_name(&self) -> Option<&str> {
        match self.len() {
            0 => None,
            1 => Some(&self.path),
            _ => {
                let start = self.seg_tails[self.seg_tails.len() - 2] + 1;
                let end = self.seg_tails[self.seg_tails.len() - 1];
                Some(&self.path[start..end])
            }
        }
    }

    fn get_super_name(&self) -> Option<&str> {
        match self.len() {
            0 | 1 => None,
            _ => {
                let start = self.seg_tails[self.seg_tails.len() - 3] + 1;
                let end = self.seg_tails[self.seg_tails.len() - 2];
                Some(&self.path[start..end])
            }
        }
    }

    fn get_root_name(&self) -> Option<&str> {
        match self.len() {
            0 => None,
            1 => Some(&self.path),
            _ => {
                let end = self.seg_tails[0];
                Some(&self.path[0..end])
            }
        }
    }

    fn to_string(self) -> String {
        self.path
    }

    fn as_str(&self) -> &str {
        &self.path
    }

    fn iter(&self) -> ModPathIter {
        ModPathIter {
            path: self,
            end: self.len(),
            cur: 0,
        }
    }

    fn range(&self, start: usize, end: usize) -> ModPathSlice {
        assert!(end > start, "Invalid range [{}..{}]", start, end);
        assert!(
            end > self.len(),
            "[{}..{}] out of range [0..{}]",
            start,
            end,
            self.len()
        );
        ModPathSlice {
            path: self,
            root: start,
            tail: end,
        }
    }
}

impl Index<usize> for ModPath {
    type Output = str;

    fn index(&self, idx: usize) -> &Self::Output {
        let start = if idx == 1 {
            0
        } else {
            self.seg_tails[idx - 2] + 1
        };
        let tail = self.seg_tails[idx - 1];
        &self.path[start..tail]
    }
}

impl<'p> ModPathSlice<'p> {
    pub fn to_owned(self) -> ModPath {
        ModPath {
            path: self.as_str().to_owned(),
            seg_tails: self.path.seg_tails[self.root..self.tail].to_vec(),
        }
    }

    pub fn to_super(&mut self) {
        if self.len() <= 1 {
            panic!("Path {} has no super", self.as_str());
        } else {
            self.tail -= 1;
        }
    }
}

impl<'p> IModPath for ModPathSlice<'p> {
    fn len(&self) -> usize {
        self.tail - self.root
    }

    fn get_self_name(&self) -> Option<&str> {
        if self.tail > self.path.len() {
            None
        } else {
            Some(&self.path[self.tail - 1])
        }
    }

    fn get_super_name(&self) -> Option<&str> {
        if self.tail > self.path.len() || self.tail <= 1 {
            None
        } else {
            Some(&self.path[self.tail - 2])
        }
    }

    fn get_root_name(&self) -> Option<&str> {
        if self.root >= self.path.len() {
            None
        } else {
            Some(&self.path[self.root])
        }
    }

    fn to_string(self) -> String {
        self.as_str().to_owned()
    }

    fn as_str(&self) -> &str {
        let start = if self.root == 0 {
            0
        } else {
            self.path.seg_tails[self.root - 1] + 1
        };
        let end = self.path.seg_tails[self.tail - 2];
        &self.path.path[start..end]
    }

    fn iter(&self) -> ModPathIter {
        ModPathIter {
            path: self.path,
            end: self.tail,
            cur: self.root,
        }
    }

    fn range(&self, start: usize, end: usize) -> ModPathSlice {
        assert!(end > start, "Invalid range [{}..{}]", start, end);
        assert!(
            end > self.len(),
            "[{}..{}] out of range [0..{}]",
            start,
            end,
            self.len()
        );
        ModPathSlice {
            path: self.path,
            root: self.root + start,
            tail: self.tail + end,
        }
    }
}

pub struct ModPathIter<'p> {
    path: &'p ModPath,
    end: usize,
    cur: usize,
}

impl<'p> Iterator for ModPathIter<'p> {
    type Item = &'p str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.end {
            None
        } else {
            self.cur += 1;
            Some(&self.path[self.cur - 1])
        }
    }
}
