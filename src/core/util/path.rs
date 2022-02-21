use std::fmt;

use lazy_static::lazy_static;
use regex::Regex;

const ID_RULE: &str = r"^[^/]*";

lazy_static! {
    // Same as identifier
    static ref SEG_RULE : Regex = Regex::new(ID_RULE).unwrap();
    static ref PATH_RULE: Regex = Regex::new(&format!("{}(/{})*", ID_RULE, ID_RULE)).unwrap();
}

#[derive(Clone)]
struct PathSegment {
    id_tail: usize,
}

pub trait IItemPath {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get_self(&self) -> Option<&str>;
    fn get_super(&self) -> Option<&str>;
    fn get_root(&self) -> Option<&str>;
    fn to_string(self) -> String;
    fn as_str(&self) -> &str;
    fn iter(&self) -> ItemPathIter;
    fn range(&self, start: usize, end: usize) -> ItemPath;
}

#[derive(Clone, Default)]
pub struct ItemPathBuf {
    path: String,
    segs: Vec<PathSegment>,
}

/// path[start:to]
pub struct ItemPath<'p> {
    path: &'p ItemPathBuf,
    start: usize,
    to: usize,
}

// FIXME: UTF-8
impl ItemPathBuf {
    /// p must be module fullname, no generic parameter
    ///
    /// TODO: rename
    pub fn from_ir_path(p: &str) -> ItemPathBuf {
        if !PATH_RULE.is_match(p) {
            panic!("Invalid path literal {}", p);
        }

        if p.is_empty() {
            ItemPathBuf::default()
        } else {
            let path = p.to_owned();
            let mut segs = Vec::new();

            for (i, c) in path.char_indices() {
                if c == '/' {
                    segs.push(PathSegment { id_tail: i });
                }
            }
            segs.push(PathSegment {
                id_tail: path.len(),
            });

            ItemPathBuf { path, segs }
        }
    }

    pub fn push(&mut self, seg: &str) {
        if !SEG_RULE.is_match(seg) {
            panic!("Invalid segment {} in path", seg);
        }

        if !self.is_empty() {
            self.path.push('/');
        }
        self.path.push_str(seg);
        self.segs.push(PathSegment {
            id_tail: self.path.len(),
        });
    }

    pub fn as_slice(&self) -> ItemPath {
        ItemPath {
            path: self,
            start: 0,
            to: self.len(),
        }
    }

    pub fn get_super(&self) -> ItemPath {
        if self.is_empty() {
            panic!("Empty path has no super");
        } else {
            ItemPath {
                path: self,
                start: 0,
                to: self.len() - 1,
            }
        }
    }

    fn index(&self, idx: usize) -> &str {
        let start = if idx == 0 {
            0
        } else {
            self.segs[idx - 1].id_tail + 1
        };
        let tail = &self.segs[idx];
        &self.path[start..tail.id_tail]
    }

    pub fn pop(&mut self) -> Result<(), &'static str> {
        if self.is_empty() {
            Err("Too short path")
        } else {
            self.segs.remove(self.len() - 1);
            self.path.truncate(self.segs.last().unwrap().id_tail);
            Ok(())
        }
    }
}

impl IItemPath for ItemPathBuf {
    fn len(&self) -> usize {
        self.segs.len()
    }

    fn is_empty(&self) -> bool {
        self.segs.is_empty()
    }

    fn get_self(&self) -> Option<&str> {
        match self.len() {
            0 => None,
            1 => Some(&self.path),
            _ => {
                let start = self.segs[self.segs.len() - 2].id_tail + 1;
                let end = self.segs[self.segs.len() - 1].id_tail;
                Some(&self.path[start..end])
            }
        }
    }

    fn get_super(&self) -> Option<&str> {
        match self.len() {
            0 | 1 => None,
            _ => {
                let seg_len = self.segs.len();
                let start = if seg_len == 2 {
                    0
                } else {
                    self.segs[self.segs.len() - 3].id_tail + 1
                };
                let end = self.segs[self.segs.len() - 2].id_tail;
                Some(&self.path[start..end])
            }
        }
    }

    fn get_root(&self) -> Option<&str> {
        match self.len() {
            0 => None,
            1 => Some(&self.path),
            _ => {
                let end = &self.segs[0];
                Some(&self.path[0..end.id_tail])
            }
        }
    }

    fn to_string(self) -> String {
        self.path
    }

    fn as_str(&self) -> &str {
        &self.path
    }

    fn iter(&self) -> ItemPathIter {
        ItemPathIter {
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
            start,
            to: end,
        }
    }
}

impl<'p> ItemPath<'p> {
    pub fn to_owned(&self) -> ItemPathBuf {
        ItemPathBuf {
            path: self.as_str().to_owned(),
            segs: self.path.segs[self.start..self.to].to_vec(),
        }
    }

    pub fn to_super(&mut self) {
        if self.len() <= 1 {
            panic!("Path {} has no super", self.as_str());
        } else {
            self.to -= 1;
        }
    }
}

impl<'p> IItemPath for ItemPath<'p> {
    fn len(&self) -> usize {
        self.to - self.start
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get_self(&self) -> Option<&str> {
        if self.to > self.path.len() {
            None
        } else {
            Some(self.path.index(self.to - 1))
        }
    }

    fn get_super(&self) -> Option<&str> {
        if self.to > self.path.len() || self.to <= 1 {
            None
        } else {
            Some(self.path.index(self.to - 2))
        }
    }

    fn get_root(&self) -> Option<&str> {
        if self.start >= self.path.len() {
            None
        } else {
            Some(self.path.index(self.start))
        }
    }

    fn to_string(self) -> String {
        self.as_str().to_owned()
    }

    fn as_str(&self) -> &str {
        let start = if self.start == 0 {
            0
        } else {
            self.path.segs[self.start - 1].id_tail + 1
        };
        let end = self.path.segs[self.to - 1].id_tail;
        &self.path.path[start..end]
    }

    fn iter(&self) -> ItemPathIter {
        ItemPathIter {
            path: self.path,
            end: self.to,
            cur: self.start,
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
            start: self.start + start,
            to: self.to + end,
        }
    }
}

pub struct ItemPathIter<'p> {
    path: &'p ItemPathBuf,
    end: usize,
    cur: usize,
}

impl<'p> Iterator for ItemPathIter<'p> {
    type Item = &'p str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.end {
            None
        } else {
            self.cur += 1;
            Some(self.path.index(self.cur - 1))
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