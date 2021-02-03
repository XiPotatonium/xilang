use super::super::ast::ast::*;
use super::super::ast::expr::*;
use super::super::err::LangErr::{self, UnexpectedToken};
use super::token::*;
use super::{lexer, token_mgr};
use crate::ir::flag::*;
use std::fs;

pub struct LLParser {
    _tokens: token_mgr::TokenMgr,
    _idx: usize,
}

// Consume one token with the specified tag
// If next token's tag doesn't match the specified tag, throw UnexpectedToken err
macro_rules! consume_by_tag {
    ($parser: ident, $tag: path) => {
        if let $tag = $parser.peek().tag {
            Ok($parser.consume())
        } else {
            Err(UnexpectedToken($tag, $parser.peek().tag))
        }
    };
}

impl LLParser {
    pub fn new(file: fs::File) -> LLParser {
        LLParser {
            _tokens: token_mgr::TokenMgr::new(lexer::Lexer::new(file)),
            _idx: 0,
        }
    }

    fn eof(&self) -> bool {
        self._idx >= self._tokens.len()
    }

    fn peek(&self) -> &Token {
        &self._tokens[self._idx]
    }

    fn consume(&mut self) -> &Token {
        self._idx += 1;
        &self._tokens[self._idx - 1]
    }

    // class
    //  KwClass Id LBraces (func | vars)* RBraces
    fn parse_class(&mut self) -> Result<AST, LangErr> {
        consume_by_tag!(self, TokenTag::KwClass)?;
        let id = consume_by_tag!(self, TokenTag::Id)?
            .literal
            .as_ref()
            .unwrap()
            .clone();

        let mut methods: Vec<AST> = Vec::new();
        let mut fields: Vec<AST> = Vec::new();

        consume_by_tag!(self, TokenTag::LBraces)?;
        loop {
            if let TokenTag::RBraces = self.peek().tag {
                self.consume();
                break;
            }

            match self.peek().tag {
                TokenTag::RBraces => {
                    self.consume();
                    break;
                }
                TokenTag::KwLFn => {
                    methods.push(self.parse_func()?);
                }
                _ => {
                    // Field
                    fields.push(self.parse_field()?);
                }
            }
        }

        Ok(AST::Class(id, methods, fields))
    }

    // type
    //  KwBool | KwI32 | KwF64
    //  Id (DoubleColon Id)*
    //  LParen type (Comma type)* RParen
    //  LBracket type SemiColon expr RBracket
    fn parse_type(&mut self) -> Result<AST, LangErr> {
        let t = self.consume();
        let ty = match t.tag {
            TokenTag::KwBool => Type::Bool,
            TokenTag::KwI32 => Type::Int,
            TokenTag::KwF64 => Type::Double,
            TokenTag::Id => {
                let mut names: Vec<String> = vec![t.literal.as_ref().unwrap().clone()];
                while let TokenTag::DoubleColon = self.peek().tag {
                    self.consume();
                    names.push(
                        consume_by_tag!(self, TokenTag::Id)?
                            .literal
                            .as_ref()
                            .unwrap()
                            .clone(),
                    );
                }
                Type::Class(names)
            }
            TokenTag::LParen => {
                unimplemented!();
            }
            TokenTag::LBracket => {
                unimplemented!();
            }
            _ => {
                return Err(LangErr::Err("Invalid type"));
            }
        };

        Ok(AST::Type(ty))
    }

    // func
    //  KwLFn Id ps (Arrow type)? block
    fn parse_func(&mut self) -> Result<AST, LangErr> {
        consume_by_tag!(self, TokenTag::KwLFn)?;
        let id = consume_by_tag!(self, TokenTag::Id)?
            .literal
            .as_ref()
            .unwrap()
            .clone();
        let ps = self.parse_ps()?;

        let debug_tag = self.peek().tag;
        let ty = if let TokenTag::Arrow = debug_tag {
            self.consume();
            self.parse_type()?
        } else {
            AST::Type(Type::empty())
        };

        Ok(AST::Func(
            id,
            Box::new(ty),
            ps,
            Box::new(self.parse_block()?),
        ))
    }

