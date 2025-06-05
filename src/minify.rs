#[cfg(feature = "no-std")]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::error::Result;
use crate::parser::{parse, Definition, Literal, Pattern, Token};

fn block_to_string(block: &[Token]) -> String {
    let mut index = 0;
    let mut stack = vec![Vec::new()];
    let mut depth = 0;
    let mut lets = vec![false];
    let mut in_if = false;

    while index < block.len() {
        match &block[index] {
            Token::Literal(literal) => {
                stack[depth].push(literal.to_string());
                index += 1;
            }
            Token::List(blocks) => {
                stack[depth].push(format!(
                    "[{}]",
                    blocks
                        .iter()
                        .map(|block| block_to_string(block))
                        .collect::<Vec<_>>()
                        .join(",")
                ));
                index += 1;
            }
            Token::UnaryOperator(op) => {
                let a = stack[depth].pop().unwrap();
                stack[depth].push(format!("{}({})", op.as_str(), a));
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
                                .map(Literal::to_string)
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
    use crate::out::Config;

    fn can_execute(output: &str) -> bool {
        let config = Config {
            dimensions: (400, 400),
            seed: Some([0; 32]),
        };
        parse(output).and_then(|tree| execute(tree, config)).is_ok()
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
