#ifndef XILANG_LANG_GRAMMAR_HPP
#define XILANG_LANG_GRAMMAR_HPP


#include <tao/pegtl.hpp>
#include <tao/pegtl/contrib/raw_string.hpp>
#include <tao/pegtl/internal/pegtl_string.hpp>

namespace lang::grammar {


// not recommended to using namespace in header but we have to do this to remove redundency.
using namespace tao;

/**
 * @brief C-style multi-line comment
 *
 */
struct CComment : pegtl::seq<pegtl::string<'/', '*'>, pegtl::until<pegtl::string<'*', '/'>>> {};
/**
 * @brief A C++ style one-line comment. two consecutive slashes followed by anything up to the end of line or end of
 * file.
 *
 */
struct CppComment : pegtl::seq<pegtl::two<'/'>, pegtl::until<pegtl::eolf>> {};
/**
 * @brief C-style comment and Cpp-style comment
 *
 */
struct Comment : pegtl::sor<CppComment, CComment> {};

struct Sep : pegtl::sor<pegtl::ascii::space, Comment> {};
struct Seps : pegtl::star<Sep> {};

struct StrAs : TAO_PEGTL_STRING("as") {};
struct StrAsync : TAO_PEGTL_STRING("async") {};
struct StrBool : TAO_PEGTL_STRING("bool") {};
struct StrBreak : TAO_PEGTL_STRING("break") {};
struct StrChar : TAO_PEGTL_STRING("char") {};
struct StrConst : TAO_PEGTL_STRING("const") {};
struct StrContinue : TAO_PEGTL_STRING("continue") {};
struct StrCrate : TAO_PEGTL_STRING("crate") {};
struct StrElse : TAO_PEGTL_STRING("else") {};
struct StrEnum : TAO_PEGTL_STRING("enum") {};
struct StrExtern : TAO_PEGTL_STRING("extern") {};
struct StrFalse : TAO_PEGTL_STRING("false") {};
struct StrFor : TAO_PEGTL_STRING("for") {};
struct StrFn : TAO_PEGTL_STRING("fn") {};
struct StrF32 : TAO_PEGTL_STRING("f32") {};
struct StrF64 : TAO_PEGTL_STRING("f64") {};
struct StrIf : TAO_PEGTL_STRING("if") {};
struct StrIn : TAO_PEGTL_STRING("in") {};
struct StrInterface : TAO_PEGTL_STRING("interface") {};
struct StrISize : TAO_PEGTL_STRING("isize") {};
struct StrI32 : TAO_PEGTL_STRING("i32") {};
struct StrI64 : TAO_PEGTL_STRING("i64") {};
struct StrLet : TAO_PEGTL_STRING("let") {};
struct StrMatch : TAO_PEGTL_STRING("match") {};
struct StrMod : TAO_PEGTL_STRING("mod") {};
struct StrMut : TAO_PEGTL_STRING("mut") {};
struct StrNew : TAO_PEGTL_STRING("new") {};
struct StrNull : TAO_PEGTL_STRING("null") {};
struct StrPriv : TAO_PEGTL_STRING("priv") {};
struct StrPub : TAO_PEGTL_STRING("pub") {};
struct StrReturn : TAO_PEGTL_STRING("return") {};
struct StrLSelf : TAO_PEGTL_STRING("self") {};
struct StrUSelf : TAO_PEGTL_STRING("Self") {};
struct StrStruct : TAO_PEGTL_STRING("struct") {};
struct StrString : TAO_PEGTL_STRING("string") {};
struct StrSuper : TAO_PEGTL_STRING("super") {};
struct StrTrue : TAO_PEGTL_STRING("true") {};
struct StrUse : TAO_PEGTL_STRING("use") {};
struct StrUSize : TAO_PEGTL_STRING("usize") {};
struct StrU32 : TAO_PEGTL_STRING("u32") {};
struct StrU64 : TAO_PEGTL_STRING("u64") {};
struct StrU8 : TAO_PEGTL_STRING("u8") {};
struct StrWhere : TAO_PEGTL_STRING("where") {};
struct StrWhile : TAO_PEGTL_STRING("while") {};
struct StrYield : TAO_PEGTL_STRING("yield") {};


template<typename KW>
struct Key : pegtl::seq<KW, pegtl::not_at<pegtl::identifier_other>> {};

struct KwAs : Key<StrAs> {};
struct KwAsync : Key<StrAsync> {};
struct KwBool : Key<StrBool> {};
struct KwBreak : Key<StrBreak> {};
struct KwChar : Key<StrChar> {};
struct KwConst : Key<StrConst> {};
struct KwContinue : Key<StrContinue> {};
struct KwCrate : Key<StrCrate> {};
struct KwElse : Key<StrElse> {};
struct KwEnum : Key<StrEnum> {};
struct KwExtern : Key<StrExtern> {};
struct KwFalse : Key<StrFalse> {};
struct KwFor : Key<StrFor> {};
struct KwFn : Key<StrFn> {};
struct KwF32 : Key<StrF32> {};
struct KwF64 : Key<StrF64> {};
struct KwIf : Key<StrIf> {};
struct KwIn : Key<StrIn> {};
struct KwInterface : Key<StrInterface> {};
struct KwISize : Key<StrISize> {};
struct KwI32 : Key<StrI32> {};
struct KwI64 : Key<StrI64> {};
struct KwLet : Key<StrLet> {};
struct KwMatch : Key<StrMatch> {};
struct KwMod : Key<StrMod> {};
struct KwMut : Key<StrMut> {};
struct KwNew : Key<StrNew> {};
struct KwNull : Key<StrNull> {};
struct KwPriv : Key<StrPriv> {};
struct KwPub : Key<StrPub> {};
struct KwReturn : Key<StrReturn> {};
struct KwLSelf : Key<StrLSelf> {};
struct KwUSelf : Key<StrUSelf> {};
struct KwStruct : Key<StrStruct> {};
struct KwString : Key<StrString> {};
struct KwSuper : Key<StrSuper> {};
struct KwTrue : Key<StrTrue> {};
struct KwUse : Key<StrUse> {};
struct KwUSize : Key<StrUSize> {};
struct KwU32 : Key<StrU32> {};
struct KwU64 : Key<StrU64> {};
struct KwU8 : Key<StrU8> {};
struct KwWhere : Key<StrWhere> {};
struct KwWhile : Key<StrWhile> {};
struct KwYield : Key<StrYield> {};

/**
 * @brief Some keywords are simply preserved and not used
 *
 */
struct Keyword
    : Key<pegtl::sor<StrAs,
          StrAsync,
          StrBool,
          StrBreak,
          StrChar,
          StrConst,
          StrContinue,
          StrCrate,
          StrElse,
          StrEnum,
          StrExtern,
          StrFalse,
          StrFor,
          StrFn,
          StrF32,
          StrF64,
          StrIf,
          StrIn,
          StrInterface,
          StrISize,
          StrI32,
          StrI64,
          StrLet,
          StrMatch,
          StrMod,
          StrMut,
          StrNew,
          StrNull,
          StrPriv,
          StrPub,
          StrReturn,
          StrLSelf,
          StrUSelf,
          StrStruct,
          StrString,
          StrSuper,
          StrTrue,
          StrUse,
          StrUSize,
          StrU32,
          StrU64,
          StrU8,
          StrWhere,
          StrWhile,
          StrYield>> {};


/**
 * @brief Matches an R that can be padded by arbitrary many S on the left and T on the right.
 * Equivalent to pegtl::seq<pegtl::star<S>, R, pegtl::star<T>>.
 * S=sep
 *
 * @tparam R
 */
template<typename R>
struct PadSep : pegtl::pad<R, Sep> {};
/**
 * @brief LstExpr = { B ~ (I ~ ",")* ~ I? ~ E }
 *
 * @tparam I
 * @tparam B
 * @tparam E
 */
template<typename I, typename B, typename E>
struct LstExpr : pegtl::seq<B, Seps, pegtl::star<I, Seps, pegtl::one<','>, Seps>, pegtl::opt<I, Seps>, E> {};
/**
 * @brief ParenLstExpr = { "(" ~ (I ~ ",")* ~ I? ~ ")" }
 *
 * @tparam I
 */
template<typename I>
struct ParenLstExpr : LstExpr<I, pegtl::one<'('>, pegtl::one<')'>> {};

struct single : pegtl::one<'a', 'b', 'f', 'n', 'r', 't', 'v', '\\', '"', '\'', '0', '\n'> {};
struct spaces : pegtl::seq<pegtl::one<'z'>, pegtl::star<pegtl::space>> {};
struct hexbyte : pegtl::if_must<pegtl::one<'x'>, pegtl::xdigit, pegtl::xdigit> {};
struct decbyte : pegtl::if_must<pegtl::digit, pegtl::rep_opt<2, pegtl::digit>> {};
struct unichar : pegtl::if_must<pegtl::one<'u'>, pegtl::one<'{'>, pegtl::plus<pegtl::xdigit>, pegtl::one<'}'>> {};
struct escaped : pegtl::if_must<pegtl::one<'\\'>, pegtl::sor<hexbyte, decbyte, unichar, single, spaces>> {};
struct regular : pegtl::not_one<'\r', '\n'> {};
struct character : pegtl::sor<escaped, regular> {};

struct StrLitral : pegtl::seq<pegtl::one<'"'>, pegtl::until<pegtl::one<'"'>, character>> {};
struct CharLiteral : pegtl::seq<pegtl::one<'\''>, character, pegtl::one<'\''>> {};
/**
 * @brief IntLiteral = { ASCII_DIGIT+ }
 *
 */
struct DecIntLiteral : pegtl::plus<pegtl::digit> {};

template<typename E>
struct exponent : pegtl::opt_must<E, pegtl::opt<pegtl::one<'+', '-'>>, pegtl::plus<pegtl::digit>> {};
/**
 * @brief FloatLiteral = @{ ASCII_DIGIT* ~ "." ~ ASCII_DIGIT+ }
 *
 */
struct FloatLiteral
    : pegtl::sor<pegtl::seq<pegtl::plus<pegtl::digit>,
                     pegtl::one<'.'>,
                     pegtl::star<pegtl::digit>,
                     exponent<pegtl::one<'e', 'E'>>>,
          pegtl::seq<pegtl::one<'.'>, pegtl::plus<pegtl::digit>, exponent<pegtl::one<'e', 'E'>>>> {};

/**
 * @brief Id = @{!KeyWord ~ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }
 *
 */
struct Id : pegtl::seq<pegtl::not_at<Keyword>, pegtl::identifier> {};
/**
 * @brief PathExpr = { (Id | KwCrate | KwSuper | KwUSelf) ~ ("::" ~ (Id | KwSuper))* }
 *
 */
struct Path
    : pegtl::seq<pegtl::sor<Id, KwCrate, KwSuper, KwUSelf>,
          Seps,
          pegtl::star_must<pegtl::two<':'>, Seps, pegtl::sor<Id, KwSuper>, Seps>> {};

struct Pattern;
/**
 * @brief TuplePattern = { "(" ~ (Pattern ~ ",")* ~ Pattern? ~ ")" }
 *
 */
struct TuplePattern : ParenLstExpr<Pattern> {};
/**
 * @brief Pattern = { Id | TuplePattern }
 *
 */
struct Pattern : pegtl::sor<Id, TuplePattern> {};

struct Type;
struct BasicType
    : pegtl::sor<KwBool, KwChar, KwF32, KwF64, KwISize, KwI32, KwI64, KwUSize, KwString, KwU8, KwU32, KwU64> {};
/**
 * @brief TupleType = { "(" ~ (Type ~ ",")* ~ Type? ~ ")" }
 *
 */
struct TupleType : ParenLstExpr<Type> {};
/**
 * @brief NonArrType = _{BasicType | KwUSelf | PathExpr | TupleType}
 *
 */
struct NonArrType : pegtl::sor<BasicType, KwUSelf, Path, TupleType> {};
/**
 * @brief Type = { NonArrType ~ (LBracket ~ RBracket)? }
 *
 */
struct Type : pegtl::seq<NonArrType, pegtl::opt<Seps, pegtl::one<'['>, Seps, pegtl::one<']'>>> {};

struct Stmt;
struct ExprWOBlock;
struct ExprWBlock;
struct Expr : pegtl::sor<ExprWOBlock, ExprWBlock> {};

/**
 * @brief
 *
 * @tparam O op
 * @tparam N chars that should not follow op
 */
template<char O, char... N>
struct OpOne : pegtl::seq<pegtl::one<O>, pegtl::at<pegtl::not_one<N...>>> {};
template<char O, char P, char... N>
struct OpTwo : pegtl::seq<pegtl::string<O, P>, pegtl::at<pegtl::not_one<N...>>> {};

/**
 * @brief { S ~ (O ~ R)* }
 *
 * @tparam S
 * @tparam O
 * @tparam R
 */
template<typename S, typename O, typename R = S>
struct LeftAssoc : pegtl::seq<S, Seps, pegtl::star_must<O, Seps, R, Seps>> {};


/**
 * @brief StructFieldInitExpr =  { Id ~ (":" ~ Expr)? }
 *
 */
struct StructFieldInitExpr : pegtl::seq<Id, Seps, pegtl::opt<pegtl::one<':'>, Seps, Expr>> {};
/**
 * @brief StructInitExpr = {"{" ~ (StructFieldInitExpr ~ ",")* ~ StructFieldInitExpr? ~ "}"}
 *
 */
struct StructInitExpr : LstExpr<StructFieldInitExpr, pegtl::one<'{'>, pegtl::one<'}'>> {};
/**
 * @brief Args = { "(" ~ (Expr ~ ",")* ~ Expr? ~ ")" }
 *
 */
struct Args : ParenLstExpr<Expr> {};
/**
 * @brief ObjAccExpr = { "." ~ Id }
 *
 */
struct ObjAccExpr : pegtl::seq<pegtl::one<'.'>, Seps, Id> {};
/**
 * @brief PathAccExpr = { "::" ~ Id }
 *
 */
struct PathAccExpr : pegtl::seq<pegtl::two<':'>, Seps, Id> {};
/**
 * @brief ArrAccExpr = { "[" ~ Expr ~ "]" }
 *
 */
struct ArrAccExpr : pegtl::seq<pegtl::one<'['>, Seps, Expr, Seps, pegtl::one<']'>> {};
/**
 * @brief literals.
 * lambda expression is also literal
 *
 */
struct LiteralExpr : pegtl::sor<KwNull, KwTrue, KwFalse, DecIntLiteral, FloatLiteral, CharLiteral, StrLitral> {};
/**
 * @brief NewExpr = { "new" ~ Type ~ (StructInitExpr | ArrAccExpr) }
 *
 */
struct NewExpr : pegtl::if_must<KwNew, Seps, Type, Seps, pegtl::sor<StructInitExpr, ArrAccExpr>> {};
/**
 * @brief PrimaryExpr = { LiteralExpr | KwLSelf | "(" ~ Expr ~ ")" | ExprWBlock | Id | Type | NewExpr }
 *
 */
struct PrimaryExpr
    : pegtl::sor<LiteralExpr,
          KwLSelf,
          pegtl::seq<pegtl::one<'('>, Seps, Expr, Seps, pegtl::one<')'>>,
          ExprWBlock,
          Id,
          Type,
          NewExpr> {};
/**
 * @brief CallExpr = { PrimaryExpr ~ (Args | ObjAccExpr | PathAccExpr | ArrAccExpr)* }
 *
 */
struct CallExpr
    : pegtl::seq<PrimaryExpr, Seps, pegtl::star<pegtl::sor<Args, ObjAccExpr, PathAccExpr, ArrAccExpr>, Seps>> {};
/**
 * @brief UnaryExpr = { (Not | Plus | Minus)* ~ CallExpr }
 *
 */
struct UnaryExpr
    : pegtl::seq<pegtl::star<pegtl::sor<pegtl::one<'-'>, pegtl::one<'+'>, pegtl::one<'!'>>, Seps>, Seps, CallExpr> {};
struct CastExpr : LeftAssoc<UnaryExpr, KwAs, Type> {};
struct MulExpr : LeftAssoc<CastExpr, pegtl::sor<pegtl::one<'/'>, pegtl::one<'*'>, pegtl::one<'%'>>> {};
struct AddExpr : LeftAssoc<MulExpr, pegtl::sor<pegtl::one<'+'>, pegtl::one<'-'>>> {};
struct CompExpr
    : LeftAssoc<AddExpr,
          pegtl::sor<pegtl::string<'<', '='>, pegtl::string<'>', '='>, pegtl::one<'<'>, pegtl::one<'>'>>> {};
struct EqExpr : LeftAssoc<CompExpr, pegtl::sor<pegtl::two<'='>, pegtl::string<'!', '='>>> {};
struct LogAndExpr : LeftAssoc<EqExpr, pegtl::two<'&'>> {};
struct LogOrExpr : LeftAssoc<LogAndExpr, pegtl::two<'|'>> {};


/**
 * @brief LetStmt = { "let" ~ Pattern ~ (":" ~ Type)? ~ (Eq ~ Expr)? ~ Semi }
 *
 */
struct LetStmt
    : pegtl::if_must<KwLet,
          Seps,
          Pattern,
          Seps,
          pegtl::opt<pegtl::one<':'>, Seps, Type, Seps>,
          pegtl::opt<pegtl::one<'='>, Seps, Expr, Seps>,
          pegtl::one<';'>> {};
/**
 * @brief Stmt = { LetStmt | ExprWithoutBlock ~ Semi | ExprWithBlock ~ Semi? }
 *
 */
struct Stmt
    : pegtl::sor<LetStmt,
          pegtl::seq<ExprWOBlock, Seps, pegtl::one<';'>>,
          pegtl::seq<ExprWBlock, Seps, pegtl::opt<pegtl::one<';'>>>> {};


/**
 * @brief BreakExpr = { "break" ~ Expr? }
 *
 */
struct BreakExpr : pegtl::if_must<KwBreak, Seps, pegtl::opt<Expr>> {};
/**
 * @brief ReturnExpr = { "return" ~ Expr? }
 *
 */
struct RetExpr : pegtl::if_must<KwReturn, Seps, pegtl::opt<Expr>> {};
/**
 * @brief AssignExpr = { LogOrExpr ~ Eq ~ LogOrExpr }
 *
 */
struct AssignExpr : pegtl::seq<LogOrExpr, Seps, pegtl::one<'='>, Seps, LogOrExpr> {};
struct ExprWOBlock : pegtl::sor<KwContinue, BreakExpr, RetExpr, AssignExpr, LogOrExpr> {};

/**
 * @brief BlockExpr = { "{" ~ Stmt* ~ ExprWithoutBlock? ~ "}" }
 *
 */
struct BlockExpr
    : pegtl::seq<pegtl::one<'{'>, Seps, pegtl::star<Stmt, Seps>, pegtl::opt<ExprWOBlock, Seps>, pegtl::one<'}'>> {};
/**
 * @brief IfExpr = { "if" ~ Expr ~ BlockExpr ~ ("else" ~ (BlockExpr | IfExpr))? }
 *
 */
struct IfExpr
    : pegtl::if_must<KwIf, Seps, Expr, Seps, BlockExpr, Seps, pegtl::opt<KwElse, Seps, pegtl::sor<BlockExpr, IfExpr>>> {
};
/**
 * @brief WhileExpr = { "while" ~ Expr ~ BlockExpr }
 *
 */
struct WhileExpr : pegtl::if_must<KwWhile, Seps, Expr, Seps, BlockExpr> {};
/**
 * @brief ExprWithBlock = { BlockExpr | LoopExpr | IfExpr }
 *
 */
struct ExprWBlock : pegtl::sor<BlockExpr, IfExpr, WhileExpr> {};


/**
 * @brief Attribute = { Id ~ ("(" ~ (LiteralExpr ~ ",")* ~ LiteralExpr? ~ ")")? }
 *
 */
struct Attrib : pegtl::seq<Id, Seps, pegtl::opt<ParenLstExpr<Expr>>> {};
/**
 * @brief AttributeLst = { "#" ~ "[" ~ Attribute ~ ("," ~ Attribute)* ~ "]" }
 *
 */
struct AttribLst : pegtl::seq<pegtl::one<'#'>, Seps, LstExpr<Attrib, pegtl::one<'['>, pegtl::one<']'>>> {};


struct StaticFnParams : ParenLstExpr<pegtl::seq<Id, Seps, pegtl::one<':'>, Seps, Type>> {};
/**
 * @brief Params = { "(" ~ KwLSelf ~ ("," ~ Id ~ ":" ~ Type)* ~ ","? ~ ")" }
 *
 */
struct MethodParams
    : pegtl::seq<pegtl::one<'('>,
          Seps,
          KwLSelf,
          Seps,
          pegtl::star<pegtl::one<','>, Seps, Id, Seps, pegtl::one<':'>, Seps, Type>,
          pegtl::opt<pegtl::one<','>, Seps>,
          pegtl::one<')'>> {};
/**
 * @brief { AttributeLst* ~ "fn" ~ Id ~ PS ~ ("->" ~ Type)? ~ (BlockExpr | Semi) }
 *
 * @tparam PS
 */
template<typename PS = StaticFnParams>
struct Fn
    : pegtl::seq<pegtl::star<AttribLst, Seps>,
          KwFn,
          Seps,
          Id,
          PS,
          Seps,
          pegtl::opt<pegtl::string<'-', '>'>, Seps, Type, Seps>,
          pegtl::sor<BlockExpr, pegtl::one<';'>>> {};
struct Method : Fn<MethodParams> {};

/**
 * @brief Field = { "let" ~ Id ~ ":" ~ Type ~ Semi }
 *
 */
struct Field : pegtl::seq<KwLet, Seps, Id, Seps, pegtl::one<':'>, Seps, Type, Seps, pegtl::one<';'>> {};
/**
 * @brief Global = { "const" ~ Id ~ ":" ~ Type ~ "=" ~ Expr ~ Semi }
 *
 */
struct Global
    : pegtl::seq<KwConst,
          Seps,
          Id,
          Seps,
          pegtl::one<':'>,
          Seps,
          Type,
          Seps,
          pegtl::one<'='>,
          Seps,
          Expr,
          Seps,
          pegtl::one<';'>> {};

/**
 * @brief Impls = { ":" ~ PathExpr ~ ("," ~ PathExpr)* }
 *
 */
struct Impls : pegtl::seq<pegtl::one<':'>, Seps, Path, Seps, pegtl::star<pegtl::one<','>, Seps, Path>> {};
/**
 * @brief Class = {
 *  AttributeLst* ~ (KwStruct | KwInterface) ~ Id ~ Impls? ~
 *  "{" ~ (Fn | Method | Field)* ~ "}"
 * }
 *
 */
struct StructOrInterface
    : pegtl::seq<pegtl::star<AttribLst, Seps>,
          pegtl::if_must<pegtl::sor<KwStruct, KwInterface>,
              Seps,
              Id,
              Seps,
              pegtl::opt<Impls, Seps>,
              pegtl::one<'{'>,
              Seps,
              pegtl::star<pegtl::sor<Fn<>, Method, Field>, Seps>,
              pegtl::one<'}'>>> {};

/**
 * @brief ExternModuleDeclare = { "extern" ~ Id ~ Semi }
 *
 */
struct ExtModDeclare : pegtl::seq<KwExtern, Seps, Id, Seps, pegtl::one<';'>> {};
/**
 * @brief ModuleDeclare = { "mod" ~ Id ~ Semi }
 *
 */
struct ModDeclare : pegtl::seq<KwMod, Seps, Id, Seps, pegtl::one<';'>> {};
/**
 * @brief UseStmt = { "use" ~ PathExpr ~ ("as" ~ Id)? ~ Semi }
 *
 */
struct UseStmt : pegtl::seq<KwUse, Seps, Path, Seps, pegtl::opt<KwAs, Seps, Id, Seps>, Seps, pegtl::one<';'>> {};

/**
 * @brief File = { SOI ~ (ExternModuleDeclare | ModuleDeclare | UseStmt)* ~ (Class | Func)* ~ EOI }
 *
 */
struct Grammar
    : pegtl::must<Seps,
          pegtl::star<pegtl::sor<ExtModDeclare, ModDeclare, UseStmt>, Seps>,
          pegtl::star<pegtl::sor<Fn<>, StructOrInterface, Global>, Seps>,
          pegtl::eof> {};
// clang-format off
// clang-format on

}// namespace lang::grammar

#endif