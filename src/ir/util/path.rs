use std::ops::Index;
use std::{fmt, usize};

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // Same as identifier
    static ref SEG_RULE : Regex = Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*").unwrap();
    static ref PATH_RULE: Regex = Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*(/^[_a-zA-Z][_a-zA-Z0-9]*)*").unwrap();
}

pub trait IItemPath {
    fn len(&self) -> usize;
    fn get_self_name(&self) -> Option<&str>;
    fn get_super_name(&self) -> Option<&str>;
    fn get_root_name(&self) -> Option<&str>;
    fn to_string(self) -> String;
    fn as_str(&self) -> &str;
    fn iter(&self) -> ModPathIter;
    fn range(&self, start: usize, end: usize) -> ItemPath;
}

#[derive(Clone)]
pub struct ItemPathBuf {
    path: String,
    seg_tails: Vec<usize>,
}

pub struct ItemPath<'p> {
    path: &'p ItemPathBuf,
    root: usize,
    tail: usize,
}

// FIXME: UTF-8
impl ItemPathBuf {
    pub fn new() -> ItemPathBuf {
        ItemPathBuf {
            path: String::new(),
            seg_tails: Vec::new(),
        }
    }

    pub fn from_str(p: &str) -> ItemPathBuf {
        if !PATH_RULE.is_match(p) {
            panic!("Invalid path literal {}", p);
        }

        if p.len() == 0 {
            ItemPathBuf::new()
        } else {
            let path = p.to_owned();
            let mut seg_tails = Vec::new();

            for (i, c) in path.char_indices() {
                if c == '/' {
                    seg_tails.push(i);
                }
            }
            seg_tails.push(path.len());

            ItemPathBuf { path, seg_tails }
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

    pub fn as_slice(&self) -> ItemPath {
        ItemPath {
            path: self,
            root: 0,
            tail: self.len(),
        }
    }

    pub fn get_super(&self) -> ItemPath {
        if self.len() == 0 {
            panic!("Empty path has no super");
        } else {
            ItemPath {
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
    pub fn canonicalize(&self) -> (bool, usize, ItemPathBuf) {
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
        let mut path = ItemPathBuf::new();
        for seg in segs.into_iter() {
            path.push(seg);
        }
        (has_crate, super_count, path)
    }
}

impl IItemPath for ItemPathBuf {
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
                let seg_len = self.seg_tails.len();
                let start = if seg_len == 2 {
                    0
                } else {
                    self.seg_tails[self.seg_tails.len() - 3] + 1
                };
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

    fn range(&self, start: usize, end: usize) -> ItemPath {
        assert!(end > start, "Invalid range [{}..{}]", start, end);
        assert!(
            end <= self.len(),
            "[{}..{}] out of range [0..{}]",
            start,
            end,
            self.len()
        );
        ItemPath {
            path: self,
            root: start,
            tail: end,
        }
    }
}

impl Index<usize> for ItemPathBuf {
    type Output = str;

    fn index(&self, idx: usize) -> &Self::Output {
        let start = if idx == 0 {
            0
        } else {
            self.seg_tails[idx - 1] + 1
        };
        let tail = self.seg_tails[idx];
        &self.path[start..tail]
    }
}

impl<'p> ItemPath<'p> {
    pub fn to_owned(self) -> ItemPathBuf {
        ItemPathBuf {
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

impl<'p> IItemPath for ItemPath<'p> {
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
        let end = self.path.seg_tails[self.tail - 1];
        &self.path.path[start..end]
    }

    fn iter(&self) -> ModPathIter {
        ModPathIter {
            path: self.path,
            end: self.tail,
            cur: self.root,
        }
    }

    fn range(&self, start: usize, end: usize) -> ItemPath {
        assert!(end > start, "Invalid range [{}..{}]", start, end);
        assert!(
            end <= self.len(),
            "[{}..{}] out of range [0..{}]",
            start,
            end,
            self.len()
        );
        ItemPath {
            path: self.path,
            root: self.root + start,
            tail: self.tail + end,
        }
    }
}

pub struct ModPathIter<'p> {
    path: &'p ItemPathBuf,
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

impl fmt::Display for ItemPathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl<'p> fmt::Display for ItemPath<'p> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
