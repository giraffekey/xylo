#[cfg(feature = "no-std")]
use alloc::{boxed::Box, vec::Vec};

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while_m_n};
use nom::character::complete::{
    alpha1, alphanumeric1, char, digit1, i32, line_ending, multispace0, multispace1, space0, space1,
};
use nom::combinator::{eof, map, map_res, not, opt, peek, recognize, value, verify};
use nom::error::{Error, ErrorKind};
use nom::multi::{many0, many1, separated_list1};
use nom::sequence::{delimited, preceded, terminated};
use nom::{Err, IResult, Parser};

const KEYWORDS: &[&str] = &["let", "if", "else", "match", "for", "loop"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Triangle,
    Square,
    Circle,
    Fill,
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i32),
    Float(f32),
    Boolean(bool),
    Hex([u8; 3]),
    Shape(ShapeKind),
    List(Vec<Literal>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Modulo,
    Exponentiation,
    EqualTo,
    NotEqualTo,
    LessThan,
    LessThanOrEqualTo,
    GreaterThan,
    GreaterThanOrEqualTo,
    And,
    Or,
    Composition,
    RangeExclusive,
    RangeInclusive,
}

impl BinaryOperator {
    pub fn precedence(&self) -> u8 {
        match self {
            Self::Addition | Self::Subtraction => 4,
            Self::Multiplication | Self::Division | Self::Modulo => 5,
            Self::Exponentiation => 6,
            Self::Composition
            | Self::EqualTo
            | Self::NotEqualTo
            | Self::LessThan
            | Self::LessThanOrEqualTo
            | Self::GreaterThan
            | Self::GreaterThanOrEqualTo => 0,
            Self::And => 2,
            Self::Or => 1,
            Self::RangeExclusive | Self::RangeInclusive => 3,
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
            Self::EqualTo => "==",
            Self::NotEqualTo => "!=",
            Self::LessThan => "<",
            Self::LessThanOrEqualTo => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqualTo => ">=",
            Self::And => "&&",
            Self::Or => "||",
            Self::Composition => ":",
            Self::RangeExclusive => "..",
            Self::RangeInclusive => "..=",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetDefinition<'a> {
    pub name: &'a str,
    pub params: Vec<&'a str>,
    pub block: Block<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Matches(Vec<Literal>),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    Literal(Literal),
    BinaryOperator(BinaryOperator),
    Call(&'a str, usize),
    Jump(usize),
    Let(Vec<LetDefinition<'a>>, usize),
    If(usize),
    Match(Vec<(Pattern, usize)>),
    For(&'a str, usize),
    Loop(usize),
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
    i32.parse(input)
        .map(|(input, value)| (input, Literal::Integer(value)))
}

fn float_value(input: &str) -> IResult<&str, f32> {
    let (input, s) = recognize((opt(char('-')), (opt(digit1), char('.'), digit1))).parse(input)?;
    Ok((input, s.parse().unwrap()))
}

fn float(input: &str) -> IResult<&str, Literal> {
    float_value(input).map(|(input, value)| (input, Literal::Float(value)))
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
    let (input, _) = tag("#")(input)?;
    let (input, (r, g, b)) = alt((
        (hex_primary, hex_primary, hex_primary),
        (hex_primary_single, hex_primary_single, hex_primary_single),
    ))
    .parse(input)?;

    Ok((input, Literal::Hex([r, g, b])))
}

fn shape(input: &str) -> IResult<&str, Literal> {
    let (input, shape) = alt((
        value(ShapeKind::Triangle, tag("TRIANGLE")),
        value(ShapeKind::Square, tag("SQUARE")),
        value(ShapeKind::Circle, tag("CIRCLE")),
        value(ShapeKind::Fill, tag("FILL")),
        value(ShapeKind::Empty, tag("EMPTY")),
    ))
    .parse(input)?;
    Ok((input, Literal::Shape(shape)))
}

fn list(input: &str) -> IResult<&str, Literal> {
    let (input, list) = delimited(
        (char('['), multispace0),
        separated_list1((multispace0, char(','), multispace0), literal),
        (multispace0, char(']')),
    )
    .parse(input)?;
    Ok((input, Literal::List(*Box::new(list))))
}

fn literal(input: &str) -> IResult<&str, Literal> {
    alt((float, integer, boolean, hex, shape, list)).parse(input)
}

fn end(input: &str) -> IResult<&str, &str> {
    preceded(space0, alt((tag(";"), line_ending, eof))).parse(input)
}

fn block(input: &str, indent: usize) -> IResult<&str, Block> {
    terminated(expr(indent), end).parse(input)
}

fn _binary_operator_tag(input: &str) -> IResult<&str, &str> {
    alt((
        tag("+"),
        recognize((tag("-"), peek(not(tag(">"))))),
        tag("**"),
        tag("*"),
        tag("/"),
        tag("%"),
        tag(":"),
        tag("=="),
        tag("!="),
        tag("<="),
        tag("<"),
        tag(">="),
        tag(">"),
        tag("&&"),
        tag("||"),
        tag("..="),
        tag(".."),
    ))
    .parse(input)
}

fn binary_operator(input: &str) -> IResult<&str, BinaryOperator> {
    alt((
        value(BinaryOperator::Addition, tag("+")),
        value(BinaryOperator::Subtraction, (tag("-"), peek(not(tag(">"))))),
        value(BinaryOperator::Exponentiation, tag("**")),
        value(BinaryOperator::Multiplication, tag("*")),
        value(BinaryOperator::Division, tag("/")),
        value(BinaryOperator::Modulo, tag("%")),
        value(BinaryOperator::Composition, tag(":")),
        value(BinaryOperator::EqualTo, tag("==")),
        value(BinaryOperator::NotEqualTo, tag("!=")),
        value(BinaryOperator::LessThanOrEqualTo, tag("<=")),
        value(BinaryOperator::LessThan, tag("<")),
        value(BinaryOperator::GreaterThanOrEqualTo, tag(">=")),
        value(BinaryOperator::GreaterThan, tag(">")),
        value(BinaryOperator::And, tag("&&")),
        value(BinaryOperator::Or, tag("||")),
        value(BinaryOperator::RangeInclusive, tag("..=")),
        value(BinaryOperator::RangeExclusive, tag("..")),
    ))
    .parse(input)
}

fn identifier(input: &str) -> IResult<&str, &str> {
    verify(
        alt((
            tag("π"),
            recognize((
                alt((alpha1, tag("_"))),
                many0(alt((alphanumeric1, tag("_")))),
            )),
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

fn let_definition(indent: usize) -> impl FnMut(&str) -> IResult<&str, LetDefinition> {
    move |input| {
        let (input, name) = identifier(input)?;
        let (input, params) = many0(preceded(multispace1, identifier)).parse(input)?;
        let (input, _) = preceded(multispace0, char('=')).parse(input)?;
        let (input, block) = expr(indent + 1)(input)?;
        let definition = LetDefinition {
            name,
            params,
            block,
        };
        Ok((input, definition))
    }
}

fn let_statement(indent: usize) -> impl FnMut(&str) -> IResult<&str, Block> {
    move |input| {
        let (input, _) = (tag("let"), space1).parse(input)?;
        let (input, definitions) =
            separated_list1((end, multispace0), let_definition(indent + 1)).parse(input)?;
        let (input, _) = alt((
            (multispace1, tag("->")),
            (space0, tag(";")),
            (space0, peek(line_ending)),
        ))
        .parse(input)?;
        let (input, expr) = block(input, indent + 1)?;

        let mut block = Vec::with_capacity(expr.len() + 1);
        block.push(Token::Let(definitions, expr.len()));
        block.extend(expr);

        Ok((input, block))
    }
}

fn if_statement(indent: usize) -> impl FnMut(&str) -> IResult<&str, Block> {
    move |input| {
        let (input, _) = tag("if")(input)?;
        let (input, condition) = preceded(space1, expr(indent + 1)).parse(input)?;
        let (input, _) =
            alt(((multispace1, tag("->")), (space0, peek(line_ending)))).parse(input)?;
        let (input, then_branch) = block(input, indent + 1)?;
        let (input, _) = (multispace0, tag("else")).parse(input)?;
        let (input, else_if) = opt(peek((multispace1, tag("if")))).parse(input)?;
        let (input, else_branch) = if else_if.is_some() {
            expr(indent)(input)?
        } else {
            let (input, _) =
                alt(((multispace1, tag("->")), (space0, peek(line_ending)))).parse(input)?;
            block(input, indent + 1)?
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
    block: Block<'a>,
}

fn pattern_block(indent: usize) -> impl FnMut(&str) -> IResult<&str, PatternBlock> {
    move |input| {
        let (input, pattern) = pattern(indent)(input)?;
        let (input, _) =
            alt(((multispace1, tag("->")), (space0, peek(line_ending)))).parse(input)?;
        let (input, block) = block(input, indent + 1)?;

        let pattern_block = PatternBlock { pattern, block };
        Ok((input, pattern_block))
    }
}

fn match_statement(indent: usize) -> impl FnMut(&str) -> IResult<&str, Block> {
    move |input| {
        let (input, _) = tag("match")(input)?;
        let (input, condition) = preceded(space1, expr(indent + 1)).parse(input)?;
        let (input, _) =
            alt(((multispace1, tag("->")), (space0, peek(line_ending)))).parse(input)?;
        let (input, pattern_blocks) = many1(pattern_block(indent + 1)).parse(input)?;

        let total: usize = pattern_blocks
            .iter()
            .map(|pattern_block| pattern_block.block.len() + 1)
            .sum();
        let mut skip = 0;

        let mut patterns = Vec::with_capacity(pattern_blocks.len());
        let mut flattened_blocks = Vec::new();

        for pattern_block in pattern_blocks {
            skip += pattern_block.block.len() + 1;
            patterns.push((pattern_block.pattern, pattern_block.block.len() + 1));
            flattened_blocks.extend(pattern_block.block);
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
            expr(indent + 1),
            alt(((multispace1, tag("->")), (space0, peek(line_ending)))),
        )
        .parse(input)?;
        let (input, expr) = block(input, indent + 1)?;

        let mut block = Vec::with_capacity(iter.len() + expr.len() + 1);
        block.extend(iter);
        block.push(Token::For(var, expr.len()));
        block.extend(expr);

        Ok((input, block))
    }
}

fn loop_statement(indent: usize) -> impl FnMut(&str) -> IResult<&str, Block> {
    move |input| {
        let (input, _) = tag("loop")(input)?;
        let (input, count) = delimited(
            space1,
            expr(indent + 1),
            alt(((multispace1, tag("->")), (space0, peek(line_ending)))),
        )
        .parse(input)?;
        let (input, expr) = block(input, indent + 1)?;

        let mut block = Vec::with_capacity(count.len() + expr.len() + 1);
        block.extend(count);
        block.push(Token::Loop(expr.len()));
        block.extend(expr);

        Ok((input, block))
    }
}

fn expr_recursive(input: &str, indent: usize, precedence: u8) -> IResult<&str, Block> {
    let (input, indent) = indentation(input, indent)?;

    let (mut input, mut lhs) = alt((
        map(literal, |literal| vec![Token::Literal(literal)]),
        let_statement(indent),
        if_statement(indent),
        match_statement(indent),
        for_statement(indent),
        loop_statement(indent),
        call(indent, precedence),
        delimited(
            (char('('), multispace0),
            |input| expr_recursive(input, 0, 0),
            (multispace0, char(')')),
        ),
    ))
    .parse(input)?;

    while let Ok((next_input, op)) = preceded(multispace0, binary_operator).parse(input) {
        if op.precedence() < precedence {
            break;
        }
        let (next_input, rhs) = expr_recursive(next_input, indent + 1, op.precedence() + 1)?;
        input = next_input;

        let mut block = Vec::with_capacity(lhs.len() + rhs.len() + 1);
        block.extend(lhs);
        block.extend(rhs);
        block.push(Token::BinaryOperator(op));

        lhs = block;
    }

    Ok((input, lhs))
}

fn expr(indent: usize) -> impl FnMut(&str) -> IResult<&str, Block> {
    move |input| expr_recursive(input, indent, 0)
}

fn expr_with_precedence(indent: usize, precedence: u8) -> impl FnMut(&str) -> IResult<&str, Block> {
    move |input| expr_recursive(input, indent, precedence)
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

pub fn parse(input: &str) -> IResult<&str, Tree> {
    let tree = terminated(
        many0(preceded(many0((space0, line_ending)), definition)),
        (multispace0, eof),
    )
    .parse(input);
    tree
}
