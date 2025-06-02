#[cfg(feature = "no-std")]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use core::str::FromStr;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while_m_n};
use nom::character::complete::{
    alpha1, alphanumeric1, char, digit1, i32, line_ending, multispace0, multispace1, space0, space1,
};
use nom::combinator::{eof, map, map_res, not, opt, peek, recognize, value, verify};
use nom::error::{Error, ErrorKind};
use nom::multi::{many0, many1, separated_list0, separated_list1};
use nom::sequence::{delimited, preceded, terminated};
use nom::{Err, IResult, Parser};
use num::Complex;
use tiny_skia::BlendMode;

const KEYWORDS: &[&str] = &["let", "if", "else", "match", "for", "loop"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Triangle,
    Square,
    Circle,
    Fill,
    Empty,
}

impl ToString for ShapeKind {
    fn to_string(&self) -> String {
        match self {
            ShapeKind::Triangle => "TRIANGLE".into(),
            ShapeKind::Square => "SQUARE".into(),
            ShapeKind::Circle => "CIRCLE".into(),
            ShapeKind::Fill => "FILL".into(),
            ShapeKind::Empty => "EMPTY".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i32),
    Float(f32),
    Complex(Complex<f32>),
    Boolean(bool),
    Hex([u8; 3]),
    Shape(ShapeKind),
    BlendMode(BlendMode),
}

impl ToString for Literal {
    fn to_string(&self) -> String {
        match self {
            Literal::Integer(n) => n.to_string(),
            Literal::Float(n) => n.to_string(),
            Literal::Complex(n) => n.to_string(),
            Literal::Boolean(b) => b.to_string(),
            Literal::Hex([r, g, b]) => format!("0x{}{}{}", r, g, b),
            Literal::Shape(kind) => kind.to_string(),
            Literal::BlendMode(b) => match b {
                BlendMode::Clear => "BLEND_CLEAR".into(),
                BlendMode::SourceOver => "BLEND_SOURCE_OVER".into(),
                BlendMode::DestinationOver => "BLEND_DESTINATION_OVER".into(),
                BlendMode::SourceIn => "BLEND_SOURCE_IN".into(),
                BlendMode::DestinationIn => "BLEND_DESTINATION_IN".into(),
                BlendMode::SourceOut => "BLEND_SOURCE_OUT".into(),
                BlendMode::DestinationOut => "BLEND_DESTINATION_OUT".into(),
                BlendMode::SourceAtop => "BLEND_SOURCE_ATOP".into(),
                BlendMode::DestinationAtop => "BLEND_DESTINATION_ATOP".into(),
                BlendMode::Source => "BLEND_SOURCE".into(),
                BlendMode::Destination => "BLEND_DESTINATION".into(),
                BlendMode::Xor => "BLEND_XOR".into(),
                BlendMode::Plus => "BLEND_PLUS".into(),
                BlendMode::Modulate => "BLEND_MODULATE".into(),
                BlendMode::Screen => "BLEND_SCREEN".into(),
                BlendMode::Overlay => "BLEND_OVERLAY".into(),
                BlendMode::Darken => "BLEND_DARKEN".into(),
                BlendMode::Lighten => "BLEND_LIGHTEN".into(),
                BlendMode::ColorDodge => "BLEND_COLOR_DODGE".into(),
                BlendMode::ColorBurn => "BLEND_COLOR_BURN".into(),
                BlendMode::HardLight => "BLEND_HARD_LIGHT".into(),
                BlendMode::SoftLight => "BLEND_SOFT_LIGHT".into(),
                BlendMode::Difference => "BLEND_DIFFERENCE".into(),
                BlendMode::Exclusion => "BLEND_EXCLUSION".into(),
                BlendMode::Multiply => "BLEND_MULTIPLY".into(),
                BlendMode::Hue => "BLEND_HUE".into(),
                BlendMode::Saturation => "BLEND_SATURATION".into(),
                BlendMode::Color => "BLEND_COLOR".into(),
                BlendMode::Luminosity => "BLEND_LUMINOSITY".into(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Negation,
    Not,
    BitNot,
}

impl UnaryOperator {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Negation => "neg",
            Self::Not => "!",
            Self::BitNot => "~",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Modulo,
    Exponentiation,
    BitAnd,
    BitOr,
    BitXor,
    BitLeft,
    BitRight,
    EqualTo,
    NotEqualTo,
    LessThan,
    LessThanOrEqualTo,
    GreaterThan,
    GreaterThanOrEqualTo,
    And,
    Or,
    RangeExclusive,
    RangeInclusive,
    Concatenation,
    Prepend,
    Append,
    Composition,
    Pipe,
}

impl BinaryOperator {
    pub fn precedence(&self) -> u8 {
        match self {
            Self::Addition | Self::Subtraction => 7,
            Self::Multiplication | Self::Division | Self::Modulo => 8,
            Self::Exponentiation => 9,
            Self::EqualTo
            | Self::NotEqualTo
            | Self::LessThan
            | Self::LessThanOrEqualTo
            | Self::GreaterThan
            | Self::GreaterThanOrEqualTo => 3,
            Self::BitAnd | Self::And => 5,
            Self::BitOr | Self::BitXor | Self::Or => 4,
            Self::BitLeft | Self::BitRight | Self::RangeExclusive | Self::RangeInclusive => 6,
            Self::Concatenation | Self::Prepend | Self::Append => 2,
            Self::Composition => 0,
            Self::Pipe => 1,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Addition => "+",
            Self::Subtraction => "-",
            Self::Multiplication => "*",
            Self::Division => "/",
            Self::Modulo => "%",
            Self::Exponentiation => "**",
            Self::BitAnd => "&",
            Self::BitOr => "|",
            Self::BitXor => "^",
            Self::BitLeft => "<<",
            Self::BitRight => ">>",
            Self::EqualTo => "==",
            Self::NotEqualTo => "!=",
            Self::LessThan => "<",
            Self::LessThanOrEqualTo => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqualTo => ">=",
            Self::And => "&&",
            Self::Or => "||",
            Self::RangeExclusive => "..",
            Self::RangeInclusive => "..=",
            Self::Concatenation => "++",
            Self::Prepend => "+>",
            Self::Append => "<+",
            Self::Composition => ":",
            Self::Pipe => "|>",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Matches(Vec<Literal>),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    Literal(Literal),
    List(Vec<Block<'a>>),
    UnaryOperator(UnaryOperator),
    BinaryOperator(BinaryOperator),
    Call(&'a str, usize),
    Jump(usize),
    Pop,
    Return(Option<usize>),
    Let(&'a str, Vec<&'a str>, usize),
    If(usize),
    Match(Vec<(Pattern, usize)>),
    ForStart(&'a str),
    ForEnd,
    LoopStart,
    LoopEnd,
}

#[derive(Debug, PartialEq)]
pub struct Definition<'a> {
    pub name: &'a str,
    pub weight: f32,
    pub params: Vec<&'a str>,
    pub block: Block<'a>,
}

pub type Block<'a> = Vec<Token<'a>>;

pub type Tree<'a> = Vec<Definition<'a>>;

fn integer(input: &str) -> IResult<&str, Literal> {
    map(i32, Literal::Integer).parse(input)
}

fn float_value(input: &str) -> IResult<&str, f32> {
    map_res(
        recognize((
            opt(char('-')),
            alt((
                recognize((opt(digit1), char('.'), digit1)),
                recognize((
                    digit1,
                    opt((char('.'), digit1)),
                    alt((char('e'), char('E'))),
                    opt(alt((char('+'), char('-')))),
                    digit1,
                )),
            )),
        )),
        |s: &str| s.parse(),
    )
    .parse(input)
}

fn float(input: &str) -> IResult<&str, Literal> {
    map(float_value, Literal::Float).parse(input)
}

fn complex_value(input: &str) -> IResult<&str, Complex<f32>> {
    map_res(
        alt((
            recognize((
                opt(char('-')),
                alt((float_value, map(i32, |n| n as f32))),
                opt(alt((char('+'), char('-')))),
                opt(alt((float_value, map(i32, |n| n as f32)))),
                alt((char('i'), char('𝑖'), char('j'), char('𝑗'))),
            )),
            recognize((
                opt(char('-')),
                alt((float_value, map(i32, |n| n as f32))),
                alt((char('i'), char('𝑖'), char('j'), char('𝑗'))),
                alt((char('+'), char('-'))),
                alt((float_value, map(i32, |n| n as f32))),
            )),
        )),
        |s: &str| Complex::from_str(&s.replace("𝑖", "i").replace("𝑗", "j")),
    )
    .parse(input)
}

fn complex(input: &str) -> IResult<&str, Literal> {
    map(complex_value, Literal::Complex).parse(input)
}

fn boolean(input: &str) -> IResult<&str, Literal> {
    alt((
        value(Literal::Boolean(true), tag("true")),
        value(Literal::Boolean(false), tag("false")),
    ))
    .parse(input)
}

fn from_hex(input: &str) -> Result<u8, core::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn from_hex_single(c: &str) -> Result<u8, core::num::ParseIntError> {
    let mut input = String::new();
    input.push_str(c);
    input.push_str(c);
    u8::from_str_radix(&input, 16)
}

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex).parse(input)
}

fn hex_primary_single(input: &str) -> IResult<&str, u8> {
    map_res(take_while_m_n(1, 1, is_hex_digit), from_hex_single).parse(input)
}

fn hex(input: &str) -> IResult<&str, Literal> {
    let (input, _) = tag("0x")(input)?;
    let (input, (r, g, b)) = alt((
        (hex_primary, hex_primary, hex_primary),
        (hex_primary_single, hex_primary_single, hex_primary_single),
    ))
    .parse(input)?;

    Ok((input, Literal::Hex([r, g, b])))
}

fn shape(input: &str) -> IResult<&str, Literal> {
    map(
        alt((
            value(
                ShapeKind::Square,
                alt((
                    tag("SQUARE"),
                    tag("⬛"),
                    tag("⬜"),
                    tag("■"),
                    tag("□"),
                    tag("▪"),
                    tag("▫"),
                )),
            ),
            value(
                ShapeKind::Circle,
                alt((
                    tag("CIRCLE"),
                    tag("⬤"),
                    tag("◯"),
                    tag("●"),
                    tag("○"),
                    tag("🞄"),
                )),
            ),
            value(
                ShapeKind::Triangle,
                alt((tag("TRIANGLE"), tag("▲"), tag("△"), tag("▴"), tag("▵"))),
            ),
            value(ShapeKind::Fill, tag("FILL")),
            value(ShapeKind::Empty, tag("EMPTY")),
        )),
        Literal::Shape,
    )
    .parse(input)
}

fn blend_mode(input: &str) -> IResult<&str, Literal> {
    map(
        alt((
            alt((
                value(BlendMode::Clear, tag("BLEND_CLEAR")),
                value(BlendMode::SourceOver, tag("BLEND_SOURCE_OVER")),
                value(BlendMode::DestinationOver, tag("BLEND_DESTINATION_OVER")),
                value(BlendMode::SourceIn, tag("BLEND_SOURCE_IN")),
                value(BlendMode::DestinationIn, tag("BLEND_DESTINATION_IN")),
                value(BlendMode::SourceOut, tag("BLEND_SOURCE_OUT")),
                value(BlendMode::DestinationOut, tag("BLEND_DESTINATION_OUT")),
                value(BlendMode::SourceAtop, tag("BLEND_SOURCE_ATOP")),
                value(BlendMode::DestinationAtop, tag("BLEND_DESTINATION_ATOP")),
                value(BlendMode::Source, tag("BLEND_SOURCE")),
                value(BlendMode::Destination, tag("BLEND_DESTINATION")),
                value(BlendMode::Xor, tag("BLEND_XOR")),
                value(BlendMode::Plus, tag("BLEND_PLUS")),
                value(BlendMode::Modulate, tag("BLEND_MODULATE")),
                value(BlendMode::Screen, tag("BLEND_SCREEN")),
                value(BlendMode::Overlay, tag("BLEND_OVERLAY")),
                value(BlendMode::Darken, tag("BLEND_DARKEN")),
                value(BlendMode::Lighten, tag("BLEND_LIGHTEN")),
                value(BlendMode::ColorDodge, tag("BLEND_COLOR_DODGE")),
                value(BlendMode::ColorBurn, tag("BLEND_COLOR_BURN")),
                value(BlendMode::HardLight, tag("BLEND_HARD_LIGHT")),
            )),
            value(BlendMode::SoftLight, tag("BLEND_SOFT_LIGHT")),
            value(BlendMode::Difference, tag("BLEND_DIFFERENCE")),
            value(BlendMode::Exclusion, tag("BLEND_EXCLUSION")),
            value(BlendMode::Multiply, tag("BLEND_MULTIPLY")),
            value(BlendMode::Hue, tag("BLEND_HUE")),
            value(BlendMode::Saturation, tag("BLEND_SATURATION")),
            value(BlendMode::Color, tag("BLEND_COLOR")),
            value(BlendMode::Luminosity, tag("BLEND_LUMINOSITY")),
        )),
        Literal::BlendMode,
    )
    .parse(input)
}

fn literal(input: &str) -> IResult<&str, Literal> {
    alt((hex, complex, float, integer, boolean, shape, blend_mode)).parse(input)
}

fn end(input: &str) -> IResult<&str, &str> {
    preceded(space0, alt((tag(";"), line_ending, eof))).parse(input)
}

fn unary_operator_tag(input: &str) -> IResult<&str, &str> {
    alt((tag("!"), tag("~"))).parse(input)
}

fn unary_operator(input: &str) -> IResult<&str, UnaryOperator> {
    alt((
        value(
            UnaryOperator::Negation,
            (tag("-"), peek(not(alt((tag(">"), digit1))))),
        ),
        value(UnaryOperator::Not, tag("!")),
        value(UnaryOperator::BitNot, tag("~")),
    ))
    .parse(input)
}

fn binary_operator_tag(input: &str) -> IResult<&str, &str> {
    alt((
        alt((
            tag("++"),
            tag("+>"),
            tag("+"),
            recognize((tag("-"), peek(not(tag(">"))))),
            tag("**"),
            tag("*"),
            tag("/"),
            tag("%"),
            tag(":"),
            tag("=="),
            tag("!="),
            tag("<<"),
            tag("<="),
            tag("<+"),
            tag("<"),
            tag(">>"),
            tag(">="),
            tag(">"),
            tag("&&"),
            tag("&"),
            tag("||"),
        )),
        tag("|>"),
        tag("|"),
        tag("..="),
        tag(".."),
        tag("^"),
    ))
    .parse(input)
}

fn binary_operator(input: &str) -> IResult<&str, BinaryOperator> {
    alt((
        alt((
            value(BinaryOperator::Concatenation, tag("++")),
            value(BinaryOperator::Prepend, tag("+>")),
            value(BinaryOperator::Addition, tag("+")),
            value(BinaryOperator::Subtraction, (tag("-"), peek(not(tag(">"))))),
            value(BinaryOperator::Exponentiation, tag("**")),
            value(BinaryOperator::Multiplication, tag("*")),
            value(BinaryOperator::Division, tag("/")),
            value(BinaryOperator::Modulo, tag("%")),
            value(BinaryOperator::Composition, tag(":")),
            value(BinaryOperator::EqualTo, tag("==")),
            value(BinaryOperator::NotEqualTo, tag("!=")),
            value(BinaryOperator::BitLeft, tag("<<")),
            value(BinaryOperator::LessThanOrEqualTo, tag("<=")),
            value(BinaryOperator::Append, tag("<+")),
            value(BinaryOperator::LessThan, tag("<")),
            value(BinaryOperator::BitRight, tag(">>")),
            value(BinaryOperator::GreaterThanOrEqualTo, tag(">=")),
            value(BinaryOperator::GreaterThan, tag(">")),
            value(BinaryOperator::And, tag("&&")),
            value(BinaryOperator::BitAnd, tag("&")),
            value(BinaryOperator::Or, tag("||")),
        )),
        value(BinaryOperator::Pipe, tag("|>")),
        value(BinaryOperator::BitOr, tag("|")),
        value(BinaryOperator::RangeInclusive, tag("..=")),
        value(BinaryOperator::RangeExclusive, tag("..")),
        value(BinaryOperator::BitXor, tag("^")),
    ))
    .parse(input)
}

fn identifier(input: &str) -> IResult<&str, &str> {
    verify(
        alt((
            tag("π"),
            tag("τ"),
            tag("ℯ"),
            tag("φ"),
            recognize((
                alt((alpha1, tag("_"))),
                many0(alt((alphanumeric1, tag("_")))),
            )),
            delimited(tag("("), unary_operator_tag, tag(")")),
            delimited(tag("("), binary_operator_tag, tag(")")),
        )),
        |ident: &str| !KEYWORDS.contains(&ident),
    )
    .parse(input)
}

fn indentation(input: &str, indent: usize) -> IResult<&str, usize> {
    let (input, line_end) = opt(many1((space0, line_ending))).parse(input)?;
    let (input, spacing) = many0(alt((char(' '), char('\t')))).parse(input)?;
    if line_end.is_some() {
        if spacing.len() < indent as usize {
            return Err(Err::Error(Error::new(input, ErrorKind::Verify)));
        }
        Ok((input, spacing.len()))
    } else {
        Ok((input, indent))
    }
}

fn list(input: &str) -> IResult<&str, Block> {
    let (input, list) = delimited(
        (char('['), multispace0),
        separated_list0((multispace0, char(','), multispace0), expr(0, false)),
        (multispace0, char(']')),
    )
    .parse(input)?;
    Ok((input, vec![Token::List(list)]))
}

fn call(indent: usize, precedence: u8) -> impl FnMut(&str) -> IResult<&str, Block> {
    move |input| {
        if precedence < u8::MAX {
            let (input, name) = identifier(input)?;
            let (input, args) =
                many0(preceded(space1, expr_with_precedence(indent + 1, u8::MAX))).parse(input)?;

            let argc = args.len();
            let mut block = Vec::new();
            for arg in args {
                block.extend(arg);
            }
            block.push(Token::Call(name, argc));

            Ok((input, block))
        } else {
            let (input, name) = identifier(input)?;

            let mut block = Vec::with_capacity(1);
            block.push(Token::Call(name, 0));

            Ok((input, block))
        }
    }
}

fn let_definition(indent: usize) -> impl FnMut(&str) -> IResult<&str, Block> {
    move |input| {
        let (input, name) = identifier(input)?;
        let (input, params) = many0(preceded(multispace1, identifier)).parse(input)?;
        let (input, _) = preceded(multispace0, char('=')).parse(input)?;
        let (input, expr) = expr(indent + 1, false)(input)?;

        let mut block = Vec::with_capacity(expr.len() + 2);
        block.push(Token::Let(name, params, expr.len() + 1));
        block.extend(expr);
        block.push(Token::Return(None));

        Ok((input, block))
    }
}

fn let_statement(indent: usize) -> impl FnMut(&str) -> IResult<&str, Block> {
    move |input| {
        let (input, _) = (tag("let"), space1).parse(input)?;
        let (input, definitions) =
            separated_list1((end, multispace0), let_definition(indent + 1)).parse(input)?;
        let (input, _) = alt((
            (multispace0, tag("->")),
            (space0, tag(";")),
            (space0, peek(line_ending)),
        ))
        .parse(input)?;
        let (input, expr) = expr(indent + 1, true)(input)?;

        let mut block = Vec::new();
        for expr in definitions {
            block.extend(expr);
        }
        block.extend(expr);
        block.push(Token::Pop);

        Ok((input, block))
    }
}

fn if_statement(indent: usize) -> impl FnMut(&str) -> IResult<&str, Block> {
    move |input| {
        let (input, _) = tag("if")(input)?;
        let (input, condition) = preceded(space1, expr(indent + 1, true)).parse(input)?;
        let (input, _) =
            alt(((multispace0, tag("->")), (space0, peek(line_ending)))).parse(input)?;
        let (input, then_branch) = expr(indent + 1, true)(input)?;
        let (input, _) = (multispace0, tag("else")).parse(input)?;
        let (input, else_if) = opt(peek((multispace1, tag("if")))).parse(input)?;
        let (input, else_branch) = if else_if.is_some() {
            preceded(multispace0, if_statement(indent)).parse(input)?
        } else {
            let (input, _) =
                alt(((multispace0, tag("->")), (space0, peek(line_ending)))).parse(input)?;
            expr(indent + 1, true)(input)?
        };

        let mut block =
            Vec::with_capacity(condition.len() + then_branch.len() + else_branch.len() + 2);
        block.extend(condition);
        block.push(Token::If(then_branch.len() + 1));
        block.extend(then_branch);
        block.push(Token::Jump(else_branch.len()));
        block.extend(else_branch);

        Ok((input, block))
    }
}

fn pattern(indent: usize) -> impl FnMut(&str) -> IResult<&str, Pattern> {
    move |input| {
        let (input, _) = indentation(input, indent)?;
        alt((
            map(separated_list1(tag(","), literal), Pattern::Matches),
            value(Pattern::Wildcard, tag("_")),
        ))
        .parse(input)
    }
}

#[derive(Debug)]
struct PatternBlock<'a> {
    pattern: Pattern,
    expr: Block<'a>,
}

fn pattern_block(indent: usize) -> impl FnMut(&str) -> IResult<&str, PatternBlock> {
    move |input| {
        let (input, pattern) = pattern(indent)(input)?;
        let (input, _) =
            alt(((multispace0, tag("->")), (space0, peek(line_ending)))).parse(input)?;
        let (input, expr) = expr(indent + 1, true)(input)?;

        let pattern_block = PatternBlock { pattern, expr };
        Ok((input, pattern_block))
    }
}

fn match_statement(indent: usize) -> impl FnMut(&str) -> IResult<&str, Block> {
    move |input| {
        let (input, _) = tag("match")(input)?;
        let (input, condition) = preceded(space1, expr(indent + 1, true)).parse(input)?;
        let (input, _) =
            alt(((multispace0, tag("->")), (space0, peek(line_ending)))).parse(input)?;
        let (input, pattern_blocks) = many1(pattern_block(indent + 1)).parse(input)?;

        let total: usize = pattern_blocks
            .iter()
            .map(|pattern_block| pattern_block.expr.len() + 1)
            .sum();
        let mut skip = 0;

        let mut patterns = Vec::with_capacity(pattern_blocks.len());
        let mut flattened_blocks = Vec::new();

        for pattern_block in pattern_blocks {
            skip += pattern_block.expr.len() + 1;
            patterns.push((pattern_block.pattern, pattern_block.expr.len() + 1));
            flattened_blocks.extend(pattern_block.expr);
            flattened_blocks.push(Token::Jump(total - skip));
        }

        let mut block = Vec::with_capacity(condition.len() + flattened_blocks.len() + 1);
        block.extend(condition);
        block.push(Token::Match(patterns));
        block.extend(flattened_blocks);

        Ok((input, block))
    }
}

fn for_statement(indent: usize) -> impl FnMut(&str) -> IResult<&str, Block> {
    move |input| {
        let (input, _) = tag("for")(input)?;
        let (input, var) = preceded(space1, identifier).parse(input)?;
        let (input, _) = (multispace1, tag("in")).parse(input)?;
        let (input, iter) = delimited(
            space1,
            expr(indent + 1, true),
            alt(((multispace0, tag("->")), (space0, peek(line_ending)))),
        )
        .parse(input)?;
        let (input, expr) = expr(indent + 1, true)(input)?;

        let mut block = Vec::with_capacity(iter.len() + expr.len() + 3);
        block.extend(iter);
        block.push(Token::ForStart(var));
        block.extend(expr);
        block.push(Token::Pop);
        block.push(Token::ForEnd);

        Ok((input, block))
    }
}

fn loop_statement(indent: usize) -> impl FnMut(&str) -> IResult<&str, Block> {
    move |input| {
        let (input, _) = tag("loop")(input)?;
        let (input, count) = delimited(
            space1,
            expr(indent + 1, true),
            alt(((multispace0, tag("->")), (space0, peek(line_ending)))),
        )
        .parse(input)?;
        let (input, expr) = expr(indent + 1, true)(input)?;

        let mut block = Vec::with_capacity(count.len() + expr.len() + 2);
        block.extend(count);
        block.push(Token::LoopStart);
        block.extend(expr);
        block.push(Token::LoopEnd);

        Ok((input, block))
    }
}

fn expr_recursive(
    input: &str,
    indent: usize,
    consume_semicolon: bool,
    precedence: u8,
) -> IResult<&str, Block> {
    let (input, indent) = indentation(input, indent)?;

    let (input, unary_operator) = opt(unary_operator).parse(input)?;

    let (mut input, mut lhs) = alt((
        map(literal, |literal| vec![Token::Literal(literal)]),
        list,
        let_statement(indent),
        if_statement(indent),
        match_statement(indent),
        for_statement(indent),
        loop_statement(indent),
        call(indent, precedence),
        delimited(
            (char('('), multispace0),
            |input| expr_recursive(input, 0, true, 0),
            (multispace0, char(')')),
        ),
    ))
    .parse(input)?;

    if let Some(unary_operator) = unary_operator {
        lhs.push(Token::UnaryOperator(unary_operator));
    }

    while let Ok((next_input, op)) = preceded(multispace0, binary_operator).parse(input) {
        if op.precedence() < precedence {
            break;
        }

        let (next_input, rhs) = expr_recursive(next_input, indent + 1, true, op.precedence() + 1)?;
        input = next_input;

        lhs.extend(rhs);
        lhs.push(Token::BinaryOperator(op));
    }

    let (input, _) = if consume_semicolon {
        opt((space0, tag(";"))).parse(input)?
    } else {
        peek(opt((space0, tag(";")))).parse(input)?
    };

    Ok((input, lhs))
}

fn expr(indent: usize, consume_semicolon: bool) -> impl FnMut(&str) -> IResult<&str, Block> {
    move |input| expr_recursive(input, indent, consume_semicolon, 0)
}

fn expr_with_precedence(indent: usize, precedence: u8) -> impl FnMut(&str) -> IResult<&str, Block> {
    move |input| expr_recursive(input, indent, true, precedence)
}

fn block(input: &str, indent: usize) -> IResult<&str, Block> {
    terminated(expr(indent, true), end).parse(input)
}

fn definition(input: &str) -> IResult<&str, Definition> {
    let (input, name) = identifier(input)?;
    let (input, weight) = opt(preceded(
        char('@'),
        alt((float_value, map(i32, |n| n as f32))),
    ))
    .parse(input)?;
    let (input, params) = many0(preceded(multispace1, identifier)).parse(input)?;
    let (input, _) = preceded(multispace0, char('=')).parse(input)?;
    let (input, block) = block(input, 1)?;
    let definition = Definition {
        name,
        weight: weight.unwrap_or(1.0),
        params,
        block,
    };
    Ok((input, definition))
}

pub fn parse(input: &str) -> crate::Result<Tree> {
    let (_input, tree) = terminated(
        many0(preceded(many0((space0, line_ending)), definition)),
        (multispace0, eof),
    )
    .parse(input)
    .map_err(|_| crate::Error::ParseError)?;
    Ok(tree)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() {
        assert_eq!(literal("3"), Ok(("", Literal::Integer(3))));
        assert_eq!(literal("5.0"), Ok(("", Literal::Float(5.0))));
        assert_eq!(literal("0.5"), Ok(("", Literal::Float(0.5))));
        assert_eq!(literal(".5"), Ok(("", Literal::Float(0.5))));
        assert_eq!(
            literal("5+3i"),
            Ok(("", Literal::Complex(Complex::new(5.0, 3.0))))
        );
        assert_eq!(
            literal("5-3i"),
            Ok(("", Literal::Complex(Complex::new(5.0, -3.0))))
        );
        assert_eq!(
            literal("3i"),
            Ok(("", Literal::Complex(Complex::new(0.0, 3.0))))
        );
        assert_eq!(
            literal("-3i"),
            Ok(("", Literal::Complex(Complex::new(0.0, -3.0))))
        );
        assert_eq!(literal("true"), Ok(("", Literal::Boolean(true))));
        assert_eq!(literal("false"), Ok(("", Literal::Boolean(false))));
        assert_eq!(literal("0xf0ae13"), Ok(("", Literal::Hex([240, 174, 19]))));
        assert_eq!(
            literal("SQUARE"),
            Ok(("", Literal::Shape(ShapeKind::Square)))
        );
        assert_eq!(
            literal("CIRCLE"),
            Ok(("", Literal::Shape(ShapeKind::Circle)))
        );
        assert_eq!(
            literal("TRIANGLE"),
            Ok(("", Literal::Shape(ShapeKind::Triangle)))
        );
        assert_eq!(literal("FILL"), Ok(("", Literal::Shape(ShapeKind::Fill))));
        assert_eq!(literal("EMPTY"), Ok(("", Literal::Shape(ShapeKind::Empty))));
        assert_eq!(
            literal("BLEND_CLEAR"),
            Ok(("", Literal::BlendMode(BlendMode::Clear)))
        );
        assert_eq!(
            literal("BLEND_SOURCE_OVER"),
            Ok(("", Literal::BlendMode(BlendMode::SourceOver)))
        );
        assert_eq!(
            literal("BLEND_DESTINATION_OVER"),
            Ok(("", Literal::BlendMode(BlendMode::DestinationOver)))
        );
    }

    #[test]
    fn test_identifer() {
        assert_eq!(identifier("snake_case"), Ok(("", "snake_case")));
        assert_eq!(identifier("camelCase3"), Ok(("", "camelCase3")));
        assert_eq!(identifier("π"), Ok(("", "π")));
        assert_eq!(identifier("τ"), Ok(("", "τ")));
        assert_eq!(identifier("ℯ"), Ok(("", "ℯ")));
        assert_eq!(identifier("φ"), Ok(("", "φ")));
        assert_eq!(identifier("(!)"), Ok(("", "!")));
        assert_eq!(identifier("(+)"), Ok(("", "+")));
        assert_eq!(identifier("(*)"), Ok(("", "*")));
    }

    #[test]
    fn test_call() {
        assert_eq!(
            call(0, 0)("func arg1 (arg2 x) arg3"),
            Ok((
                "",
                vec![
                    Token::Call("arg1", 0),
                    Token::Call("x", 0),
                    Token::Call("arg2", 1),
                    Token::Call("arg3", 0),
                    Token::Call("func", 3)
                ]
            ))
        );
        assert_eq!(call(0, 0)("π"), Ok(("", vec![Token::Call("π", 0)])));
        assert_eq!(
            call(0, 0)("(+) 2 3"),
            Ok((
                "",
                vec![
                    Token::Literal(Literal::Integer(2)),
                    Token::Literal(Literal::Integer(3)),
                    Token::Call("+", 2)
                ]
            ))
        );
    }

    #[test]
    fn test_let_statement() {
        assert_eq!(
            let_statement(0)("let x = 3 -> x",),
            Ok((
                "",
                vec![
                    Token::Let("x", vec![], 2),
                    Token::Literal(Literal::Integer(3)),
                    Token::Return(None),
                    Token::Call("x", 0),
                    Token::Pop
                ]
            ))
        );
        assert_eq!(
            let_statement(0)(
                "\
let x = 3
    y = 5
    x + y\
                ",
            ),
            Ok((
                "",
                vec![
                    Token::Let("x", vec![], 2),
                    Token::Literal(Literal::Integer(3)),
                    Token::Return(None),
                    Token::Let("y", vec![], 2),
                    Token::Literal(Literal::Integer(5)),
                    Token::Return(None),
                    Token::Call("x", 0),
                    Token::Call("y", 0),
                    Token::BinaryOperator(BinaryOperator::Addition),
                    Token::Pop,
                ]
            ))
        );
    }

    #[test]
    fn test_if_statement() {
        assert_eq!(
            if_statement(0)("if x == 0 -> 2; else -> 3"),
            Ok((
                "",
                vec![
                    Token::Call("x", 0),
                    Token::Literal(Literal::Integer(0)),
                    Token::BinaryOperator(BinaryOperator::EqualTo),
                    Token::If(2),
                    Token::Literal(Literal::Integer(2)),
                    Token::Jump(1),
                    Token::Literal(Literal::Integer(3)),
                ]
            ))
        );
        assert_eq!(
            if_statement(0)(
                "\
if x > 0
    1
else if x < 0
    (-1)
else
    0\
                "
            ),
            Ok((
                "",
                vec![
                    Token::Call("x", 0),
                    Token::Literal(Literal::Integer(0)),
                    Token::BinaryOperator(BinaryOperator::GreaterThan),
                    Token::If(2),
                    Token::Literal(Literal::Integer(1)),
                    Token::Jump(7),
                    Token::Call("x", 0),
                    Token::Literal(Literal::Integer(0)),
                    Token::BinaryOperator(BinaryOperator::LessThan),
                    Token::If(2),
                    Token::Literal(Literal::Integer(-1)),
                    Token::Jump(1),
                    Token::Literal(Literal::Integer(0))
                ]
            ))
        );
    }

    #[test]
    fn test_match_statement() {
        assert_eq!(
            match_statement(0)("match n -> 3 -> 1; 2 -> 2; 1 -> 3"),
            Ok((
                "",
                vec![
                    Token::Call("n", 0),
                    Token::Match(vec![
                        (Pattern::Matches(vec![Literal::Integer(3)]), 2),
                        (Pattern::Matches(vec![Literal::Integer(2)]), 2),
                        (Pattern::Matches(vec![Literal::Integer(1)]), 2),
                    ]),
                    Token::Literal(Literal::Integer(1)),
                    Token::Jump(4),
                    Token::Literal(Literal::Integer(2)),
                    Token::Jump(2),
                    Token::Literal(Literal::Integer(3)),
                    Token::Jump(0)
                ]
            ))
        );
        assert_eq!(
            match_statement(0)(
                "\
match n
    3.0 -> 3
    2.0 -> 2
    1.0 -> 1\
                "
            ),
            Ok((
                "",
                vec![
                    Token::Call("n", 0),
                    Token::Match(vec![
                        (Pattern::Matches(vec![Literal::Float(3.0)]), 2),
                        (Pattern::Matches(vec![Literal::Float(2.0)]), 2),
                        (Pattern::Matches(vec![Literal::Float(1.0)]), 2),
                    ]),
                    Token::Literal(Literal::Integer(3)),
                    Token::Jump(4),
                    Token::Literal(Literal::Integer(2)),
                    Token::Jump(2),
                    Token::Literal(Literal::Integer(1)),
                    Token::Jump(0)
                ]
            ))
        );
    }

    #[test]
    fn test_for_statement() {
        assert_eq!(
            for_statement(0)("for i in 0..3 -> i"),
            Ok((
                "",
                vec![
                    Token::Literal(Literal::Integer(0)),
                    Token::Literal(Literal::Integer(3)),
                    Token::BinaryOperator(BinaryOperator::RangeExclusive),
                    Token::ForStart("i"),
                    Token::Call("i", 0),
                    Token::Pop,
                    Token::ForEnd
                ]
            ))
        );
        assert_eq!(
            for_statement(0)(
                "\
for i in 0..5
    i + 1\
                "
            ),
            Ok((
                "",
                vec![
                    Token::Literal(Literal::Integer(0)),
                    Token::Literal(Literal::Integer(5)),
                    Token::BinaryOperator(BinaryOperator::RangeExclusive),
                    Token::ForStart("i"),
                    Token::Call("i", 0),
                    Token::Literal(Literal::Integer(1)),
                    Token::BinaryOperator(BinaryOperator::Addition),
                    Token::Pop,
                    Token::ForEnd
                ]
            ))
        );
    }

    #[test]
    fn test_loop_statement() {
        assert_eq!(
            loop_statement(0)("loop 3 -> 0"),
            Ok((
                "",
                vec![
                    Token::Literal(Literal::Integer(3)),
                    Token::LoopStart,
                    Token::Literal(Literal::Integer(0)),
                    Token::LoopEnd
                ]
            ))
        );
        assert_eq!(
            loop_statement(0)(
                "\
loop 5
    rand * 10\
                "
            ),
            Ok((
                "",
                vec![
                    Token::Literal(Literal::Integer(5)),
                    Token::LoopStart,
                    Token::Call("rand", 0),
                    Token::Literal(Literal::Integer(10)),
                    Token::BinaryOperator(BinaryOperator::Multiplication),
                    Token::LoopEnd
                ]
            ))
        );
    }

    #[test]
    fn test_unary_operation() {
        assert_eq!(
            expr(0, true)("-3.0"),
            Ok(("", vec![Token::Literal(Literal::Float(-3.0)),]))
        );
        assert_eq!(
            expr(0, true)("-(3.0)"),
            Ok((
                "",
                vec![
                    Token::Literal(Literal::Float(3.0)),
                    Token::UnaryOperator(UnaryOperator::Negation),
                ]
            ))
        );
        assert_eq!(
            expr(0, true)("!true"),
            Ok((
                "",
                vec![
                    Token::Literal(Literal::Boolean(true)),
                    Token::UnaryOperator(UnaryOperator::Not),
                ]
            ))
        );
        assert_eq!(
            expr(0, true)("~5"),
            Ok((
                "",
                vec![
                    Token::Literal(Literal::Integer(5)),
                    Token::UnaryOperator(UnaryOperator::BitNot),
                ]
            ))
        );
    }

    #[test]
    fn test_binary_operation() {
        assert_eq!(
            expr(0, true)("5 - 3.0"),
            Ok((
                "",
                vec![
                    Token::Literal(Literal::Integer(5)),
                    Token::Literal(Literal::Float(3.0)),
                    Token::BinaryOperator(BinaryOperator::Subtraction),
                ]
            ))
        );
        assert_eq!(
            expr(0, true)("SQUARE : CIRCLE"),
            Ok((
                "",
                vec![
                    Token::Literal(Literal::Shape(ShapeKind::Square)),
                    Token::Literal(Literal::Shape(ShapeKind::Circle)),
                    Token::BinaryOperator(BinaryOperator::Composition),
                ]
            ))
        );
    }

    #[test]
    fn test_definition() {
        assert_eq!(
            definition("addition x y = x + y"),
            Ok((
                "",
                Definition {
                    name: "addition",
                    params: vec!["x", "y"],
                    block: vec![
                        Token::Call("x", 0),
                        Token::Call("y", 0),
                        Token::BinaryOperator(BinaryOperator::Addition)
                    ],
                    weight: 1.0,
                }
            ))
        );
        assert_eq!(
            definition(
                "\
draw@5 =
    SQUARE\
                "
            ),
            Ok((
                "",
                Definition {
                    name: "draw",
                    params: vec![],
                    block: vec![Token::Literal(Literal::Shape(ShapeKind::Square))],
                    weight: 5.0,
                }
            ))
        );
    }

    #[test]
    fn test_parse() {
        let tree = parse(
            "\
root = collect (shapes 5)

shapes@3 x =
    loop x
        SQUARE

shapes@2 x =
    for i in 0..x
        if i % 2 == 0
            CIRCLE
        else
            TRIANGLE
            ",
        );
        assert!(tree.is_ok());
        assert_eq!(
            tree.unwrap(),
            vec![
                Definition {
                    name: "root",
                    params: vec![],
                    block: vec![
                        Token::Literal(Literal::Integer(5)),
                        Token::Call("shapes", 1),
                        Token::Call("collect", 1)
                    ],
                    weight: 1.0,
                },
                Definition {
                    name: "shapes",
                    params: vec!["x"],
                    block: vec![
                        Token::Call("x", 0),
                        Token::LoopStart,
                        Token::Literal(Literal::Shape(ShapeKind::Square)),
                        Token::LoopEnd
                    ],
                    weight: 3.0,
                },
                Definition {
                    name: "shapes",
                    params: vec!["x"],
                    block: vec![
                        Token::Literal(Literal::Integer(0)),
                        Token::Call("x", 0),
                        Token::BinaryOperator(BinaryOperator::RangeExclusive),
                        Token::ForStart("i"),
                        Token::Call("i", 0),
                        Token::Literal(Literal::Integer(2)),
                        Token::BinaryOperator(BinaryOperator::Modulo),
                        Token::Literal(Literal::Integer(0)),
                        Token::BinaryOperator(BinaryOperator::EqualTo),
                        Token::If(2),
                        Token::Literal(Literal::Shape(ShapeKind::Circle)),
                        Token::Jump(1),
                        Token::Literal(Literal::Shape(ShapeKind::Triangle)),
                        Token::Pop,
                        Token::ForEnd
                    ],
                    weight: 2.0,
                },
            ]
        );
    }
}