    // ps
    //  LParen ((KwLSelf | Id Colon type) (Comma Id Colon type)*)? RParen
    fn parse_ps(&mut self) -> Result<Vec<AST>, LangErr> {
        consume_by_tag!(self, TokenTag::LParen)?;
        let mut ps: Vec<AST> = Vec::new();

        match self.peek().tag {
            TokenTag::KwLSelf => {
                self.consume();
                ps.push(AST::Var(
                    String::from("self"),
                    Box::new(AST::Type(Type::Unk)),
                    Flag::default(),
                    Box::new(AST::None),
                ))
            }
            TokenTag::Id => {
                let id = self.consume().literal.as_ref().unwrap().clone();

                consume_by_tag!(self, TokenTag::Colon)?;

                let ty = self.parse_type()?;
                // default value for param is not supported
                ps.push(AST::Var(
                    id,
                    Box::new(ty),
                    Flag::default(),
                    Box::new(AST::None),
                ))
            }
            TokenTag::RParen => {
                // empty ps
                self.consume();
                return Ok(ps);
            }
            _ => {
                return Err(LangErr::Err("Invalid ps"));
            }
        }

        loop {
            let token = self.consume();

            match token.tag {
                TokenTag::Comma => {} // valid deli
                TokenTag::RParen => {
                    break;
                } // end of ps
                _ => {
                    return Err(LangErr::Err("Invalid delimeter in parameters"));
                }
            }

            let id = consume_by_tag!(self, TokenTag::Id)?
                .literal
                .as_ref()
                .unwrap()
                .clone();

            consume_by_tag!(self, TokenTag::Colon)?;

            let ty = self.parse_type()?;
            // default value for param is not supported
            ps.push(AST::Var(
                id,
                Box::new(ty),
                Flag::default(),
                Box::new(AST::None),
            ));
        }

        Ok(ps)
    }

    // pattern
    //  ident_pattern
    //  tuple_pattern
    fn parse_pattern(&mut self) -> Result<AST, LangErr> {
        match self.peek().tag {
            TokenTag::Id => self.parse_ident_pattern(),
            TokenTag::LParen => self.parse_tuple_pattern(),
            _ => Err(LangErr::Err("Unknown pattern")),
        }
    }

    // ident_pattern
    //  Id
    fn parse_ident_pattern(&mut self) -> Result<AST, LangErr> {
        Ok(AST::Id(
            consume_by_tag!(self, TokenTag::Id)?
                .literal
                .as_ref()
                .unwrap()
                .clone(),
        ))
    }

    // tuple_pattern
    //  LParen pattern (Comma pattern)* Comma? RParen
    fn parse_tuple_pattern(&mut self) -> Result<AST, LangErr> {
        unimplemented!();
    }

    // Field
    //  (KwStatic | KwLet) ident-pattern Colon type SemiColon
    fn parse_field(&mut self) -> Result<AST, LangErr> {
        let mut flag = Flag::default();
        match self.consume().tag {
            TokenTag::KwStatic => {
                flag.set(FlagTag::Static);
            }
            TokenTag::KwLet => (),
            _ => return Err(LangErr::Err("Invalid field declaration")),
        }

        let id = self.parse_ident_pattern()?;

        consume_by_tag!(self, TokenTag::Colon)?;
        let ty = self.parse_type()?;

        consume_by_tag!(self, TokenTag::SemiColon)?;

        Ok(AST::Var(
            if let AST::Id(id_) = id {
                id_
            } else {
                panic!("Parser error!")
            },
            Box::new(ty),
            flag,
            Box::new(AST::None),
        ))
    }

