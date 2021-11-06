use std::fmt;

use lazy_static::lazy_static;
use regex::Regex;

use super::super::ast::ASTType;

const ID_RULE: &str = r"^[_a-zA-Z][_a-zA-Z0-9]*";

lazy_static! {
    // Same as identifier
    static ref SEG_RULE : Regex = Regex::new(ID_RULE).unwrap();
    static ref PATH_RULE: Regex = Regex::new(&format!("{}(/{})*", ID_RULE, ID_RULE)).unwrap();
}

#[derive(Clone)]
struct PathSegment {
    id_tail: usize,
    generic_params: Option<Vec<Box<ASTType>>>,
}

pub trait IItemPath {
    fn len(&self) -> usize;
    fn get_self(&self) -> Option<(&str, &Option<Vec<Box<ASTType>>>)>;
    fn get_super(&self) -> Option<(&str, &Option<Vec<Box<ASTType>>>)>;
    fn get_root(&self) -> Option<(&str, &Option<Vec<Box<ASTType>>>)>;
    fn to_string(self) -> String;
    fn as_str(&self) -> &str;
    fn iter(&self) -> ItemPathIter;
    fn range(&self, start: usize, end: usize) -> ItemPath;
}

#[derive(Clone)]
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
    pub fn new() -> ItemPathBuf {
        ItemPathBuf {
            path: String::new(),
            segs: Vec::new(),
        }
    }

    /// p must be module fullname, no generic parameter
    pub fn from_module(p: &str) -> ItemPathBuf {
        if !PATH_RULE.is_match(p) {
            panic!("Invalid path literal {}", p);
        }

        if p.is_empty() {
            ItemPathBuf::new()
        } else {
            let path = p.to_owned();
            let mut segs = Vec::new();

            for (i, c) in path.char_indices() {
                if c == '/' {
                    segs.push(PathSegment {
                        id_tail: i,
                        generic_params: None,
                    });
                }
            }
            segs.push(PathSegment {
                id_tail: path.len(),
                generic_params: None,
            });

            ItemPathBuf { path, segs }
        }
    }

    pub fn push(&mut self, seg: &str) {
        if !SEG_RULE.is_match(seg) {
            panic!("Invalid segment {} in path", seg);
        }

        if self.len() != 0 {
            self.path.push('/');
        }
        self.path.push_str(seg);
        self.segs.push(PathSegment {
            id_tail: self.path.len(),
            generic_params: None,
        });
    }

    pub fn push_id_with_generic(&mut self, id: &str, generic_params: Option<Vec<Box<ASTType>>>) {
        if !SEG_RULE.is_match(id) {
            panic!("Invalid segment {} in path", id);
        }

        if self.len() != 0 {
            self.path.push('/');
        }
        self.path.push_str(id);
        self.segs.push(PathSegment {
            id_tail: self.path.len(),
            generic_params,
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
        if self.len() == 0 {
            panic!("Empty path has no super");
        } else {
            ItemPath {
                path: self,
                start: 0,
                to: self.len() - 1,
            }
        }
    }

    /// Canonicalize path
    ///
    /// A valid canonicalized path: `("crate"? | "super"*) ~ Id*`
    ///
    /// Return:
    pub fn canonicalize(&self) -> (bool, usize, ItemPathBuf) {
        let mut has_crate: bool = false;
        let mut super_count = 0;
        let mut segs: Vec<(&str, Option<Vec<Box<ASTType>>>)> = Vec::new();
        for (i, seg) in self.segs.iter().enumerate() {
            let (id, _) = self.index(i);
            if id == "crate" {
                if i == 0 {
                    has_crate = true;
                    segs.push(("crate", None));
                } else {
                    panic!(
                        "crate should be the first segment in path {}",
                        self.as_str()
                    );
                }
            } else if id == "super" {
                if segs.is_empty() {
                    super_count += 1;
                    segs.push(("super", None));
                } else if segs.last().unwrap().0 == "crate" {
                    panic!("Super of crate is invalid");
                } else if segs.last().unwrap().0 == "super" {
                    super_count += 1;
                    segs.push(("super", None));
                } else {
                    // remove last
                    segs.remove(segs.len() - 1);
                }
            } else if id == "self" {
                continue;
            } else {
                segs.push((id, seg.generic_params.clone()));
            }
        }
        let mut path = ItemPathBuf::new();
        for (id, generic_params) in segs.into_iter() {
            path.push_id_with_generic(id, generic_params);
        }
        (has_crate, super_count, path)
    }

    fn index(&self, idx: usize) -> (&str, &Option<Vec<Box<ASTType>>>) {
        let start = if idx == 0 {
            0
        } else {
            self.segs[idx - 1].id_tail + 1
        };
        let tail = &self.segs[idx];
        (&self.path[start..tail.id_tail], &tail.generic_params)
    }
}

impl IItemPath for ItemPathBuf {
    fn len(&self) -> usize {
        self.segs.len()
    }

    fn get_self(&self) -> Option<(&str, &Option<Vec<Box<ASTType>>>)> {
        match self.len() {
            0 => None,
            1 => Some((&self.path, &self.segs[0].generic_params)),
            _ => {
                let start = self.segs[self.segs.len() - 2].id_tail + 1;
                let end = self.segs[self.segs.len() - 1].id_tail;
                Some((
                    &self.path[start..end],
                    &self.segs[self.segs.len() - 1].generic_params,
                ))
            }
        }
    }

    fn get_super(&self) -> Option<(&str, &Option<Vec<Box<ASTType>>>)> {
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
                Some((
                    &self.path[start..end],
                    &self.segs[self.segs.len() - 1].generic_params,
                ))
            }
        }
    }

    fn get_root(&self) -> Option<(&str, &Option<Vec<Box<ASTType>>>)> {
        match self.len() {
            0 => None,
            1 => Some((&self.path, &self.segs[0].generic_params)),
            _ => {
                let end = &self.segs[0];
                Some((&self.path[0..end.id_tail], &end.generic_params))
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
    pub fn to_owned(self) -> ItemPathBuf {
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

    fn get_self(&self) -> Option<(&str, &Option<Vec<Box<ASTType>>>)> {
        if self.to > self.path.len() {
            None
        } else {
            Some(self.path.index(self.to - 1))
        }
    }

    fn get_super(&self) -> Option<(&str, &Option<Vec<Box<ASTType>>>)> {
        if self.to > self.path.len() || self.to <= 1 {
            None
        } else {
            Some(self.path.index(self.to - 2))
        }
    }

    fn get_root(&self) -> Option<(&str, &Option<Vec<Box<ASTType>>>)> {
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
    type Item = (&'p str, &'p Option<Vec<Box<ASTType>>>);

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
