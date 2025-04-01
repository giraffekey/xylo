#[cfg(feature = "no-std")]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::error::Result;
use crate::parser::{parse, Definition, Literal, Pattern, Token};

use tiny_skia::BlendMode;

fn blend_mode_to_string(blend_mode: BlendMode) -> String {
    match blend_mode {
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
    }
}

fn literal_to_string(literal: &Literal) -> String {
    match literal {
        Literal::Integer(n) => n.to_string(),
        Literal::Float(n) => n.to_string(),
        Literal::Complex(n) => n.to_string(),
        Literal::Boolean(b) => b.to_string(),
        Literal::Hex([r, g, b]) => format!("0x{}{}{}", r, g, b),
        Literal::Shape(kind) => kind.to_string(),
        Literal::BlendMode(b) => blend_mode_to_string(*b),
        Literal::List(list) => format!(
            "[{}]",
            list.iter()
                .map(literal_to_string)
                .collect::<Vec<String>>()
                .join(",")
        ),
    }
}

fn block_to_string(block: &[Token]) -> String {
    let mut index = 0;
    let mut stack = vec![Vec::new()];
    let mut depth = 0;
    let mut lets = vec![false];
    let mut in_if = false;

    while index < block.len() {
        match &block[index] {
            Token::Literal(literal) => {
                stack[depth].push(literal_to_string(literal));
                index += 1;
            }
            Token::UnaryOperator(op) => {
                let a = stack[depth].pop().unwrap();
                stack[depth].push(format!("({}{})", op.as_str(), a));
                index += 1;
            }
            Token::BinaryOperator(op) => {
                let b = stack[depth].pop().unwrap();
                let a = stack[depth].pop().unwrap();
                stack[depth].push(format!("({}{}{})", a, op.as_str(), b));
                index += 1;
            }
            Token::Call(name, argc) => {
                let mut args = Vec::new();
                for _ in 0..*argc {
                    args.push(stack[depth].pop().unwrap());
                }
                args.reverse();

                if args.is_empty() {
                    stack[depth].push(name.to_string());
                } else {
                    stack[depth].push(format!("({}{})", name, format!(" {}", args.join(" "))));
                }
                index += 1;
            }
            Token::Let(name, params, skip) => {
                if !lets[depth] {
                    depth += 1;
                    stack.push(Vec::new());
                    lets.push(true);
                }

                stack[depth].push(format!(
                    "{}{}={}",
                    name,
                    if params.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", params.join(" "))
                    },
                    block_to_string(&block[index + 1..index + skip])
                ));
                lets[depth] = true;
                index += skip + 1;
            }
            Token::Pop => {
                if lets[depth] {
                    let defs = stack[depth][..stack[depth].len() - 1].join(";");
                    let block = stack[depth].last().unwrap().clone();
                    stack[depth - 1].push(format!("let {}->{}", defs, block));

                    stack.pop().unwrap();
                    lets.pop().unwrap();
                    depth -= 1;
                }
                index += 1;
            }
            Token::If(skip) => {
                stack[depth].push(block_to_string(&block[index + 1..index + skip]));
                in_if = true;
                index += skip;
            }
            Token::Jump(skip) => {
                if in_if {
                    let else_block = block_to_string(&block[index + 1..index + skip + 1]);
                    let if_block = stack[depth].pop().unwrap();
                    let condition = stack[depth].pop().unwrap();
                    stack[depth].push(format!(
                        "if {}->{};else->{}",
                        condition, if_block, else_block
                    ));
                    index += skip + 1;
                    in_if = false;
                } else {
                    index += 1;
                }
            }
            Token::Match(patterns) => {
                let a = stack[depth].pop().unwrap();
                index += 1;

                let mut pats = Vec::new();
                for (pattern, skip) in patterns {
                    match pattern {
                        Pattern::Matches(matches) => {
                            let matches = matches
                                .iter()
                                .map(literal_to_string)
                                .collect::<Vec<String>>()
                                .join(",");
                            pats.push(format!(
                                "{}->{}",
                                matches,
                                block_to_string(&block[index..index + skip])
                            ));
                        }
                        Pattern::Wildcard => {
                            pats.push(format!(
                                "_->{}",
                                block_to_string(&block[index..index + skip])
                            ));
                        }
                    }
                    index += skip;
                }

                stack[depth].push(format!("match {}->{}", a, pats.join(";")));
            }
            Token::ForStart(var) => {
                let iter = stack[depth].pop().unwrap();
                depth += 1;
                stack.push(vec![format!("for {} in {}->", var, iter)]);
                lets.push(false);
                index += 1;
            }
            Token::ForEnd => {
                let block = stack[depth].pop().unwrap();
                let def = stack[depth].pop().unwrap();
                stack[depth - 1].push([def, block].concat());

                stack.pop().unwrap();
                lets.pop().unwrap();
                depth -= 1;
                index += 1;
            }
            Token::LoopStart => {
                let count = stack[depth].pop().unwrap();
                depth += 1;
                stack.push(vec![format!("loop {}->", count)]);
                lets.push(false);
                index += 1;
            }
            Token::LoopEnd => {
                let block = stack[depth].pop().unwrap();
                let def = stack[depth].pop().unwrap();
                stack[depth - 1].push([def, block].concat());

                stack.pop().unwrap();
                lets.pop().unwrap();
                depth -= 1;
                index += 1;
            }
            Token::Return(_) => index += 1,
        }
    }

    assert!(stack.len() == 1);
    stack[0].pop().unwrap()
}