    // var
    //  KwLet pattern (Colon type)? (Assign expr)? SemiColon
    fn parse_vars(&mut self) -> Result<Vec<AST>, LangErr> {
        consume_by_tag!(self, TokenTag::KwLet)?;
        let pattern = self.parse_pattern()?;

        let ty = if let TokenTag::Colon = self.peek().tag {
            self.consume();
            self.parse_type()?
        } else {
            AST::Type(Type::Unk)
        };

        let init = if let TokenTag::Assign = self.peek().tag {
            self.consume();
            self.parse_expr()?
        } else {
            AST::None
        };

        consume_by_tag!(self, TokenTag::SemiColon)?;

        match pattern {
            AST::Id(id) => Ok(vec![AST::Var(
                id,
                Box::new(ty),
                Flag::default(),
                Box::new(init),
            )]),
            _ => Err(LangErr::Err("Invalid pattern")),
        }
    }

    // block
    //  LBraces stmt* RBraces
    // stmt
    //  for | while | loop| if | continue | break | return | block | var | expr (SemiColon)?
    fn parse_block(&mut self) -> Result<AST, LangErr> {
        let mut children: Vec<AST> = Vec::new();

        consume_by_tag!(self, TokenTag::LBraces)?;
        loop {
            let t = self.peek();
            match t.tag {
                TokenTag::RBraces => {
                    self.consume();
                    break;
                }
                _ => {
                    match t.tag {
                        TokenTag::KwFor => children.push(self.parse_for()?),
                        TokenTag::KwWhile => children.push(self.parse_while()?),
                        TokenTag::KwLoop => children.push(self.parse_loop()?),
                        TokenTag::KwIf => children.push(self.parse_if()?),
                        TokenTag::KwContinue => {
                            self.consume();
                            consume_by_tag!(self, TokenTag::SemiColon)?;
                            children.push(AST::Continue);
                        }
                        TokenTag::KwBreak => {
                            self.consume();
                            consume_by_tag!(self, TokenTag::SemiColon)?;
                            children.push(AST::Break);
                        }
                        TokenTag::KwReturn => {
                            self.consume();
                            children.push(if let TokenTag::SemiColon = self.peek().tag {
                                self.consume();
                                AST::Return(Box::new(AST::None))
                            } else {
                                // Return with value
                                let ret_v = self.parse_expr()?;
                                consume_by_tag!(self, TokenTag::SemiColon)?;
                                AST::Return(Box::new(ret_v))
                            });
                        }
                        TokenTag::LBraces => children.push(self.parse_block()?),
                        TokenTag::KwLet => children.append(&mut self.parse_vars()?),
                        _ => {
                            let expr = self.parse_expr()?;
                            children.push(match self.peek().tag {
                                TokenTag::SemiColon => {
                                    self.consume();
                                    AST::ExprStmt(Box::new(expr))
                                }
                                TokenTag::RBraces => expr,
                                _ => return Err(LangErr::Err("Invalid expression or statement")),
                            });
                        }
                    };
                }
            }
        }

        Ok(AST::Block(children))
    }

    // if
    //  KwIf expr block (KwElse (block | if))?
    fn parse_if(&mut self) -> Result<AST, LangErr> {
        consume_by_tag!(self, TokenTag::KwIf)?;
        let cond = self.parse_expr()?;
        let then = self.parse_block()?;

        Ok(AST::If(
            Box::new(cond),
            Box::new(then),
            Box::new(if let TokenTag::KwElse = self.peek().tag {
                self.consume();
                if let TokenTag::LBraces = self.peek().tag {
                    self.parse_block()?
                } else {
                    self.parse_if()?
                }
            } else {
                AST::None
            }),
        ))
    }

    // for
    //  KwFor pattern KwIn expr block
    fn parse_for(&mut self) -> Result<AST, LangErr> {
        consume_by_tag!(self, TokenTag::KwFor)?;

        let pattern = self.parse_pattern()?;

        consume_by_tag!(self, TokenTag::KwIn)?;
        let iter = self.parse_expr()?;

        Ok(AST::For(
            Box::new(pattern),
            Box::new(iter),
            Box::new(self.parse_block()?),
        ))
    }

    // while
    //  KwWhile expr block
    fn parse_while(&mut self) -> Result<AST, LangErr> {
        consume_by_tag!(self, TokenTag::KwWhile)?;
        let cond = self.parse_expr()?;
        Ok(AST::While(Box::new(cond), Box::new(self.parse_block()?)))
    }

