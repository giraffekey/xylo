#[cfg(feature = "no-std")]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::colors::color;

use asdf_pixel_sort::{DEFAULT_BLACK, DEFAULT_BRIGHTNESS, DEFAULT_WHITE};
use core::str::FromStr;
use image::imageops::FilterType;
use imageproc::distance_transform::Norm;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while_m_n};
use nom::character::complete::{
    alpha1, alphanumeric1, char, digit1, i32, line_ending, multispace0, multispace1, none_of,
    space0, space1,
};
use nom::combinator::{eof, map, map_res, not, opt, peek, recognize, value, verify};
use nom::error::{Error, ErrorKind};
use nom::multi::{many0, many1, separated_list0, separated_list1};
use nom::sequence::{delimited, preceded, terminated};
use nom::{Err, IResult, Parser};
use num::Complex;
use tiny_skia::{BlendMode, FilterQuality, LineCap, LineJoin, SpreadMode};

const KEYWORDS: &[&str] = &["let", "if", "else", "match", "for", "loop"];

pub type SortMode = asdf_pixel_sort::Mode;

pub type SortDirection = asdf_pixel_sort::Direction;

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

type ImageThresholdType = imageproc::contrast::ThresholdType;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ThresholdType {
    Binary,
    BinaryInverted,
    Truncate,
    ToZero,
    ToZeroInverted,
}

