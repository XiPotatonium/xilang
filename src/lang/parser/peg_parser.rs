use super::super::ast::ast::{Op, AST};
use crate::ir::flag::{Flag, FlagTag};

use std::fs;
use std::path::PathBuf;

use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "lang/parser/grammar.pest"]
pub struct LRParser;

pub fn parse(path: &PathBuf) -> Result<Box<AST>, Error<Rule>> {
    let code = fs::read_to_string(path).unwrap();

    let file = LRParser::parse(Rule::File, &code)?.next().unwrap();

    let mut classes: Vec<Box<AST>> = Vec::new();
    for sub in file.into_inner() {
        match sub.as_rule() {
            Rule::EOI => break,
            Rule::Class => classes.push(build_class(sub)),
            _ => unreachable!(),
        };
    }

    Ok(Box::new(AST::File(classes)))
}

fn build_class(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let id = build_id(iter.next().unwrap());
    let mut fields: Vec<Box<AST>> = Vec::new();
    let mut methods: Vec<Box<AST>> = Vec::new();
    for sub in iter {
        match sub.as_rule() {
            Rule::StaticField => fields.push(build_field(sub, true)),
            Rule::NonStaticField => fields.push(build_field(sub, false)),
            Rule::Func => methods.push(build_func(sub)),
            _ => unreachable!(),
        }
    }

    Box::new(AST::Class(id, methods, fields))
}

fn build_field(tree: Pair<Rule>, is_static: bool) -> Box<AST> {
    let mut iter = tree.into_inner();
    let id = build_id(iter.next().unwrap());
    let mut flag = Flag::default();
    if is_static {
        flag.set(FlagTag::Static);
    }

    Box::new(AST::Field(id, build_type(iter.next().unwrap()), flag))
}

fn build_func(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let id = build_id(iter.next().unwrap());

    let mut ps: Vec<Box<AST>> = Vec::new();
    let mut sub = iter.next().unwrap();
    if let Rule::Params = sub.as_rule() {
        // Build parameters
        let mut p_iter = sub.into_inner();
        let p0 = p_iter.next().unwrap();
        ps.push(Box::new(match p0.as_rule() {
            Rule::SelfParam => {
                AST::Param(String::from("self"), Box::new(AST::None), Flag::default())
            }
            Rule::Id => AST::Param(
                build_id(p0),
                build_type(p_iter.next().unwrap()),
                Flag::default(),
            ),
            _ => unreachable!(),
        }));

        loop {
            if let Some(p_id) = p_iter.next() {
                ps.push(Box::new(AST::Param(
                    build_id(p_id),
                    build_type(p_iter.next().unwrap()),
                    Flag::default(),
                )));
            } else {
                break;
            }
        }
        sub = iter.next().unwrap();
    }

    let ret_type = if let Rule::BlockExpr = sub.as_rule() {
        Box::new(AST::None)
    } else {
        let ret_type = build_type(sub);
        sub = iter.next().unwrap();
        ret_type
    };

    Box::new(AST::Func(id, ret_type, ps, build_block(sub)))
}

fn build_type(tree: Pair<Rule>) -> Box<AST> {
    let tree = tree.into_inner().next().unwrap();
    let ret = Box::new(match tree.as_rule() {
        Rule::BoolType => AST::BoolType,
        Rule::I32Type => AST::I32Type,
        Rule::F64Type => AST::F64Type,
        Rule::PathExpr => AST::ClassType(tree.into_inner().map(|id| build_id(id)).collect()),
        Rule::TupleType => AST::TupleType(tree.into_inner().map(|ty| build_type(ty)).collect()),
        Rule::ArrType => {
            let mut iter = tree.into_inner();
            let sub_ty = build_type(iter.next().unwrap());
            AST::ArrType(sub_ty, build_expr(iter.next().unwrap()))
        }
        _ => unreachable!(format!("Found {:?}", tree.as_rule())),
    });
    ret
}

fn build_expr(tree: Pair<Rule>) -> Box<AST> {
    match tree.as_rule() {
        Rule::BlockExpr => build_block(tree),
        Rule::LoopExpr => Box::new(AST::Loop(build_block(tree.into_inner().next().unwrap()))),
        Rule::IfExpr => build_if(tree),
        Rule::ContinueExpr => Box::new(AST::Continue),
        Rule::BreakExpr => Box::new(AST::Break(if let Some(ret_v) = tree.into_inner().next() {
            build_expr(ret_v)
        } else {
            Box::new(AST::None)
        })),
        Rule::ReturnExpr => Box::new(AST::Return(if let Some(ret_v) = tree.into_inner().next() {
            build_expr(ret_v)
        } else {
            Box::new(AST::None)
        })),
        Rule::AssignExpr => build_assign(tree),
        Rule::OpExpr => {
            let tree = tree.into_inner().next().unwrap();
            assert_eq!(tree.as_rule(), Rule::LogOrExpr);
            build_log_or_expr(tree)
        }
        _ => unreachable!(format!("Found {:?}", tree.as_rule())),
    }
}