fn definition_to_string(definition: &Definition) -> String {
    if definition.weight == 1.0 {
        format!(
            "{}{}={}",
            definition.name,
            if definition.params.is_empty() {
                String::new()
            } else {
                format!(" {}", definition.params.join(" "))
            },
            block_to_string(&definition.block)
        )
    } else {
        format!(
            "{}@{}{}={}",
            definition.name,
            definition.weight,
            if definition.params.is_empty() {
                String::new()
            } else {
                format!(" {}", definition.params.join(" "))
            },
            block_to_string(&definition.block)
        )
    }
}

pub fn minify(input: &str) -> Result<String> {
    let tree = parse(input)?;
    let output = tree
        .iter()
        .map(|definition| definition_to_string(definition))
        .collect::<Vec<String>>()
        .join("\n");
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::execute;

    fn can_execute(output: &str) -> bool {
        parse(output)
            .and_then(|tree| execute(tree, Some([0; 32])))
            .is_ok()
    }

    #[test]
    fn test_binary_operation() {
        let output = minify(
            "
root = square

square = ss (3 + 5) SQUARE
            ",
        );
        assert!(output.is_ok());
        assert_eq!(
            output.as_ref().unwrap(),
            "\
root=square
square=(ss (3+5) SQUARE)\
                "
        );
        assert!(can_execute(output.as_ref().unwrap()));
    }

    #[test]
    fn test_let_statement() {
        let output = minify(
            "
root = square

square =
    let n1 = 3
        n2 = 5
    ->
        let n3 = 1
            tx (n1 + n2 + n3) SQUARE
            ",
        );
        assert!(output.is_ok());
        assert_eq!(
            output.as_ref().unwrap(),
            "\
root=square
square=let n1=3;n2=5;n3=1->(tx ((n1+n2)+n3) SQUARE)\
                "
        );
        assert!(can_execute(output.as_ref().unwrap()));
    }

    #[test]
    fn test_if_statement() {
        let output = minify(
            "
root = shape true

shape is_square =
    if is_square
        SQUARE
    else
        CIRCLE
            ",
        );
        assert!(output.is_ok());
        assert_eq!(
            output.as_ref().unwrap(),
            "\
root=(shape true)
shape is_square=if is_square->SQUARE;else->CIRCLE\
            "
        );
        assert!(can_execute(output.as_ref().unwrap()));
    }

    #[test]
    fn test_match_statement() {
        let output = minify(
            "
root = shape 1

shape n =
    match n
        1 -> SQUARE
        2 -> CIRCLE
        3 -> TRIANGLE
        _ -> EMPTY
            ",
        );
        assert!(output.is_ok());
        assert_eq!(
            output.as_ref().unwrap(),
            "\
root=(shape 1)
shape n=match n->1->SQUARE;2->CIRCLE;3->TRIANGLE;_->EMPTY\
            "
        );
        assert!(can_execute(output.as_ref().unwrap()));
    }

    #[test]
    fn test_for_statement() {
        let output = minify(
            "
root = collect squares

squares =
    for i in 0..3
        tx i SQUARE
            ",
        );
        assert!(output.is_ok());
        assert_eq!(
            output.as_ref().unwrap(),
            "\
root=(collect squares)
squares=for i in (0..3)->(tx i SQUARE)\
            "
        );
        assert!(can_execute(output.as_ref().unwrap()));
    }

    #[test]
    fn test_loop_statement() {
        let output = minify(
            "
root = collect squares

squares =
    loop 3
        tx (rand * 10) SQUARE
            ",
        );
        assert!(output.is_ok());
        assert_eq!(
            output.as_ref().unwrap(),
            "\
root=(collect squares)
squares=loop 3->(tx (rand*10) SQUARE)\
            "
        );
        assert!(can_execute(output.as_ref().unwrap()));
    }
}