impl Into<ImageThresholdType> for ThresholdType {
    fn into(self) -> ImageThresholdType {
        match self {
            ThresholdType::Binary => ImageThresholdType::Binary,
            ThresholdType::BinaryInverted => ImageThresholdType::BinaryInverted,
            ThresholdType::Truncate => ImageThresholdType::Truncate,
            ThresholdType::ToZero => ImageThresholdType::ToZero,
            ThresholdType::ToZeroInverted => ImageThresholdType::ToZeroInverted,
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
    Char(char),
    String(String),
    Shape(ShapeKind),
    BlendMode(BlendMode),
    LineCap(LineCap),
    LineJoin(LineJoin),
    SpreadMode(SpreadMode),
    FilterQuality(FilterQuality),
    FilterType(FilterType),
    ThresholdType(ThresholdType),
    Norm(Norm),
    SortMode(SortMode),
    SortDirection(SortDirection),
}

impl ToString for Literal {
    fn to_string(&self) -> String {
        match self {
            Literal::Integer(n) => n.to_string(),
            Literal::Float(n) => n.to_string(),
            Literal::Complex(n) => n.to_string(),
            Literal::Boolean(b) => b.to_string(),
            Literal::Hex([r, g, b]) => format!("0x{}{}{}", r, g, b),
            Literal::Char(c) => format!("'{}'", c),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Shape(kind) => kind.to_string(),
            Literal::BlendMode(bm) => match bm {
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
            Literal::LineCap(lc) => match lc {
                LineCap::Butt => "LINE_CAP_BUTT".into(),
                LineCap::Round => "LINE_CAP_ROUND".into(),
                LineCap::Square => "LINE_CAP_SQUARE".into(),
            },
            Literal::LineJoin(lj) => match lj {
                LineJoin::Miter => "LINE_JOIN_MITER".into(),
                LineJoin::MiterClip => "LINE_JOIN_MITER_CLIP".into(),
                LineJoin::Round => "LINE_JOIN_ROUND".into(),
                LineJoin::Bevel => "LINE_JOIN_BEVEL".into(),
            },
            Literal::SpreadMode(sm) => match sm {
                SpreadMode::Pad => "SPREAD_MODE_PAD".into(),
                SpreadMode::Reflect => "SPREAD_MODE_REFLECT".into(),
                SpreadMode::Repeat => "SPREAD_MODE_REPEAT".into(),
            },
            Literal::FilterQuality(fq) => match fq {
                FilterQuality::Nearest => "QUALITY_NEAREST".into(),
                FilterQuality::Bilinear => "QUALITY_BILINEAR".into(),
                FilterQuality::Bicubic => "QUALITY_BICUBIC".into(),
            },
            Literal::FilterType(ft) => match ft {
                FilterType::Nearest => "FILTER_NEAREST".into(),
                FilterType::Triangle => "FILTER_TRIANGLE".into(),
                FilterType::CatmullRom => "FILTER_CATMULL_ROM".into(),
                FilterType::Gaussian => "FILTER_GAUSSIAN".into(),
                FilterType::Lanczos3 => "FILTER_LANCZOS3".into(),
            },
            Literal::ThresholdType(tt) => match tt {
                ThresholdType::Binary => "THRESHOLD_BINARY".into(),
                ThresholdType::BinaryInverted => "THRESHOLD_BINARY_INVERTED".into(),
                ThresholdType::Truncate => "THRESHOLD_TRUNCATE".into(),
                ThresholdType::ToZero => "THRESHOLD_TO_ZERO".into(),
                ThresholdType::ToZeroInverted => "THRESHOLD_TO_ZERO_INVERTED".into(),
            },
            Literal::Norm(norm) => match norm {
                Norm::L1 => "NORM_L1".into(),
                Norm::L2 => "NORM_L2".into(),
                Norm::LInf => "NORM_LINF".into(),
            },
            Literal::SortMode(sm) => match sm {
                SortMode::Black(_) => "SORT_BLACK".into(),
                SortMode::Brightness(_) => "SORT_BRIGHTNESS".into(),
                SortMode::White(_) => "SORT_WHITE".into(),
            },
            Literal::SortDirection(sd) => match sd {
                SortDirection::Both => "DIRECTION_BOTH".into(),
                SortDirection::Column => "DIRECTION_COLUMN".into(),
                SortDirection::Row => "DIRECTION_ROW".into(),
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
    Match(Vec<(Pattern, bool, usize)>),
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
                recognize((digit1, char('%'))),
                recognize((opt(digit1), opt(char('.')), digit1, char('%'))),
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
        |s: &str| {
            if s.chars().last() == Some('%') {
                let substr = &s[0..s.len() - 1];
                substr
                    .parse::<f32>()
                    .map(|n| n / 100.0)
                    .or(format!("{}.0", substr).parse::<f32>().map(|n| n / 100.0))
            } else {
                s.parse()
            }
        },
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

fn hex_color(input: &str) -> IResult<&str, Literal> {
    map(color, Literal::Hex).parse(input)
}

fn character(input: &str) -> IResult<&str, Literal> {
    map(
        delimited(char('\''), none_of("'\\"), char('\'')),
        Literal::Char,
    )
    .parse(input)
}

fn string(input: &str) -> IResult<&str, Literal> {
    map(
        delimited(char('"'), many0(none_of("\"\\")), char('"')),
        |chars| Literal::String(chars.into_iter().collect()),
    )
    .parse(input)
}

fn shape(input: &str) -> IResult<&str, Literal> {
    map(
        alt((
            value(
                ShapeKind::Triangle,
                alt((tag("TRIANGLE"), tag("▲"), tag("△"), tag("▴"), tag("▵"))),
            ),
            value(
                ShapeKind::Square,
                alt((
                    tag("SQUARE"),
                    tag("⬜"),
                    tag("⬛"),
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

fn line_cap(input: &str) -> IResult<&str, Literal> {
    map(
        alt((
            value(LineCap::Butt, tag("LINE_CAP_BUTT")),
            value(LineCap::Round, tag("LINE_CAP_ROUND")),
            value(LineCap::Square, tag("LINE_CAP_SQUARE")),
        )),
        Literal::LineCap,
    )
    .parse(input)
}

fn line_join(input: &str) -> IResult<&str, Literal> {
    map(
        alt((
            value(LineJoin::MiterClip, tag("LINE_JOIN_MITER_CLIP")),
            value(LineJoin::Miter, tag("LINE_JOIN_MITER")),
            value(LineJoin::Round, tag("LINE_JOIN_ROUND")),
            value(LineJoin::Bevel, tag("LINE_JOIN_BEVEL")),
        )),
        Literal::LineJoin,
    )
    .parse(input)
}

fn spread_mode(input: &str) -> IResult<&str, Literal> {
    map(
        alt((
            value(SpreadMode::Pad, tag("SPREAD_MODE_PAD")),
            value(SpreadMode::Reflect, tag("SPREAD_MODE_REFLECT")),
            value(SpreadMode::Repeat, tag("SPREAD_MODE_REPEAT")),
        )),
        Literal::SpreadMode,
    )
    .parse(input)
}

fn filter_quality(input: &str) -> IResult<&str, Literal> {
    map(
        alt((
            value(FilterQuality::Nearest, tag("QUALITY_NEAREST")),
            value(FilterQuality::Bilinear, tag("QUALITY_BILINEAR")),
            value(FilterQuality::Bicubic, tag("QUALITY_BICUBIC")),
        )),
        Literal::FilterQuality,
    )
    .parse(input)
}

fn filter_type(input: &str) -> IResult<&str, Literal> {
    map(
        alt((
            value(FilterType::Nearest, tag("FILTER_NEAREST")),
            value(FilterType::Triangle, tag("FILTER_TRIANGLE")),
            value(FilterType::CatmullRom, tag("FILTER_CATMULL_ROM")),
            value(FilterType::Gaussian, tag("FILTER_GAUSSIAN")),
            value(FilterType::Lanczos3, tag("FILTER_LANCZOS3")),
        )),
        Literal::FilterType,
    )
    .parse(input)
}

fn threshold_type(input: &str) -> IResult<&str, Literal> {
    map(
        alt((
            value(
                ThresholdType::BinaryInverted,
                tag("THRESHOLD_BINARY_INVERTED"),
            ),
            value(ThresholdType::Binary, tag("THRESHOLD_BINARY")),
            value(ThresholdType::Truncate, tag("THRESHOLD_TRUNCATE")),
            value(
                ThresholdType::ToZeroInverted,
                tag("THRESHOLD_TO_ZERO_INVERTED"),
            ),
            value(ThresholdType::ToZero, tag("THRESHOLD_TO_ZERO")),
        )),
        Literal::ThresholdType,
    )
    .parse(input)
}

fn norm(input: &str) -> IResult<&str, Literal> {
    map(
        alt((
            value(Norm::L1, tag("NORM_L1")),
            value(Norm::L2, tag("NORM_L2")),
            value(Norm::LInf, tag("NORM_LINF")),
        )),
        Literal::Norm,
    )
    .parse(input)
}

fn sort_mode(input: &str) -> IResult<&str, Literal> {
    map(
        alt((
            value(SortMode::Black(DEFAULT_BLACK.clone()), tag("SORT_BLACK")),
            value(
                SortMode::Brightness(DEFAULT_BRIGHTNESS),
                tag("SORT_BRIGHTNESS"),
            ),
            value(SortMode::White(DEFAULT_WHITE.clone()), tag("SORT_WHITE")),
        )),
        Literal::SortMode,
    )
    .parse(input)
}

fn sort_direction(input: &str) -> IResult<&str, Literal> {
    map(
        alt((
            value(SortDirection::Both, tag("DIRECTION_BOTH")),
            value(SortDirection::Column, tag("DIRECTION_COLUMN")),
            value(SortDirection::Row, tag("DIRECTION_ROW")),
        )),
        Literal::SortDirection,
    )
    .parse(input)
}

fn literal(input: &str) -> IResult<&str, Literal> {
    alt((
        hex,
        complex,
        float,
        integer,
        boolean,
        shape,
        hex_color,
        character,
        string,
        blend_mode,
        line_cap,
        line_join,
        spread_mode,
        filter_quality,
        filter_type,
        threshold_type,
        norm,
        sort_mode,
        sort_direction,
    ))
    .parse(input)
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
    guard: Option<Block<'a>>,
}

fn pattern_block(indent: usize) -> impl FnMut(&str) -> IResult<&str, PatternBlock> {
    move |input| {
        let (input, pattern) = pattern(indent)(input)?;

        let (input, has_guard) = opt((space1, tag("if"))).parse(input)?;
        let (input, guard) = if has_guard.is_some() {
            let (input, guard) = preceded(space1, expr(indent + 1, true)).parse(input)?;
            (input, Some(guard))
        } else {
            (input, None)
        };

        let (input, _) =
            alt(((multispace0, tag("->")), (space0, peek(line_ending)))).parse(input)?;
        let (input, expr) = expr(indent + 1, true)(input)?;

        let pattern_block = PatternBlock {
            pattern,
            expr,
            guard,
        };
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
        let mut guard_blocks = Vec::new();
        let mut flattened_blocks = Vec::new();

        for pattern_block in pattern_blocks {
            skip += pattern_block.expr.len() + 1;
            patterns.push((
                pattern_block.pattern,
                pattern_block.guard.is_some(),
                pattern_block.expr.len() + 1,
            ));
            flattened_blocks.extend(pattern_block.expr);
            flattened_blocks.push(Token::Jump(total - skip));

            if let Some(mut guard) = pattern_block.guard {
                guard.reverse();
                guard_blocks.extend(guard);
            }
        }

        guard_blocks.reverse();

        let mut block =
            Vec::with_capacity(condition.len() + guard_blocks.len() + flattened_blocks.len() + 1);
        block.extend(guard_blocks);
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
    let (input, tree) = terminated(
        many0(preceded(many0((space0, line_ending)), definition)),
        (multispace0, eof),
    )
    .parse(input)
    .map_err(|_| crate::Error::ParseError)?;
    assert!(input.is_empty());
    Ok(tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::{BlendMode, LineCap, LineJoin, SpreadMode};

    // Helper function to simplify test assertions
    fn assert_parses<'a, P, O>(mut parser: P, input: &'a str, expected: O)
    where
        P: FnMut(&'a str) -> IResult<&'a str, O>,
        O: PartialEq + core::fmt::Debug,
    {
        assert_eq!(parser(input), Ok(("", expected)));
    }

    #[test]
    fn test_literals() {
        // Numeric literals
        assert_parses(literal, "42", Literal::Integer(42));
        assert_parses(literal, "3.14", Literal::Float(3.14));
        assert_parses(literal, "50%", Literal::Float(0.5));
        assert_parses(literal, "3+4i", Literal::Complex(Complex::new(3.0, 4.0)));
        assert_parses(literal, "-2.5i", Literal::Complex(Complex::new(0.0, -2.5)));

        // Boolean literals
        assert_parses(literal, "true", Literal::Boolean(true));
        assert_parses(literal, "false", Literal::Boolean(false));

        // Color literals
        assert_parses(literal, "0xff00cc", Literal::Hex([255, 0, 204]));
        assert_parses(literal, "0xf0c", Literal::Hex([255, 0, 204])); // shorthand

        // Shape literals
        assert_parses(literal, "TRIANGLE", Literal::Shape(ShapeKind::Triangle));
        assert_parses(literal, "▲", Literal::Shape(ShapeKind::Triangle));
        assert_parses(literal, "SQUARE", Literal::Shape(ShapeKind::Square));
        assert_parses(literal, "■", Literal::Shape(ShapeKind::Square));
        assert_parses(literal, "CIRCLE", Literal::Shape(ShapeKind::Circle));
        assert_parses(literal, "●", Literal::Shape(ShapeKind::Circle));
    }

    #[test]
    fn test_enum_literals() {
        // Blend modes
        let blend_modes = vec![
            ("BLEND_CLEAR", BlendMode::Clear),
            ("BLEND_SOURCE_OVER", BlendMode::SourceOver),
            ("BLEND_DESTINATION_OVER", BlendMode::DestinationOver),
            ("BLEND_SOURCE_IN", BlendMode::SourceIn),
            ("BLEND_DESTINATION_IN", BlendMode::DestinationIn),
            ("BLEND_SOURCE_OUT", BlendMode::SourceOut),
            ("BLEND_DESTINATION_OUT", BlendMode::DestinationOut),
            ("BLEND_SOURCE_ATOP", BlendMode::SourceAtop),
            ("BLEND_DESTINATION_ATOP", BlendMode::DestinationAtop),
            ("BLEND_SOURCE", BlendMode::Source),
            ("BLEND_DESTINATION", BlendMode::Destination),
            ("BLEND_XOR", BlendMode::Xor),
            ("BLEND_PLUS", BlendMode::Plus),
            ("BLEND_MODULATE", BlendMode::Modulate),
            ("BLEND_SCREEN", BlendMode::Screen),
            ("BLEND_OVERLAY", BlendMode::Overlay),
            ("BLEND_DARKEN", BlendMode::Darken),
            ("BLEND_LIGHTEN", BlendMode::Lighten),
            ("BLEND_COLOR_DODGE", BlendMode::ColorDodge),
            ("BLEND_COLOR_BURN", BlendMode::ColorBurn),
            ("BLEND_HARD_LIGHT", BlendMode::HardLight),
            ("BLEND_SOFT_LIGHT", BlendMode::SoftLight),
            ("BLEND_DIFFERENCE", BlendMode::Difference),
            ("BLEND_EXCLUSION", BlendMode::Exclusion),
            ("BLEND_MULTIPLY", BlendMode::Multiply),
            ("BLEND_HUE", BlendMode::Hue),
            ("BLEND_SATURATION", BlendMode::Saturation),
            ("BLEND_COLOR", BlendMode::Color),
            ("BLEND_LUMINOSITY", BlendMode::Luminosity),
        ];

        for (input, expected) in blend_modes {
            assert_parses(blend_mode, input, Literal::BlendMode(expected));
            assert_parses(literal, input, Literal::BlendMode(expected));
        }

        // Line caps
        let line_caps = vec![
            ("LINE_CAP_BUTT", LineCap::Butt),
            ("LINE_CAP_ROUND", LineCap::Round),
            ("LINE_CAP_SQUARE", LineCap::Square),
        ];

        for (input, expected) in line_caps {
            assert_parses(line_cap, input, Literal::LineCap(expected));
            assert_parses(literal, input, Literal::LineCap(expected));
        }

        // Line joins
        let line_joins = vec![
            ("LINE_JOIN_MITER_CLIP", LineJoin::MiterClip),
            ("LINE_JOIN_MITER", LineJoin::Miter),
            ("LINE_JOIN_ROUND", LineJoin::Round),
            ("LINE_JOIN_BEVEL", LineJoin::Bevel),
        ];

        for (input, expected) in line_joins {
            assert_parses(line_join, input, Literal::LineJoin(expected));
            assert_parses(literal, input, Literal::LineJoin(expected));
        }

        // Spread modes
        let spread_modes = vec![
            ("SPREAD_MODE_PAD", SpreadMode::Pad),
            ("SPREAD_MODE_REFLECT", SpreadMode::Reflect),
            ("SPREAD_MODE_REPEAT", SpreadMode::Repeat),
        ];

        for (input, expected) in spread_modes {
            assert_parses(spread_mode, input, Literal::SpreadMode(expected));
            assert_parses(literal, input, Literal::SpreadMode(expected));
        }

        // Filter qualities
        let filter_qualities = vec![
            ("QUALITY_NEAREST", FilterQuality::Nearest),
            ("QUALITY_BILINEAR", FilterQuality::Bilinear),
            ("QUALITY_BICUBIC", FilterQuality::Bicubic),
        ];

        for (input, expected) in filter_qualities {
            assert_parses(filter_quality, input, Literal::FilterQuality(expected));
            assert_parses(literal, input, Literal::FilterQuality(expected));
        }

        // Filter types
        let filter_types = vec![
            ("FILTER_NEAREST", FilterType::Nearest),
            ("FILTER_TRIANGLE", FilterType::Triangle),
            ("FILTER_CATMULL_ROM", FilterType::CatmullRom),
            ("FILTER_GAUSSIAN", FilterType::Gaussian),
            ("FILTER_LANCZOS3", FilterType::Lanczos3),
        ];

        for (input, expected) in filter_types {
            assert_parses(filter_type, input, Literal::FilterType(expected));
            assert_parses(literal, input, Literal::FilterType(expected));
        }

        // Sort modes
        let sort_modes = vec![
            ("SORT_BLACK", SortMode::Black(DEFAULT_BLACK.clone())),
            ("SORT_BRIGHTNESS", SortMode::Brightness(DEFAULT_BRIGHTNESS)),
            ("SORT_WHITE", SortMode::White(DEFAULT_WHITE.clone())),
        ];

        for (input, expected) in sort_modes {
            assert_parses(sort_mode, input, Literal::SortMode(expected.clone()));
            assert_parses(literal, input, Literal::SortMode(expected));
        }

        // Sort directions
        let sort_directions = vec![
            ("DIRECTION_BOTH", SortDirection::Both),
            ("DIRECTION_COLUMN", SortDirection::Column),
            ("DIRECTION_ROW", SortDirection::Row),
        ];

        for (input, expected) in sort_directions {
            assert_parses(
                sort_direction,
                input,
                Literal::SortDirection(expected.clone()),
            );
            assert_parses(literal, input, Literal::SortDirection(expected));
        }

        // Test that invalid enum values fail
        assert!(blend_mode("BLEND_INVALID").is_err());
        assert!(line_cap("LINE_CAP_INVALID").is_err());
        assert!(line_join("LINE_JOIN_INVALID").is_err());
        assert!(spread_mode("SPREAD_MODE_INVALID").is_err());
        assert!(filter_quality("QUALITY_INVALID").is_err());
        assert!(filter_type("FILTER_INVALID").is_err());
        assert!(sort_mode("SORT_INVALID").is_err());
        assert!(sort_direction("DIRECTION_INVALID").is_err());
    }

    #[test]
    fn test_operators() {
        // Unary operators
        assert_parses(unary_operator, "-", UnaryOperator::Negation);
        assert_parses(unary_operator, "!", UnaryOperator::Not);
        assert_parses(unary_operator, "~", UnaryOperator::BitNot);

        // Binary operators
        assert_parses(binary_operator, "+", BinaryOperator::Addition);
        assert_parses(binary_operator, "**", BinaryOperator::Exponentiation);
        assert_parses(binary_operator, "==", BinaryOperator::EqualTo);
        assert_parses(binary_operator, "..", BinaryOperator::RangeExclusive);
        assert_parses(binary_operator, "|>", BinaryOperator::Pipe);
    }

    #[test]
    fn test_expressions() {
        // Simple expressions
        assert_parses(
            expr(0, true),
            "3 + 4 * 2",
            vec![
                Token::Literal(Literal::Integer(3)),
                Token::Literal(Literal::Integer(4)),
                Token::Literal(Literal::Integer(2)),
                Token::BinaryOperator(BinaryOperator::Multiplication),
                Token::BinaryOperator(BinaryOperator::Addition),
            ],
        );

        // Parentheses
        assert_parses(
            expr(0, true),
            "(3 + 4) * 2",
            vec![
                Token::Literal(Literal::Integer(3)),
                Token::Literal(Literal::Integer(4)),
                Token::BinaryOperator(BinaryOperator::Addition),
                Token::Literal(Literal::Integer(2)),
                Token::BinaryOperator(BinaryOperator::Multiplication),
            ],
        );

        // Function calls
        assert_parses(
            expr(0, true),
            "f x y",
            vec![
                Token::Call("x", 0),
                Token::Call("y", 0),
                Token::Call("f", 2),
            ],
        );
    }

    #[test]
    fn test_control_structures() {
        // If-else
        assert_parses(
            if_statement(0),
            "if x > 0 -> 1 else -> -1",
            vec![
                Token::Call("x", 0),
                Token::Literal(Literal::Integer(0)),
                Token::BinaryOperator(BinaryOperator::GreaterThan),
                Token::If(2),
                Token::Literal(Literal::Integer(1)),
                Token::Jump(1),
                Token::Literal(Literal::Integer(-1)),
            ],
        );

        // Match
        assert_parses(
            match_statement(0),
            "match x -> 1 -> 10; 2 -> 20",
            vec![
                Token::Call("x", 0),
                Token::Match(vec![
                    (Pattern::Matches(vec![(Literal::Integer(1))]), false, 2),
                    (Pattern::Matches(vec![Literal::Integer(2)]), false, 2),
                ]),
                Token::Literal(Literal::Integer(10)),
                Token::Jump(2),
                Token::Literal(Literal::Integer(20)),
                Token::Jump(0),
            ],
        );

        // Match
        assert_parses(
            match_statement(0),
            "match x -> 1 -> 10; 2 if x == 2 -> 20",
            vec![
                Token::Call("x", 0),
                Token::Literal(Literal::Integer(2)),
                Token::BinaryOperator(BinaryOperator::EqualTo),
                Token::Call("x", 0),
                Token::Match(vec![
                    (Pattern::Matches(vec![(Literal::Integer(1))]), false, 2),
                    (Pattern::Matches(vec![Literal::Integer(2)]), true, 2),
                ]),
                Token::Literal(Literal::Integer(10)),
                Token::Jump(2),
                Token::Literal(Literal::Integer(20)),
                Token::Jump(0),
            ],
        );

        // For loop
        assert_parses(
            for_statement(0),
            "for i in 0..5 -> i * 2",
            vec![
                Token::Literal(Literal::Integer(0)),
                Token::Literal(Literal::Integer(5)),
                Token::BinaryOperator(BinaryOperator::RangeExclusive),
                Token::ForStart("i"),
                Token::Call("i", 0),
                Token::Literal(Literal::Integer(2)),
                Token::BinaryOperator(BinaryOperator::Multiplication),
                Token::Pop,
                Token::ForEnd,
            ],
        );
    }

    #[test]
    fn test_definitions() {
        // Simple definition
        assert_parses(
            definition,
            "square = SQUARE",
            Definition {
                name: "square",
                params: vec![],
                block: vec![Token::Literal(Literal::Shape(ShapeKind::Square))],
                weight: 1.0,
            },
        );

        // Parameterized definition
        assert_parses(
            definition,
            "repeat@2.5 n shape = loop n -> shape",
            Definition {
                name: "repeat",
                params: vec!["n", "shape"],
                block: vec![
                    Token::Call("n", 0),
                    Token::LoopStart,
                    Token::Call("shape", 0),
                    Token::LoopEnd,
                ],
                weight: 2.5,
            },
        );
    }

    #[test]
    fn test_full_parser() {
        let program = r#"
root =
    let shapes = [CIRCLE, SQUARE, TRIANGLE]
        for shape in shapes -> shape

double x = x * 2
        "#;

        let result = parse(program);
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].name, "root");
        assert_eq!(tree[1].name, "double");
    }

    #[test]
    fn test_error_cases() {
        // Invalid numeric literals
        assert!(float("abc").is_err());
        assert!(complex("3+").is_err());

        // Invalid identifiers
        assert!(identifier("123").is_err());
        assert!(identifier("let").is_err()); // keyword

        // Malformed expressions
        assert!(expr(0, true)("3 +").is_err());
        assert!(if_statement(0)("if x ->").is_err());

        // Incomplete definitions
        assert!(definition("foo =").is_err());
        assert!(definition("bar x").is_err());
    }

    #[test]
    fn test_edge_cases() {
        // Empty input
        assert_eq!(parse("").ok(), Some(vec![]));

        // Whitespace only
        assert_eq!(parse("  \n  \t  ").ok(), Some(vec![]));

        // Unicode identifiers
        assert_parses(identifier, "π", "π");
        assert_parses(identifier, "ℯ", "ℯ");
    }

    #[test]
    fn test_operator_precedence() {
        assert_parses(
            expr(0, true),
            "1 + 2 * 3",
            vec![
                Token::Literal(Literal::Integer(1)),
                Token::Literal(Literal::Integer(2)),
                Token::Literal(Literal::Integer(3)),
                Token::BinaryOperator(BinaryOperator::Multiplication),
                Token::BinaryOperator(BinaryOperator::Addition),
            ],
        );

        assert_parses(
            expr(0, true),
            "1 * 2 + 3",
            vec![
                Token::Literal(Literal::Integer(1)),
                Token::Literal(Literal::Integer(2)),
                Token::BinaryOperator(BinaryOperator::Multiplication),
                Token::Literal(Literal::Integer(3)),
                Token::BinaryOperator(BinaryOperator::Addition),
            ],
        );

        assert_parses(
            expr(0, true),
            "1 + 2 + 3",
            vec![
                Token::Literal(Literal::Integer(1)),
                Token::Literal(Literal::Integer(2)),
                Token::BinaryOperator(BinaryOperator::Addition),
                Token::Literal(Literal::Integer(3)),
                Token::BinaryOperator(BinaryOperator::Addition),
            ],
        );
    }

    #[test]
    fn test_list_expressions() {
        assert_parses(
            list,
            "[1, 2, 3]",
            vec![Token::List(vec![
                vec![Token::Literal(Literal::Integer(1))],
                vec![Token::Literal(Literal::Integer(2))],
                vec![Token::Literal(Literal::Integer(3))],
            ])],
        );

        assert_parses(
            list,
            "[SQUARE, CIRCLE, 0xff0000]",
            vec![Token::List(vec![
                vec![Token::Literal(Literal::Shape(ShapeKind::Square))],
                vec![Token::Literal(Literal::Shape(ShapeKind::Circle))],
                vec![Token::Literal(Literal::Hex([255, 0, 0]))],
            ])],
        );
    }
}