fn build_if(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let cond = build_expr(iter.next().unwrap());
    let then = build_block(iter.next().unwrap());
    let els = if let Some(els) = iter.next() {
        match els.as_rule() {
            Rule::IfExpr => build_if(els),
            Rule::BlockExpr => build_block(els),
            _ => unreachable!(),
        }
    } else {
        Box::new(AST::None)
    };
    Box::new(AST::If(cond, then, els))
}

fn build_block(tree: Pair<Rule>) -> Box<AST> {
    Box::new(AST::Block(
        tree.into_inner()
            .map(|sub| {
                match sub.as_rule() {
                    Rule::LetStmt => {
                        let mut iter = sub.into_inner();
                        let pattern = build_pattern(iter.next().unwrap());
                        Box::new(if let Some(sub) = iter.next() {
                            match sub.as_rule() {
                                Rule::Type => {
                                    // has type
                                    let ty = build_type(sub);
                                    if let Some(sub) = iter.next() {
                                        AST::Let(pattern, ty, Flag::default(), build_expr(sub))
                                    } else {
                                        // no init
                                        AST::Let(pattern, ty, Flag::default(), Box::new(AST::None))
                                    }
                                }
                                _ => {
                                    // no type but has init
                                    AST::Let(
                                        pattern,
                                        Box::new(AST::None),
                                        Flag::default(),
                                        build_expr(sub),
                                    )
                                }
                            }
                        } else {
                            // no type and no init
                            AST::Let(
                                pattern,
                                Box::new(AST::None),
                                Flag::default(),
                                Box::new(AST::None),
                            )
                        })
                    }
                    _ => build_expr(sub),
                }
            })
            .collect(),
    ))
}

fn build_assign(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let lhs = build_log_or_expr(iter.next().unwrap());
    iter.next();
    Box::new(AST::Binary(
        Op::Assign,
        lhs,
        build_log_or_expr(iter.next().unwrap()),
    ))
}

fn build_log_or_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_log_and_expr(iter.next().unwrap());

    for rhs in iter {
        // log or is left associative
        ret = Box::new(AST::Binary(Op::LogOr, ret, build_log_and_expr(rhs)));
    }
    ret
}

fn build_log_and_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_eq_expr(iter.next().unwrap());

    for rhs in iter {
        ret = Box::new(AST::Binary(Op::LogAnd, ret, build_eq_expr(rhs)));
    }
    ret
}

fn build_eq_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_comp_expr(iter.next().unwrap());

    loop {
        if let Some(op) = iter.next() {
            ret = Box::new(AST::Binary(
                match op.as_rule() {
                    Rule::EqOp => Op::Eq,
                    Rule::NeOp => Op::Ne,
                    _ => unreachable!(),
                },
                ret,
                build_comp_expr(iter.next().unwrap()),
            ));
        } else {
            break;
        }
    }
    ret
}

fn build_comp_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_add_expr(iter.next().unwrap());

    loop {
        if let Some(op) = iter.next() {
            ret = Box::new(AST::Binary(
                match op.as_rule() {
                    Rule::LeOp => Op::Le,
                    Rule::LtOp => Op::Lt,
                    Rule::GeOp => Op::Ge,
                    Rule::GtOp => Op::Gt,
                    _ => unreachable!(),
                },
                ret,
                build_add_expr(iter.next().unwrap()),
            ));
        } else {
            break;
        }
    }
    ret
}

fn build_add_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_mul_expr(iter.next().unwrap());

    loop {
        if let Some(op) = iter.next() {
            ret = Box::new(AST::Binary(
                match op.as_rule() {
                    Rule::AddOp => Op::Add,
                    Rule::SubOp => Op::Sub,
                    _ => unreachable!(),
                },
                ret,
                build_mul_expr(iter.next().unwrap()),
            ));
        } else {
            break;
        }
    }
    ret
}

fn build_mul_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_cast_expr(iter.next().unwrap());

    loop {
        if let Some(op) = iter.next() {
            ret = Box::new(AST::Binary(
                match op.as_rule() {
                    Rule::MulOp => Op::Mul,
                    Rule::DivOp => Op::Div,
                    Rule::ModOp => Op::Mod,
                    _ => unreachable!(),
                },
                ret,
                build_cast_expr(iter.next().unwrap()),
            ));
        } else {
            break;
        }
    }
    ret
}