    // loop
    // KwLoop block
    fn parse_loop(&mut self) -> Result<AST, LangErr> {
        consume_by_tag!(self, TokenTag::KwLoop)?;
        Ok(AST::Loop(Box::new(self.parse_block()?)))
    }

    // expr(assign_expr)
    //     log_or_expr (ANY_ASSIGN expr)?
    fn parse_expr(&mut self) -> Result<AST, LangErr> {
        let lhs = self.parse_log_or_expr()?;
        Ok(AST::Binary(
            match self.peek().tag {
                TokenTag::Assign => {
                    self.consume();
                    Op::Assign
                }
                TokenTag::AddAssign => {
                    self.consume();
                    Op::AddAssign
                }
                TokenTag::SubAssign => {
                    self.consume();
                    Op::SubAssign
                }
                TokenTag::MulAssign => {
                    self.consume();
                    Op::MulAssign
                }
                TokenTag::DivAssign => {
                    self.consume();
                    Op::DivAssign
                }
                TokenTag::ModAssign => {
                    self.consume();
                    Op::ModAssign
                }
                _ => return Ok(lhs),
            },
            Box::new(lhs),
            Box::new(self.parse_expr()?),
        ))
    }

    // log_or_expr
    //  log_and_expr (LogOr log_or_expr)?
    fn parse_log_or_expr(&mut self) -> Result<AST, LangErr> {
        let lhs = self.parse_log_and_expr()?;
        if let TokenTag::LogOr = self.peek().tag {
            self.consume();
            Ok(AST::Binary(
                Op::LogOr,
                Box::new(lhs),
                Box::new(self.parse_log_or_expr()?),
            ))
        } else {
            Ok(lhs)
        }
    }

    // log_and_expr
    //  eq_expr (LogAnd log_and_expr)?
    fn parse_log_and_expr(&mut self) -> Result<AST, LangErr> {
        let lhs = self.parse_eq_expr()?;
        if let TokenTag::LogAnd = self.peek().tag {
            self.consume();
            Ok(AST::Binary(
                Op::LogAnd,
                Box::new(lhs),
                Box::new(self.parse_log_and_expr()?),
            ))
        } else {
            Ok(lhs)
        }
    }

    // eq_expr
    //  compare_expr ((Eq | Ne) compare_expr)?
    fn parse_eq_expr(&mut self) -> Result<AST, LangErr> {
        let lhs = self.parse_compare_expr()?;
        Ok(match self.peek().tag {
            TokenTag::Eq => {
                self.consume();
                AST::Binary(Op::Eq, Box::new(lhs), Box::new(self.parse_eq_expr()?))
            }
            TokenTag::Ne => {
                self.consume();
                AST::Binary(Op::Ne, Box::new(lhs), Box::new(self.parse_eq_expr()?))
            }
            _ => lhs,
        })
    }

    // compare_expr
    //  add_expr ((Le | Lt | Ge | Gt) add_expr)?
    fn parse_compare_expr(&mut self) -> Result<AST, LangErr> {
        let lhs = self.parse_add_expr()?;
        Ok(match self.peek().tag {
            TokenTag::Le => {
                self.consume();
                AST::Binary(Op::Le, Box::new(lhs), Box::new(self.parse_add_expr()?))
            }
            TokenTag::Lt => {
                self.consume();
                AST::Binary(Op::Lt, Box::new(lhs), Box::new(self.parse_add_expr()?))
            }
            TokenTag::Ge => {
                self.consume();
                AST::Binary(Op::Ge, Box::new(lhs), Box::new(self.parse_add_expr()?))
            }
            TokenTag::Gt => {
                self.consume();
                AST::Binary(Op::Gt, Box::new(lhs), Box::new(self.parse_add_expr()?))
            }
            _ => lhs,
        })
    }

