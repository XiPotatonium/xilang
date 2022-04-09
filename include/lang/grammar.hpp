#ifndef XILANG_LANG_GRAMMAR_HPP
#define XILANG_LANG_GRAMMAR_HPP


#include <tao/pegtl.hpp>
#include <tao/pegtl/contrib/raw_string.hpp>
#include <tao/pegtl/internal/pegtl_string.hpp>

namespace lang::grammar {

// clang-format off
/**
 * @brief C-style multi-line comment
 *
 */
struct CComment : tao::pegtl::if_must<tao::pegtl::string<'/', '*'>, tao::pegtl::until<tao::pegtl::string<'*', '/'>>> {};
/**
 * @brief A C++ style one-line comment. two consecutive slashes followed by anything up to the end of line or end of
 * file.
 *
 */
struct CppComment : tao::pegtl::if_must<tao::pegtl::two<'/'>, tao::pegtl::until<tao::pegtl::eolf>> {};
/**
 * @brief C-style comment and Cpp-style comment
 *
 */
struct Comment : tao::pegtl::sor<CppComment, CComment> {};

struct Sep : tao::pegtl::sor<tao::pegtl::ascii::space, Comment> {};
struct Seps : tao::pegtl::star<Sep> {};

struct StrAs : TAO_PEGTL_STRING("as") {};
struct StrAsync : TAO_PEGTL_STRING("async") {};
struct StrBool : TAO_PEGTL_STRING("bool") {};
struct StrBreak : TAO_PEGTL_STRING("break") {};
struct StrChar : TAO_PEGTL_STRING("char") {};
struct StrConst : TAO_PEGTL_STRING("const") {};
struct StrContinue : TAO_PEGTL_STRING("continue") {};
// struct StrCrate : TAO_PEGTL_STRING("crate") {};
struct StrElse : TAO_PEGTL_STRING("else") {};
struct StrEnum : TAO_PEGTL_STRING("enum") {};
// struct StrExtern : TAO_PEGTL_STRING("extern") {};
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
// struct StrMod : TAO_PEGTL_STRING("mod") {};
struct StrMut : TAO_PEGTL_STRING("mut") {};
struct StrNew : TAO_PEGTL_STRING("new") {};
// struct StrNull : TAO_PEGTL_STRING("null") {};
struct StrPriv : TAO_PEGTL_STRING("priv") {};
struct StrPub : TAO_PEGTL_STRING("pub") {};
struct StrReturn : TAO_PEGTL_STRING("return") {};
struct StrLSelf : TAO_PEGTL_STRING("self") {};
struct StrUSelf : TAO_PEGTL_STRING("Self") {};
struct StrStruct : TAO_PEGTL_STRING("struct") {};
struct StrStr : TAO_PEGTL_STRING("str") {};
// struct StrSuper : TAO_PEGTL_STRING("super") {};
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
struct Key : tao::pegtl::seq<KW, tao::pegtl::not_at<tao::pegtl::identifier_other>> {};

struct KwAs : Key<StrAs> {};
struct KwAsync : Key<StrAsync> {};
struct KwBool : Key<StrBool> {};
struct KwBreak : Key<StrBreak> {};
struct KwChar : Key<StrChar> {};
struct KwConst : Key<StrConst> {};
struct KwContinue : Key<StrContinue> {};
// struct KwCrate : Key<StrCrate> {};
struct KwElse : Key<StrElse> {};
struct KwEnum : Key<StrEnum> {};
// struct KwExtern : Key<StrExtern> {};
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
// struct KwMod : Key<StrMod> {};
struct KwMut : Key<StrMut> {};
struct KwNew : Key<StrNew> {};
// struct KwNull : Key<StrNull> {};
struct KwPriv : Key<StrPriv> {};
struct KwPub : Key<StrPub> {};
struct KwReturn : Key<StrReturn> {};
struct KwLSelf : Key<StrLSelf> {};
struct KwUSelf : Key<StrUSelf> {};
struct KwStruct : Key<StrStruct> {};
struct KwStr : Key<StrStr> {};
// struct KwSuper : Key<StrSuper> {};
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
    : Key<tao::pegtl::sor<StrAs,
          StrAsync,
          StrBool,
          StrBreak,
          StrChar,
          StrConst,
          StrContinue,
          // StrCrate,
          StrElse,
          StrEnum,
          // StrExtern,
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
          // StrMod,
          StrMut,
          StrNew,
          // StrNull,
          StrPriv,
          StrPub,
          StrReturn,
          StrLSelf,
          StrUSelf,
          StrStruct,
          StrStr,
          // StrSuper,
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
 * @brief Used in Path
 *
 */
struct Dot: tao::pegtl::one<'.'> {};

/**
 * @brief Matches an R that can be padded by arbitrary many S on the left and T on the right.
 * Equivalent to tao::pegtl::seq<tao::pegtl::star<S>, R, tao::pegtl::star<T>>.
 * S=sep
 *
 * @tparam R
 */
template<typename R>
struct PadSep : tao::pegtl::pad<R, Sep> {};


/**
 * @brief LstExpr = { B ~ (I ~ ",")* ~ I? ~ E }
 *
 * @tparam I
 * @tparam B
 * @tparam E
 */
template<typename I, typename B, typename E>
struct LstExpr
    : tao::pegtl::seq<B, Seps, tao::pegtl::star<I, Seps, tao::pegtl::one<','>, Seps>, tao::pegtl::opt<I, Seps>, E> {};
/**
 * @brief ParenLstExpr = { "(" ~ (I ~ ",")* ~ I? ~ ")" }
 *
 * @tparam I
 */
template<typename I>
struct ParenLstExpr : LstExpr<I, tao::pegtl::one<'('>, tao::pegtl::one<')'>> {};

struct single : tao::pegtl::one<'a', 'b', 'f', 'n', 'r', 't', 'v', '\\', '"', '\'', '0', '\n'> {};
struct spaces : tao::pegtl::seq<tao::pegtl::one<'z'>, tao::pegtl::star<tao::pegtl::space>> {};
struct hexbyte : tao::pegtl::if_must<tao::pegtl::one<'x'>, tao::pegtl::xdigit, tao::pegtl::xdigit> {};
struct decbyte : tao::pegtl::if_must<tao::pegtl::digit, tao::pegtl::rep_opt<2, tao::pegtl::digit>> {};
struct unichar
    : tao::pegtl::if_must<tao::pegtl::one<'u'>,
          tao::pegtl::one<'{'>,
          tao::pegtl::plus<tao::pegtl::xdigit>,
          tao::pegtl::one<'}'>> {};
struct escaped
    : tao::pegtl::if_must<tao::pegtl::one<'\\'>, tao::pegtl::sor<hexbyte, decbyte, unichar, single, spaces>> {};
struct regular : tao::pegtl::not_one<'\r', '\n'> {};
struct character : tao::pegtl::sor<escaped, regular> {};

struct StrLitral : tao::pegtl::seq<tao::pegtl::one<'"'>, tao::pegtl::until<tao::pegtl::one<'"'>, character>> {};
struct CharLiteral : tao::pegtl::seq<tao::pegtl::one<'\''>, character, tao::pegtl::one<'\''>> {};
/**
 * @brief IntLiteral = { ASCII_DIGIT+ }
 *
 */
struct DecIntLiteral : tao::pegtl::plus<tao::pegtl::digit> {};

template<typename E>
struct exponent
    : tao::pegtl::opt_must<E, tao::pegtl::opt<tao::pegtl::one<'+', '-'>>, tao::pegtl::plus<tao::pegtl::digit>> {};
/**
 * @brief FloatLiteral = @{ ASCII_DIGIT* ~ "." ~ ASCII_DIGIT+ }
 *
 */
struct FloatLiteral: tao::pegtl::sor<
    tao::pegtl::seq<
        tao::pegtl::plus<tao::pegtl::digit>,
        tao::pegtl::one<'.'>,
        tao::pegtl::star<tao::pegtl::digit>,
        exponent<tao::pegtl::one<'e', 'E'>>
    >,
    tao::pegtl::seq<
        tao::pegtl::one<'.'>,
        tao::pegtl::plus<tao::pegtl::digit>,
        exponent<tao::pegtl::one<'e', 'E'>>
    >
> {};

struct Type;


/**
 * @brief Id = @{!KeyWord ~ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }
 *
 */
struct Id : tao::pegtl::seq<tao::pegtl::not_at<Keyword>, tao::pegtl::identifier> {};
/**
 * @brief IdG = {Id ~ ("<" ~ (Type ~ ",")* ~ Type? ~ ">")?}
 *
 */
struct IdG : tao::pegtl::seq<Id, tao::pegtl::opt<Seps, LstExpr<Type, tao::pegtl::one<'<'>, tao::pegtl::one<'>'>>>> {};
/**
 * @brief Generic = {"<" ~ (Id ~ ",")* ~ Id? ">"}
 *
 */
struct Generic : LstExpr<Id, tao::pegtl::one<'<'>, tao::pegtl::one<'>'>> {};
/**
 * @brief Path = { Dot* ~ IdG ~ ("::" ~ IdG)* }
 *
 */
struct Path
    : tao::pegtl::seq<tao::pegtl::star<Dot>,
          Seps,
          IdG,
          Seps,
          tao::pegtl::star_must<tao::pegtl::two<':'>, Seps, IdG, Seps>> {};


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
struct Pattern : tao::pegtl::sor<Id, TuplePattern> {};

struct BasicType
    : tao::pegtl::sor<KwBool, KwChar, KwF32, KwF64, KwISize, KwI32, KwI64, KwUSize, KwStr, KwU8, KwU32, KwU64> {};
/**
 * @brief TupleType = { "(" ~ (Type ~ ",")* ~ Type? ~ ")" }
 *
 */
struct TupleType : ParenLstExpr<Type> {};
/**
 * @brief NonArrType = _{BasicType | KwUSelf | Path | TupleType}
 *
 */
struct NonArrType : tao::pegtl::sor<BasicType, KwUSelf, Path, TupleType> {};
/**
 * @brief Type = { NonArrType ~ (LBracket ~ RBracket)? }
 *
 */
struct Type : tao::pegtl::seq<NonArrType, tao::pegtl::opt<Seps, tao::pegtl::one<'['>, Seps, tao::pegtl::one<']'>>> {};

struct Stmt;
struct ExprWOBlock;
struct ExprWBlock;
struct Expr : tao::pegtl::sor<ExprWOBlock, ExprWBlock> {};

/**
 * @brief
 *
 * @tparam O op
 * @tparam N chars that should not follow op
 */
template<char O, char... N>
struct OpOne : tao::pegtl::seq<tao::pegtl::one<O>, tao::pegtl::at<tao::pegtl::not_one<N...>>> {};
template<char O, char P, char... N>
struct OpTwo : tao::pegtl::seq<tao::pegtl::string<O, P>, tao::pegtl::at<tao::pegtl::not_one<N...>>> {};

/**
 * @brief { S ~ (O ~ R)* }
 *
 * @tparam S
 * @tparam O
 * @tparam R
 */
template<typename S, typename O, typename R = S>
struct LeftAssoc : tao::pegtl::seq<S, Seps, tao::pegtl::star_must<O, Seps, R, Seps>> {};


/**
 * @brief StructFieldInitExpr =  { Id ~ (":" ~ Expr)? }
 *
 */
struct StructFieldInitExpr : tao::pegtl::seq<Id, Seps, tao::pegtl::opt<tao::pegtl::one<':'>, Seps, Expr>> {};
/**
 * @brief StructInitExpr = {"{" ~ (StructFieldInitExpr ~ ",")* ~ StructFieldInitExpr? ~ "}"}
 *
 */
struct StructInitExpr : LstExpr<StructFieldInitExpr, tao::pegtl::one<'{'>, tao::pegtl::one<'}'>> {};
/**
 * @brief Args = { "(" ~ (Expr ~ ",")* ~ Expr? ~ ")" }
 *
 */
struct Args : ParenLstExpr<Expr> {};
/**
 * @brief ObjAccExpr = { "." ~ IdG }
 *
 */
struct ObjAccExpr : tao::pegtl::seq<tao::pegtl::one<'.'>, Seps, IdG> {};
/**
 * @brief StaticAccExpr = { "::" ~ IdG }
 *
 */
struct StaticAccExpr : tao::pegtl::seq<tao::pegtl::two<':'>, Seps, IdG> {};
/**
 * @brief ArrAccExpr = { "[" ~ Expr ~ "]" }
 *
 */
struct ArrAccExpr : tao::pegtl::seq<tao::pegtl::one<'['>, Seps, Expr, Seps, tao::pegtl::one<']'>> {};
/**
 * @brief literals.
 * lambda expression is also literal
 *
 */
struct LiteralExpr : tao::pegtl::sor<KwTrue, KwFalse, FloatLiteral, DecIntLiteral, CharLiteral, StrLitral> {};
/**
 * @brief NewExpr = { "new" ~ Type ~ (StructInitExpr | ArrAccExpr) }
 *
 */
struct NewExpr : tao::pegtl::if_must<KwNew, Seps, Type, Seps, tao::pegtl::sor<StructInitExpr, ArrAccExpr>> {};
/**
 * @brief PrimaryExpr = { LiteralExpr | KwLSelf | "(" ~ Expr ~ ")" | ExprWBlock | Type | NewExpr }
 *
 */
struct PrimaryExpr
    : tao::pegtl::sor<LiteralExpr,
          KwLSelf,
          tao::pegtl::seq<tao::pegtl::one<'('>, Seps, Expr, Seps, tao::pegtl::one<')'>>,
          ExprWBlock,
          Type,
          NewExpr> {};
/**
 * @brief CallExpr = { PrimaryExpr ~ (Args | ObjAccExpr | StaticAccExpr | ArrAccExpr)* }
 *
 */
struct CallExpr
    : tao::pegtl::
          seq<PrimaryExpr, Seps, tao::pegtl::star<tao::pegtl::sor<Args, ObjAccExpr, StaticAccExpr, ArrAccExpr>, Seps>> {
};
/**
 * @brief UnaryExpr = { (Not | Plus | Minus)* ~ CallExpr }
 *
 */
struct UnaryExpr
    : tao::pegtl::seq<
          tao::pegtl::star<tao::pegtl::sor<tao::pegtl::one<'-'>, tao::pegtl::one<'+'>, tao::pegtl::one<'!'>>, Seps>,
          Seps,
          CallExpr> {};
/**
 * @brief CastExpr = {UnaryExpr ~ ("as" ~ Type)*}
 *
 */
struct CastExpr : LeftAssoc<UnaryExpr, KwAs, Type> {};
/**
 * @brief MulExpr = {CastExpr ~ (("*" | "/" | "%") ~ CastExpr)*}
 *
 */
struct MulExpr
    : LeftAssoc<CastExpr, tao::pegtl::sor<tao::pegtl::one<'/'>, tao::pegtl::one<'*'>, tao::pegtl::one<'%'>>> {};
/**
 * @brief AddExpr = {MulExpr ~ (("+" | "-") ~ MulExpr)*}
 *
 */
struct AddExpr : LeftAssoc<MulExpr, tao::pegtl::sor<tao::pegtl::one<'+'>, tao::pegtl::one<'-'>>> {};
struct CompExpr
    : LeftAssoc<AddExpr,
          tao::pegtl::sor<tao::pegtl::string<'<', '='>,
              tao::pegtl::string<'>', '='>,
              tao::pegtl::one<'<'>,
              tao::pegtl::one<'>'>>> {};
struct EqExpr : LeftAssoc<CompExpr, tao::pegtl::sor<tao::pegtl::two<'='>, tao::pegtl::string<'!', '='>>> {};
struct LogAndExpr : LeftAssoc<EqExpr, tao::pegtl::two<'&'>> {};
struct LogOrExpr : LeftAssoc<LogAndExpr, tao::pegtl::two<'|'>> {};


/**
 * @brief LetStmt = { "let" ~ Pattern ~ (":" ~ Type)? ~ (Eq ~ Expr)? ~ Semi }
 *
 */
struct LetStmt
    : tao::pegtl::if_must<KwLet,
          Seps,
          Pattern,
          Seps,
          tao::pegtl::opt<tao::pegtl::one<':'>, Seps, Type, Seps>,
          tao::pegtl::opt<tao::pegtl::one<'='>, Seps, Expr, Seps>,
          tao::pegtl::one<';'>> {};
/**
 * @brief Stmt = { LetStmt | ExprWithoutBlock ~ Semi | ExprWithBlock ~ Semi? }
 *
 */
struct Stmt
    : tao::pegtl::sor<LetStmt,
          tao::pegtl::seq<ExprWOBlock, Seps, tao::pegtl::one<';'>>,
          tao::pegtl::seq<ExprWBlock, Seps, tao::pegtl::opt<tao::pegtl::one<';'>>>> {};


/**
 * @brief BreakExpr = { "break" ~ Expr? }
 *
 */
struct BreakExpr : tao::pegtl::if_must<KwBreak, Seps, tao::pegtl::opt<Expr>> {};
/**
 * @brief ReturnExpr = { "return" ~ Expr? }
 *
 */
struct RetExpr : tao::pegtl::if_must<KwReturn, Seps, tao::pegtl::opt<Expr>> {};
/**
 * @brief AssignExpr = { LogOrExpr ~ Eq ~ LogOrExpr }
 *
 */
struct AssignExpr : tao::pegtl::seq<LogOrExpr, Seps, tao::pegtl::one<'='>, Seps, LogOrExpr> {};
struct ExprWOBlock : tao::pegtl::sor<KwContinue, BreakExpr, RetExpr, AssignExpr, LogOrExpr> {};

/**
 * @brief BlockExpr = { "{" ~ Stmt* ~ ExprWithoutBlock? ~ "}" }
 *
 */
struct BlockExpr
    : tao::pegtl::seq<tao::pegtl::one<'{'>,
          Seps,
          tao::pegtl::star<Stmt, Seps>,
          tao::pegtl::opt<ExprWOBlock, Seps>,
          tao::pegtl::one<'}'>> {};
/**
 * @brief IfExpr = { "if" ~ Expr ~ BlockExpr ~ ("else" ~ (BlockExpr | IfExpr))? }
 *
 */
struct IfExpr
    : tao::pegtl::if_must<KwIf,
          Seps,
          Expr,
          Seps,
          BlockExpr,
          Seps,
          tao::pegtl::opt<KwElse, Seps, tao::pegtl::sor<BlockExpr, IfExpr>>> {};
/**
 * @brief WhileExpr = { "while" ~ Expr ~ BlockExpr }
 *
 */
struct WhileExpr : tao::pegtl::if_must<KwWhile, Seps, Expr, Seps, BlockExpr> {};
/**
 * @brief ExprWithBlock = { BlockExpr | LoopExpr | IfExpr }
 *
 */
struct ExprWBlock : tao::pegtl::sor<BlockExpr, IfExpr, WhileExpr> {};


/**
 * @brief Attrib = { Id ~ Args? }
 *
 */
struct Attrib : tao::pegtl::seq<Id, Seps, tao::pegtl::opt<Args>> {};
/**
 * @brief AttribLst = { "#" ~ "[" ~ (Attrib ~ ",")* ~ Attrib? ~ "]" }
 *
 */
struct AttribLst
    : tao::pegtl::seq<tao::pegtl::one<'#'>, Seps, LstExpr<Attrib, tao::pegtl::one<'['>, tao::pegtl::one<']'>>> {};


struct StaticFnParams : ParenLstExpr<tao::pegtl::seq<Id, Seps, tao::pegtl::one<':'>, Seps, Type>> {};
/**
 * @brief Params = { "(" ~ KwLSelf ~ ("," ~ Id ~ ":" ~ Type)* ~ ","? ~ ")" }
 *
 */
struct MethodParams
    : tao::pegtl::seq<tao::pegtl::one<'('>,
          Seps,
          KwLSelf,
          Seps,
          tao::pegtl::star<tao::pegtl::one<','>, Seps, Id, Seps, tao::pegtl::one<':'>, Seps, Type>,
          tao::pegtl::opt<tao::pegtl::one<','>, Seps>,
          tao::pegtl::one<')'>> {};
/**
 * @brief { AttribLst* ~ "fn" ~ Id ~ Generic? ~ PS ~ ("->" ~ Type)? ~ (BlockExpr | Semi) }
 *
 * @tparam PS
 */
template<typename PS = StaticFnParams>
struct Fn
    : tao::pegtl::seq<tao::pegtl::star<AttribLst, Seps>,
          KwFn,
          Seps,
          Id,
          Seps,
          tao::pegtl::opt<Generic, Seps>,
          PS,
          Seps,
          tao::pegtl::opt<tao::pegtl::string<'-', '>'>, Seps, Type, Seps>,
          tao::pegtl::sor<BlockExpr, tao::pegtl::one<';'>>> {};
struct Method : Fn<MethodParams> {};

/**
 * @brief Field = { Id ~ ":" ~ Type ~ "," }
 *
 */
struct Field : tao::pegtl::seq<Id, Seps, tao::pegtl::one<':'>, Seps, Type, Seps, tao::pegtl::one<','>> {};
/**
 * @brief Global = { "const" ~ Id ~ ":" ~ Type ~ "=" ~ Expr ~ Semi }
 *
 */
struct Global
    : tao::pegtl::if_must<KwConst,
          Seps,
          Id,
          Seps,
          tao::pegtl::one<':'>,
          Seps,
          Type,
          Seps,
          tao::pegtl::one<'='>,
          Seps,
          Expr,
          Seps,
          tao::pegtl::one<';'>> {};

/**
 * @brief Impls = { ":" ~ Path ~ ("," ~ Path)* ~ ","? }
 *
 */
struct Impls
    : tao::pegtl::seq<tao::pegtl::one<':'>, Seps, Path, Seps, tao::pegtl::star<tao::pegtl::one<','>, Seps, Path>> {};
/**
 * @brief Struct = {
 *  AttribLst* ~ KwStruct ~ Id ~ Generic? ~ Impls? ~
 *  "{" ~ (Fn | Method | Field)* ~ "}"
 * }
 *
 */
struct Struct
    : tao::pegtl::seq<tao::pegtl::star<AttribLst, Seps>,
          tao::pegtl::if_must<KwStruct,
              Seps,
              Id,
              Seps,
              tao::pegtl::opt<Generic, Seps>,
              Seps,
              tao::pegtl::opt<Impls, Seps>,
              tao::pegtl::one<'{'>,
              Seps,
              tao::pegtl::star<tao::pegtl::sor<Fn<>, Method, Field>, Seps>,
              tao::pegtl::one<'}'>>> {};
/**
 * @brief Interface = {
 *  AttribLst* ~ KwInterface ~ Id ~ Generic? ~ Impls? ~
 *  "{" ~ Method* ~ "}"
 * }
 *
 */
struct Interface
    : tao::pegtl::seq<tao::pegtl::star<AttribLst, Seps>,
          tao::pegtl::if_must<KwInterface,
              Seps,
              Id,
              Seps,
              tao::pegtl::opt<Generic, Seps>,
              Seps,
              tao::pegtl::opt<Impls, Seps>,
              tao::pegtl::one<'{'>,
              Seps,
              tao::pegtl::star<Method, Seps>,
              tao::pegtl::one<'}'>>> {};

/**
 * @brief EnumField = {Id ~ "(" ~ Type* ~ ")" ~ ","}
 *
 */
struct EnumField
    : tao::pegtl::seq<Id,
          Seps,
          tao::pegtl::one<'('>,
          Seps,
          tao::pegtl::opt<Type, Seps>,
          tao::pegtl::one<')'>,
          Seps,
          tao::pegtl::one<','>> {};
/**
 * @brief Enum = {
 *  AttribLst* ~ KwEnum ~ Id ~ Generic? ~
 *  "{" ~ (Fn | Method | EnumField)* ~ "}"
 * }
 *
 */
struct Enum
    : tao::pegtl::seq<tao::pegtl::star<AttribLst, Seps>,
          tao::pegtl::if_must<KwEnum,
              Seps,
              Id,
              Seps,
              tao::pegtl::opt<Generic, Seps>,
              Seps,
              tao::pegtl::one<'{'>,
              Seps,
              tao::pegtl::star<tao::pegtl::sor<Fn<>, Method, EnumField>, Seps>,
              tao::pegtl::one<'}'>>> {};

/**
 * @brief UseStmt = { "use" ~ Path ~ ("as" ~ Id)? ~ Semi }
 *
 */
struct UseStmt
    : tao::pegtl::seq<KwUse, Seps, Path, Seps, tao::pegtl::opt<KwAs, Seps, Id, Seps>, Seps, tao::pegtl::one<';'>> {};

/**
 * @brief File = { SOI ~ UseStmt* ~ (Struct | Interface | Fn | Global | Enum)* ~ EOI }
 *
 */
struct Grammar
    : tao::pegtl::must<Seps,
          tao::pegtl::star<UseStmt, Seps>,
          tao::pegtl::star<tao::pegtl::sor<Fn<>, Struct, Interface, Global, Enum>, Seps>,
          tao::pegtl::eof> {};
// clang-format on

}// namespace lang::grammar

#endif