fn build_cast_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_unary_expr(iter.next().unwrap());

    loop {
        if let Some(rhs) = iter.next() {
            ret = Box::new(AST::Cast(build_type(rhs), ret));
        } else {
            break;
        }
    }
    ret
}

fn build_unary_expr(tree: Pair<Rule>) -> Box<AST> {
    // unary is right associative, iterate reversely
    let mut iter = tree.into_inner().rev();
    let mut ret = build_call_expr(iter.next().unwrap());

    for op in iter {
        ret = Box::new(AST::Unary(
            match op.as_rule() {
                Rule::LogNegOp => Op::LogNot,
                Rule::SubOp => Op::Neg,
                Rule::NewOp => Op::New,
                _ => unreachable!(),
            },
            ret,
        ))
    }
    ret
}

fn build_call_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_primary_expr(iter.next().unwrap());

    for rhs in iter {
        ret = Box::new(match rhs.as_rule() {
            Rule::ArgsExpr => AST::Call(ret, rhs.into_inner().map(|sub| build_expr(sub)).collect()),
            Rule::ObjAccessExpr => AST::Binary(
                Op::ObjAccess,
                ret,
                Box::new(AST::Id(build_id(rhs.into_inner().next().unwrap()))),
            ),
            Rule::PathAccessExpr => AST::Binary(
                Op::StaticAccess,
                ret,
                Box::new(AST::Id(build_id(rhs.into_inner().next().unwrap()))),
            ),
            Rule::ArrAccessExpr => AST::Binary(
                Op::ArrayAccess,
                ret,
                build_expr(rhs.into_inner().next().unwrap()),
            ),
            _ => unreachable!(),
        });
    }
    ret
}

fn build_primary_expr(tree: Pair<Rule>) -> Box<AST> {
    let tree = tree.into_inner().next().unwrap();
    match tree.as_rule() {
        Rule::Id => Box::new(AST::Id(build_id(tree))),
        Rule::GroupedExpr => build_expr(tree.into_inner().next().unwrap()),
        Rule::Type => build_type(tree),
        Rule::LiteralExpr => build_literal(tree),
        // Actually only expr with block
        _ => build_expr(tree),
    }
}

fn build_literal(tree: Pair<Rule>) -> Box<AST> {
    let tree = tree.into_inner().next().unwrap();
    Box::new(match tree.as_rule() {
        Rule::TrueLiteral => AST::Bool(true),
        Rule::FalseLiteral => AST::Bool(false),
        Rule::NullLiteral => AST::Null,
        Rule::EmptyLiteral => AST::None,
        Rule::IntLiteral => AST::Int(tree.as_span().as_str().trim().parse::<i32>().expect(
            &format!("Unable to parse \"{}\" as i32", tree.as_span().as_str()),
        )),
        Rule::FloatLiteral => AST::Float(tree.as_span().as_str().trim().parse::<f64>().unwrap()),
        Rule::StringLiteral => {
            let mut chars = tree.as_span().as_str().trim().chars();
            chars.next(); // skip first '"'
            let mut s = String::new();
            loop {
                match chars.next() {
                    Some(ch) => {
                        match ch {
                            '\\' => {
                                // escape
                                match chars.next().expect("Invalid string literal") {
                                    'n' => s.push('\n'),
                                    _ => unimplemented!("Unsupported escape char"),
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
        Rule::CharLiteral => {
            let mut chars = tree.as_span().as_str().trim().chars();
            chars.next(); // skip first '\''
            let ch = AST::Char(match chars.next().unwrap() {
                '\'' => panic!("Empty char literal"),
                '\\' => chars.next().unwrap().into(),
                c => c.into(),
            });
            match chars.next().expect("Invalid char literal") {
                '\'' => (),
                _ => panic!("Too many chars in char literal"),
            }
            ch
        }
        _ => unreachable!(),
    })
}

fn build_pattern(tree: Pair<Rule>) -> Box<AST> {
    let tree = tree.into_inner().next().unwrap();
    Box::new(match tree.as_rule() {
        Rule::Id => AST::Id(build_id(tree)),
        Rule::TuplePattern => {
            AST::TuplePattern(tree.into_inner().map(|sub| build_pattern(sub)).collect())
        }
        _ => unreachable!(),
    })
}

fn build_id(tree: Pair<Rule>) -> String {
    assert_eq!(tree.as_rule(), Rule::Id);
    String::from(tree.as_span().as_str().trim())
}