    // add_expr
    //  mul_expr ((Add | Sub) mul_expr)*
    fn parse_add_expr(&mut self) -> Result<AST, LangErr> {
        let lhs = self.parse_mul_expr()?;
        Ok(match self.peek().tag {
            TokenTag::Add => {
                self.consume();
                AST::Binary(Op::Add, Box::new(lhs), Box::new(self.parse_add_expr()?))
            }
            TokenTag::Sub => {
                self.consume();
                AST::Binary(Op::Sub, Box::new(lhs), Box::new(self.parse_add_expr()?))
            }
            _ => lhs,
        })
    }

    // mul_expr
    //  cast_or_unary_expr ((Mul | Div | Mod) cast_expr)*
    fn parse_mul_expr(&mut self) -> Result<AST, LangErr> {
        let lhs = self.parse_cast_or_unary_expr()?;
        Ok(match self.peek().tag {
            TokenTag::Mul => {
                self.consume();
                AST::Binary(Op::Mul, Box::new(lhs), Box::new(self.parse_mul_expr()?))
            }
            TokenTag::Div => {
                self.consume();
                AST::Binary(Op::Div, Box::new(lhs), Box::new(self.parse_mul_expr()?))
            }
            TokenTag::Mod => {
                self.consume();
                AST::Binary(Op::Mod, Box::new(lhs), Box::new(self.parse_mul_expr()?))
            }
            _ => lhs,
        })
    }

    // cast_or_unary_expr
    //  cast_expr | unary
    fn parse_cast_or_unary_expr(&mut self) -> Result<AST, LangErr> {
        // Deconflict with parenthesis expression in UnaryExpr
        // (a as T) is Cast Expression
        // (a) + b is unary expression
        let prev_idx = self._idx;
        let try_cast = self.parse_cast_expr();
        match try_cast {
            Ok(cast) => Ok(cast),
            Err(_) => {
                // resume and parse unary
                self._idx = prev_idx;
                self.parse_unary()
            }
        }
    }

    // cast_expr
    //  LParen cast_or_unary_expr KwAs type RParen
    fn parse_cast_expr(&mut self) -> Result<AST, LangErr> {
        consume_by_tag!(self, TokenTag::LParen)?;
        let value = self.parse_cast_or_unary_expr()?;
        consume_by_tag!(self, TokenTag::KwAs)?;
        let ty = self.parse_type()?;
        consume_by_tag!(self, TokenTag::RParen)?;
        Ok(AST::Cast(Box::new(ty), Box::new(value)))
    }

    // unary
    //  (LogNot | Sub) cast_or_unary_expr | call
    fn parse_unary(&mut self) -> Result<AST, LangErr> {
        match self.peek().tag {
            TokenTag::LogNot => {
                self.consume();
                Ok(AST::Unary(
                    Op::LogNot,
                    Box::new(self.parse_cast_or_unary_expr()?),
                ))
            }
            TokenTag::Sub => {
                self.consume();
                Ok(AST::Unary(
                    Op::Neg,
                    Box::new(self.parse_cast_or_unary_expr()?),
                ))
            }
            _ => Ok(self.parse_call()?),
        }
    }

