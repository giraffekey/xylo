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
pub struct BinaryOperation<'a> {
    pub op: BinaryOperator,
    pub a: Box<Expr<'a>>,
    pub b: Box<Expr<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call<'a> {
    pub name: &'a str,
    pub args: Vec<Expr<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Let<'a> {
    pub definitions: Vec<Definition<'a>>,
    pub block: Box<Expr<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct If<'a> {
    pub condition: Box<Expr<'a>>,
    pub if_block: Box<Expr<'a>>,
    pub else_block: Box<Expr<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern<'a> {
    Matches(Vec<Expr<'a>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Match<'a> {
    pub condition: Box<Expr<'a>>,
    pub patterns: Vec<(Pattern<'a>, Expr<'a>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct For<'a> {
    pub var: &'a str,
    pub iter: Box<Expr<'a>>,
    pub block: Box<Expr<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Loop<'a> {
    pub count: Box<Expr<'a>>,
    pub block: Box<Expr<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'a> {
    Literal(Literal),
    BinaryOperation(BinaryOperation<'a>),
    Call(Call<'a>),
    Let(Let<'a>),
    If(If<'a>),
    Match(Match<'a>),
    For(For<'a>),
    Loop(Loop<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Definition<'a> {
    pub name: &'a str,
    pub weight: Option<f32>,
    pub params: Vec<&'a str>,
    pub block: Expr<'a>,
}

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

fn block(input: &str, indent: usize) -> IResult<&str, Expr> {
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

fn call(indent: usize, precedence: u8) -> impl FnMut(&str) -> IResult<&str, Call> {
    move |input| {
        if precedence < u8::MAX {
            let (input, name) = identifier(input)?;
            let (input, args) =
                many0(preceded(space1, expr_with_precedence(indent + 1, u8::MAX))).parse(input)?;
            let call = Call { name, args };
            Ok((input, call))
        } else {
            let (input, name) = identifier(input)?;
            let call = Call {
                name,
                args: Vec::new(),
            };
            Ok((input, call))
        }
    }
}

fn let_definition(indent: usize) -> impl FnMut(&str) -> IResult<&str, Definition> {
    move |input| {
        let (input, name) = identifier(input)?;
        let (input, params) = many0(preceded(multispace1, identifier)).parse(input)?;
        let (input, _) = preceded(multispace0, char('=')).parse(input)?;
        let (input, block) = expr(indent + 1)(input)?;
        let definition = Definition {
            name,
            weight: None,
            params,
            block,
        };
        Ok((input, definition))
    }
}

fn let_statement(indent: usize) -> impl FnMut(&str) -> IResult<&str, Let> {
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
        let (input, block) = block(input, indent + 1)?;
        let let_statement = Let {
            definitions,
            block: Box::new(block),
        };
        Ok((input, let_statement))
    }
}

fn if_statement(indent: usize) -> impl FnMut(&str) -> IResult<&str, If> {
    move |input| {
        let (input, _) = tag("if")(input)?;
        let (input, condition) = preceded(space1, expr(indent + 1)).parse(input)?;
        let (input, _) =
            alt(((multispace1, tag("->")), (space0, peek(line_ending)))).parse(input)?;
        let (input, if_block) = block(input, indent + 1)?;
        let (input, _) = (multispace0, tag("else")).parse(input)?;
        let (input, else_if) = opt(peek((multispace1, tag("if")))).parse(input)?;
        let (input, else_block) = if else_if.is_some() {
            expr(indent)(input)?
        } else {
            let (input, _) =
                alt(((multispace1, tag("->")), (space0, peek(line_ending)))).parse(input)?;
            block(input, indent + 1)?
        };
        let if_statement = If {
            condition: Box::new(condition),
            if_block: Box::new(if_block),
            else_block: Box::new(else_block),
        };
        Ok((input, if_statement))
    }
}

fn pattern(indent: usize) -> impl FnMut(&str) -> IResult<&str, Pattern> {
    move |input| map(separated_list1(tag(","), expr(indent)), Pattern::Matches).parse(input)
}

fn pattern_block(indent: usize) -> impl FnMut(&str) -> IResult<&str, (Pattern, Expr)> {
    move |input| {
        let (input, pattern) = pattern(indent)(input)?;
        let (input, _) =
            alt(((multispace1, tag("->")), (space0, peek(line_ending)))).parse(input)?;
        let (input, block) = block(input, indent + 1)?;
        Ok((input, (pattern, block)))
    }
}

fn match_statement(indent: usize) -> impl FnMut(&str) -> IResult<&str, Match> {
    move |input| {
        let (input, _) = tag("match")(input)?;
        let (input, condition) = preceded(space1, expr(indent + 1)).parse(input)?;
        let (input, _) =
            alt(((multispace1, tag("->")), (space0, peek(line_ending)))).parse(input)?;
        let (input, patterns) = many1(pattern_block(indent + 1)).parse(input)?;
        let match_statement = Match {
            condition: Box::new(condition),
            patterns,
        };
        Ok((input, match_statement))
    }
}

fn for_statement(indent: usize) -> impl FnMut(&str) -> IResult<&str, For> {
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
        let (input, block) = block(input, indent + 1)?;
        let for_statement = For {
            var,
            iter: Box::new(iter),
            block: Box::new(block),
        };
        Ok((input, for_statement))
    }
}

fn loop_statement(indent: usize) -> impl FnMut(&str) -> IResult<&str, Loop> {
    move |input| {
        let (input, _) = tag("loop")(input)?;
        let (input, count) = delimited(
            space1,
            expr(indent + 1),
            alt(((multispace1, tag("->")), (space0, peek(line_ending)))),
        )
        .parse(input)?;
        let (input, block) = block(input, indent + 1)?;
        let for_statement = Loop {
            count: Box::new(count),
            block: Box::new(block),
        };
        Ok((input, for_statement))
    }
}

fn expr_recursive(input: &str, indent: usize, precedence: u8) -> IResult<&str, Expr> {
    let (input, line_end) = opt(many1((space0, line_ending))).parse(input)?;
    let (input, spacing) = many0(alt((char(' '), char('\t')))).parse(input)?;
    let indent = if line_end.is_some() {
        if spacing.len() < indent as usize {
            return Err(Err::Error(Error::new(input, ErrorKind::Verify)));
        }
        spacing.len()
    } else {
        indent
    };

    let (mut input, mut lhs) = alt((
        map(literal, Expr::Literal),
        map(let_statement(indent), Expr::Let),
        map(if_statement(indent), Expr::If),
        map(match_statement(indent), Expr::Match),
        map(for_statement(indent), Expr::For),
        map(loop_statement(indent), Expr::Loop),
        map(call(indent, precedence), Expr::Call),
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
        lhs = Expr::BinaryOperation(BinaryOperation {
            op,
            a: Box::new(lhs),
            b: Box::new(rhs),
        });
    }

    Ok((input, lhs))
}

fn expr(indent: usize) -> impl FnMut(&str) -> IResult<&str, Expr> {
    move |input| expr_recursive(input, indent, 0)
}

fn expr_with_precedence(indent: usize, precedence: u8) -> impl FnMut(&str) -> IResult<&str, Expr> {
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
        weight: Some(weight.unwrap_or(1.0)),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "no-std")]
    use alloc::vec;

    #[test]
    fn test() {
        let res = parse(
            "
root =
    add_circle SQUARE

add_circle shape = shape : CIRCLE

weighted@2 = 3
weighted@3 = 2

MATH =
    3 + 4.0 * 5.0 + 6..(1 + 2) * 3

LET =
    let x = 3.0; y = 2.0
        (x + y) / 2.0

IF n =
    if n > 0
        TRIANGLE
    else if n < 0
        SQUARE
    else
        CIRCLE

MATCH n =
    match n
        0,1,2 -> true
        3..6
            false

FOR =
    for i in 1..=3
        i * i

LOOP =
    loop 3 -> SQUARE
            ",
        );
        assert!(res.is_ok(), "{}", res.unwrap_err());

        let (remaining, tree) = res.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(
            tree,
            vec![
                Definition {
                    name: "root",
                    weight: Some(1.0),
                    params: vec![],
                    block: Expr::Call(Call {
                        name: "add_circle",
                        args: vec![Expr::Literal(Literal::Shape(ShapeKind::Square))]
                    }),
                },
                Definition {
                    name: "add_circle",
                    weight: Some(1.0),
                    params: vec!["shape"],
                    block: Expr::BinaryOperation(BinaryOperation {
                        op: BinaryOperator::Composition,
                        a: Box::new(Expr::Call(Call {
                            name: "shape",
                            args: vec![]
                        })),
                        b: Box::new(Expr::Literal(Literal::Shape(ShapeKind::Circle))),
                    }),
                },
                Definition {
                    name: "weighted",
                    weight: Some(2.0),
                    params: vec![],
                    block: Expr::Literal(Literal::Integer(3)),
                },
                Definition {
                    name: "weighted",
                    weight: Some(3.0),
                    params: vec![],
                    block: Expr::Literal(Literal::Integer(2)),
                },
                Definition {
                    name: "MATH",
                    weight: Some(1.0),
                    params: vec![],
                    block: Expr::BinaryOperation(BinaryOperation {
                        op: BinaryOperator::RangeExclusive,
                        a: Box::new(Expr::BinaryOperation(BinaryOperation {
                            op: BinaryOperator::Addition,
                            a: Box::new(Expr::BinaryOperation(BinaryOperation {
                                op: BinaryOperator::Addition,
                                a: Box::new(Expr::Literal(Literal::Integer(3))),
                                b: Box::new(Expr::BinaryOperation(BinaryOperation {
                                    op: BinaryOperator::Multiplication,
                                    a: Box::new(Expr::Literal(Literal::Float(4.0))),
                                    b: Box::new(Expr::Literal(Literal::Float(5.0))),
                                })),
                            })),
                            b: Box::new(Expr::Literal(Literal::Integer(6))),
                        })),
                        b: Box::new(Expr::BinaryOperation(BinaryOperation {
                            op: BinaryOperator::Multiplication,
                            a: Box::new(Expr::BinaryOperation(BinaryOperation {
                                op: BinaryOperator::Addition,
                                a: Box::new(Expr::Literal(Literal::Integer(1))),
                                b: Box::new(Expr::Literal(Literal::Integer(2))),
                            })),
                            b: Box::new(Expr::Literal(Literal::Integer(3))),
                        })),
                    }),
                },
                Definition {
                    name: "LET",
                    weight: Some(1.0),
                    params: vec![],
                    block: Expr::Let(Let {
                        definitions: vec![
                            Definition {
                                name: "x",
                                weight: None,
                                params: vec![],
                                block: Expr::Literal(Literal::Float(3.0)),
                            },
                            Definition {
                                name: "y",
                                weight: None,
                                params: vec![],
                                block: Expr::Literal(Literal::Float(2.0)),
                            },
                        ],
                        block: Box::new(Expr::BinaryOperation(BinaryOperation {
                            op: BinaryOperator::Division,
                            a: Box::new(Expr::BinaryOperation(BinaryOperation {
                                op: BinaryOperator::Addition,
                                a: Box::new(Expr::Call(Call {
                                    name: "x",
                                    args: vec![],
                                })),
                                b: Box::new(Expr::Call(Call {
                                    name: "y",
                                    args: vec![],
                                })),
                            })),
                            b: Box::new(Expr::Literal(Literal::Float(2.0))),
                        })),
                    }),
                },
                Definition {
                    name: "IF",
                    weight: Some(1.0),
                    params: vec!["n"],
                    block: Expr::If(If {
                        condition: Box::new(Expr::BinaryOperation(BinaryOperation {
                            op: BinaryOperator::GreaterThan,
                            a: Box::new(Expr::Call(Call {
                                name: "n",
                                args: vec![],
                            })),
                            b: Box::new(Expr::Literal(Literal::Integer(0))),
                        })),
                        if_block: Box::new(Expr::Literal(Literal::Shape(ShapeKind::Triangle))),
                        else_block: Box::new(Expr::If(If {
                            condition: Box::new(Expr::BinaryOperation(BinaryOperation {
                                op: BinaryOperator::LessThan,
                                a: Box::new(Expr::Call(Call {
                                    name: "n",
                                    args: vec![],
                                })),
                                b: Box::new(Expr::Literal(Literal::Integer(0))),
                            })),
                            if_block: Box::new(Expr::Literal(Literal::Shape(ShapeKind::Square))),
                            else_block: Box::new(Expr::Literal(Literal::Shape(ShapeKind::Circle))),
                        })),
                    }),
                },
                Definition {
                    name: "MATCH",
                    weight: Some(1.0),
                    params: vec!["n"],
                    block: Expr::Match(Match {
                        condition: Box::new(Expr::Call(Call {
                            name: "n",
                            args: vec![],
                        })),
                        patterns: vec![
                            (
                                Pattern::Matches(vec![
                                    Expr::Literal(Literal::Integer(0)),
                                    Expr::Literal(Literal::Integer(1)),
                                    Expr::Literal(Literal::Integer(2)),
                                ]),
                                Expr::Literal(Literal::Boolean(true))
                            ),
                            (
                                Pattern::Matches(vec![Expr::BinaryOperation(BinaryOperation {
                                    op: BinaryOperator::RangeExclusive,
                                    a: Box::new(Expr::Literal(Literal::Integer(3))),
                                    b: Box::new(Expr::Literal(Literal::Integer(6))),
                                })]),
                                Expr::Literal(Literal::Boolean(false))
                            ),
                        ],
                    }),
                },
                Definition {
                    name: "FOR",
                    weight: Some(1.0),
                    params: vec![],
                    block: Expr::For(For {
                        var: "i",
                        iter: Box::new(Expr::BinaryOperation(BinaryOperation {
                            op: BinaryOperator::RangeInclusive,
                            a: Box::new(Expr::Literal(Literal::Integer(1))),
                            b: Box::new(Expr::Literal(Literal::Integer(3))),
                        })),
                        block: Box::new(Expr::BinaryOperation(BinaryOperation {
                            op: BinaryOperator::Multiplication,
                            a: Box::new(Expr::Call(Call {
                                name: "i",
                                args: vec![],
                            })),
                            b: Box::new(Expr::Call(Call {
                                name: "i",
                                args: vec![],
                            })),
                        })),
                    }),
                },
                Definition {
                    name: "LOOP",
                    weight: Some(1.0),
                    params: vec![],
                    block: Expr::Loop(Loop {
                        count: Box::new(Expr::Literal(Literal::Integer(3))),
                        block: Box::new(Expr::Literal(Literal::Shape(ShapeKind::Square))),
                    }),
                },
            ]
        );
    }
}