    // call
    //  primary (LParen (expr (Comma expr)*)? RParen | Dot Id | DoubleColon Id | LBracket expr RBracket)*
    fn parse_call(&mut self) -> Result<AST, LangErr> {
        let mut lhs = self.parse_primary()?;
        loop {
            match self.peek().tag {
                TokenTag::LParen => {
                    // call
                    self.consume();
                    let mut ps: Vec<AST> = Vec::new();
                    loop {
                        // args
                        if let TokenTag::RParen = self.peek().tag {
                            break;
                        }

                        ps.push(self.parse_expr()?);

                        if let TokenTag::Comma = self.peek().tag {
                            self.consume();
                        } else {
                            break;
                        }
                    }
                    consume_by_tag!(self, TokenTag::RParen)?;
                    lhs = AST::Call(Box::new(lhs), ps)
                }
                TokenTag::Dot => {
                    // access
                    self.consume();
                    lhs = AST::Binary(
                        Op::ClassAccess,
                        Box::new(lhs),
                        Box::new(AST::Id(
                            consume_by_tag!(self, TokenTag::Id)?
                                .literal
                                .as_ref()
                                .unwrap()
                                .clone(),
                        )),
                    )
                }
                TokenTag::DoubleColon => {
                    // static access
                    self.consume();
                    lhs = AST::Binary(
                        Op::StaticAccess,
                        Box::new(lhs),
                        Box::new(AST::Id(
                            consume_by_tag!(self, TokenTag::Id)?
                                .literal
                                .as_ref()
                                .unwrap()
                                .clone(),
                        )),
                    )
                }
                TokenTag::LBracket => {
                    // array access
                    self.consume();
                    lhs = AST::Binary(Op::ArrayAccess, Box::new(lhs), Box::new(self.parse_expr()?));
                    consume_by_tag!(self, TokenTag::RBracket)?;
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    // primary
    //  literal | Id | LParen expr RParen | KwLSelf | KwNew type
    fn parse_primary(&mut self) -> Result<AST, LangErr> {
        Ok(match self.peek().tag {
            TokenTag::Id => AST::Id(self.consume().literal.as_ref().unwrap().clone()),
            // "this" behaves like tag here
            TokenTag::KwLSelf => {
                self.consume();
                AST::Id(String::from("self"))
            }
            TokenTag::LParen => {
                self.consume();
                let sub_expr = self.parse_expr()?;
                consume_by_tag!(self, TokenTag::RParen)?;
                sub_expr
            }
            TokenTag::KwNew => {
                self.consume();
                AST::New(Box::new(self.parse_type()?))
            }
            _ => self.parse_literal()?,
        })
    }

    // literal
    //  KwTrue | KwFalse | KwNull | DecLiteral | FpLiteral | StrLiteral | ChLiteral
    fn parse_literal(&mut self) -> Result<AST, LangErr> {
        Ok(match self.peek().tag {
            TokenTag::KwTrue => {
                self.consume();
                AST::Bool(true)
            }
            TokenTag::KwFalse => {
                self.consume();
                AST::Bool(false)
            }
            TokenTag::KwNull => {
                self.consume();
                AST::Null
            }
            TokenTag::DecLiteral => match self.consume().literal.as_ref().unwrap().parse::<i32>() {
                Ok(val) => AST::Int(val),
                Err(_) => return Err(LangErr::Err("Parse to i32 failed")),
            },
            TokenTag::FpLiteral => match self.consume().literal.as_ref().unwrap().parse::<f64>() {
                Ok(val) => AST::Float(val),
                Err(_) => return Err(LangErr::Err("Parse to f64 failed")),
            },
            TokenTag::StrLiteral => {
                let mut chars = self.consume().literal.as_ref().unwrap().chars();
                chars.next(); // skip first '"'
                let mut s = String::new();
                loop {
                    match chars.next() {
                        Some(ch) => {
                            match ch {
                                '\\' => {
                                    // escape
                                    match chars.next() {
                                        Some(ch) => match ch {
                                            'n' => s.push('\n'),
                                            _ => unimplemented!("Unsupported escape char"),
                                        },
                                        None => return Err(LangErr::Err("Invalid string literal")),
                                    }
                                }
                                '"' => break,
                                _ => s.push(ch),
                            }
                        }
                        None => unreachable!(),
                    }
                }
                AST::String(s)
            }
            TokenTag::ChLiteral => {
                let mut chars = self.consume().literal.as_ref().unwrap().chars();
                chars.next(); // skip first '\''
                let ch = AST::Char(match chars.next().unwrap() {
                    '\'' => return Err(LangErr::Err("Empty char literal")),
                    '\\' => chars.next().unwrap().into(),
                    c => c.into(),
                });
                match chars.next() {
                    Some('\'') => (),
                    Some(_) => return Err(LangErr::Err("Too many chars in char literal")),
                    None => return Err(LangErr::Err("Invalid char literal")),
                }
                ch
            }
            _ => return Err(LangErr::Err("Invalid literal")),
        })
    }

    pub fn parse(&mut self) -> Result<AST, LangErr> {
        let mut classes: Vec<AST> = Vec::new();

        while !self.eof() {
            classes.push(self.parse_class()?);
        }

        Ok(AST::File(classes))
    }
}
