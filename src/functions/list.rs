#[cfg(feature = "no-std")]
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::builtin_function;
use crate::error::{Error, Result};
use crate::interpreter::{Data, Value};

use itertools::Itertools;
use rand_chacha::ChaCha8Rng;

builtin_function!(range => {
    [Value::Integer(from), Value::Integer(to)] => {
        Value::List((*from..*to).map(Value::Integer).collect())
    },
    [Value::Float(from), Value::Float(to)] => {
        Value::List((*from as i32..*to as i32).map(|i| Value::Float(i as f32)).collect())
    },
    [Value::Char(from), Value::Char(to)] => {
        Value::String((*from..*to).collect())
    },
});

builtin_function!(rangei => {
    [Value::Integer(from), Value::Integer(to)] => {
        Value::List((*from..=*to).map(Value::Integer).collect())
    },
    [Value::Float(from), Value::Float(to)] => {
        Value::List((*from as i32..=*to as i32).map(|i| Value::Float(i as f32)).collect())
    },
    [Value::Char(from), Value::Char(to)] => {
        Value::String((*from..=*to).collect())
    },
});

builtin_function!(concat => {
    [Value::List(a), Value::List(b)] => {
        let mut list = Vec::with_capacity(a.len() + b.len());
        list.extend(a.clone());
        list.extend(b.clone());

        let value = Value::List(list);
        value.kind()?;
        value
    },
    [Value::String(a), Value::String(b)] => {
        let mut s = String::with_capacity(a.len() + b.len());
        s.push_str(a);
        s.push_str(b);
        Value::String(s)
    },
});

builtin_function!(prepend => {
    [a, Value::List(b)] => {
        let mut list = Vec::with_capacity(b.len() + 1);
        list.push(a.clone());
        list.extend(b.clone());

        let value = Value::List(list);
        value.kind()?;
        value
    },
    [Value::Char(a), Value::String(b)] => {
        let mut s = String::with_capacity(b.len() + 1);
        s.push(*a);
        s.push_str(b);
        Value::String(s)
    },
});

builtin_function!(append => {
    [Value::List(a), b] => {
        let mut list = Vec::with_capacity(a.len() + 1);
        list.extend(a.clone());
        list.push(b.clone());

        let value = Value::List(list);
        value.kind()?;
        value
    },
    [Value::String(a), Value::Char(b)] => {
        let mut s = String::with_capacity(a.len() + 1);
        s.push_str(a);
        s.push(*b);
        Value::String(s)
    },
});

builtin_function!(nth => {
    [Value::Integer(index), Value::List(list)] => {
        let len = list.len() as i32;
        let index = if *index >= 0 { *index } else { len + *index };
        if index < 0 || index >= len {
            return Err(Error::OutOfBounds);
        }
        list[index as usize].clone()
    },
    [Value::Integer(index), Value::String(s)] => {
        let len = s.chars().count() as i32;
        let index = if *index >= 0 { *index } else { len + *index };
        if index < 0 || index >= len {
            return Err(Error::OutOfBounds);
        }
        Value::Char(s.chars().nth(index as usize).unwrap())
    },
});

builtin_function!(set => {
    [Value::Integer(index), value, Value::List(list)] => {
        let len = list.len() as i32;
        let index = if *index >= 0 { *index } else { len + *index };
        if index < 0 || index >= len {
            return Err(Error::OutOfBounds);
        }
        let mut list = list.clone();
        list[index as usize] = value.clone();
        let list = Value::List(list);
        list.kind()?;
        list
    },
    [Value::Integer(index), Value::Char(c), Value::String(s)] => {
        let len = s.chars().count() as i32;
        let index = if *index >= 0 { *index } else { len + *index };
        if index < 0 || index >= len {
            return Err(Error::OutOfBounds);
        }
        let mut chars: Vec<_> = s.chars().collect();
        chars[index as usize] = *c;
        Value::String(chars.iter().collect())
    },
});

builtin_function!(length => {
    [Value::List(list)] => Value::Integer(list.len() as i32),
    [Value::String(s)] => Value::Integer(s.len() as i32),
});

builtin_function!(is_empty => {
    [Value::List(list)] => Value::Boolean(list.is_empty()),
    [Value::String(s)] => Value::Boolean(s.is_empty()),
});

builtin_function!(head => {
    [Value::List(list)] => {
        if list.is_empty() {
            return Err(Error::OutOfBounds);
        }
        list[0].clone()
    },
    [Value::String(s)] => {
        if s.is_empty() {
            return Err(Error::OutOfBounds);
        }
        Value::Char(s.chars().next().unwrap())
    },
});

builtin_function!(tail => {
    [Value::List(list)] => {
        if list.is_empty() {
            return Err(Error::OutOfBounds);
        }
        Value::List(list[1..].to_vec())
    },
    [Value::String(s)] => {
        if s.is_empty() {
            return Err(Error::OutOfBounds);
        }
        Value::String(s.chars().skip(1).collect())
    },
});

builtin_function!(init => {
    [Value::List(list)] => {
        if list.is_empty() {
            return Err(Error::OutOfBounds);
        }
        Value::List(list[..list.len()-1].to_vec())
    },
    [Value::String(s)] => {
        if s.is_empty() {
            return Err(Error::OutOfBounds);
        }
        Value::String(s.chars().take(s.len() - 1).collect())
    },
});

builtin_function!(last => {
    [Value::List(list)] => {
        if list.is_empty() {
            return Err(Error::OutOfBounds);
        }
        list[list.len()-1].clone()
    },
    [Value::String(s)] => {
        if s.is_empty() {
            return Err(Error::OutOfBounds);
        }
        Value::Char(s.chars().last().unwrap())
    },
});

builtin_function!(contains => {
    [value, Value::List(list)] => Value::Boolean(list.contains(value)),
    [Value::Char(c), Value::String(s)] => Value::Boolean(s.contains(*c)),
});

builtin_function!(take => {
    [Value::Integer(count), Value::List(list)] => {
        if *count >= 0 {
            Value::List(list.iter().take(*count as usize).cloned().collect())
        } else {
            Value::List(
                list.iter()
                    .skip(list.len().saturating_sub(*count as usize))
                    .cloned()
                    .collect(),
            )
        }
    },
    [Value::Integer(count), Value::String(s)] => {
        if *count >= 0 {
            Value::String(s.chars().take(*count as usize).collect())
        } else {
            Value::String(
                s.chars()
                    .skip(s.chars().count().saturating_sub(*count as usize))
                    .collect(),
            )
        }
    },
});

builtin_function!(drop => {
    [Value::Integer(count), Value::List(list)] => {
        if *count >= 0 {
            Value::List(list.iter().skip(*count as usize).cloned().collect())
        } else {
            Value::List(
                list.iter().take(list.len() - *count as usize)
                    .cloned()
                    .collect(),
            )
        }
    },
    [Value::Integer(count), Value::String(s)] => {
        if *count >= 0 {
            Value::String(s.chars().skip(*count as usize).collect())
        } else {
            Value::String(
                s.chars().take(s.chars().count() - *count as usize).collect(),
            )
        }
    },
});

builtin_function!(index_of => {
    [value, Value::List(list)] => {
        let i = list.iter().position(|v| v == value);
        match i {
            Some(i) => Value::Integer(i as i32),
            None => Value::Integer(-1),
        }
    },
    [Value::Char(c), Value::String(s)] => {
        let i = s.chars().position(|c2| c2 == *c);
        match i {
            Some(i) => Value::Integer(i as i32),
            None => Value::Integer(-1),
        }
    },
});

builtin_function!(reverse => {
    [Value::List(list)] => Value::List(list.iter().rev().cloned().collect()),
    [Value::String(s)] => Value::String(s.chars().rev().collect()),
});

builtin_function!(slice => {
    [Value::Integer(start), Value::Integer(end), Value::List(list)] => {
        let len = list.len() as i32;
        let start = if *start >= 0 { *start } else { len + *start };
        let end = if *end >= 0 { *end } else { len + *end };
        if start < 0 || start >= len || end < 0 || end >= len {
            return Err(Error::OutOfBounds);
        }
        if start > end {
            Value::List(Vec::new())
        } else {
            Value::List(list[start as usize..end as usize].to_vec())
        }
    },
    [Value::Integer(start), Value::Integer(end), Value::String(s)] => {
        let len = s.chars().count() as i32;
        let start = if *start >= 0 { *start } else { len + *start };
        let end = if *end >= 0 { *end } else { len + *end };
        if start < 0 || start >= len || end < 0 || end >= len {
            return Err(Error::OutOfBounds);
        }
        if start > end {
            Value::String(String::new())
        } else {
            Value::String(s.chars().skip(start as usize).take((end - start) as usize).collect())
        }
    },
});

builtin_function!(split => {
    [Value::Integer(index), Value::List(list)] => {
        let len = list.len() as i32;
        let index = if *index >= 0 { *index } else { len + *index };
        if index < 0 || index >= len {
            return Err(Error::OutOfBounds);
        }
        Value::List(vec![
            Value::List(list[..index as usize].to_vec()),
            Value::List(list[index as usize..].to_vec()),
        ])
    },
    [Value::Integer(index), Value::String(s)] => {
        let len = s.chars().count() as i32;
        let index = if *index >= 0 { *index } else { len + *index };
        if index < 0 || index >= len {
            return Err(Error::OutOfBounds);
        }
        Value::List(vec![
            Value::String(s.chars().take(index as usize).collect()),
            Value::String(s.chars().skip(index as usize).collect()),
        ])
    },
});

builtin_function!(unique => {
    [Value::List(list)] => {
        let mut seen = Vec::new();
        let mut result = Vec::new();
        for value in list {
            if !seen.contains(value) {
                seen.push(value.clone());
                result.push(value.clone());
            }
        }
        Value::List(result)
    },
    [Value::String(s)] => {
        #[cfg(feature = "std")]
        {
            Value::String(s.chars().unique().collect())
        }
        #[cfg(feature = "no-std")]
        {
            let mut seen = Vec::new();
            let mut result = String::new();
            for c in s.chars() {
                if !seen.contains(&c) {
                    seen.push(c);
                    result.push(c);
                }
            }
            Value::String(result)
        }
    },
});

builtin_function!(min_of => {
    [Value::List(list)] => {
        if list.is_empty() {
            return Err(Error::InvalidList);
        } else {
            match &list[0] {
                Value::Integer(_) => {
                    let sum = list
                        .iter()
                        .map(|v| {
                            if let Value::Integer(n) = v {
                                n
                            } else {
                                unreachable!()
                            }
                        })
                        .min()
                        .unwrap();
                    Value::Integer(*sum)
                }
                Value::Float(_) => {
                    let sum = list
                        .iter()
                        .map(|v| {
                            if let Value::Float(n) = v {
                                n
                            } else {
                                unreachable!()
                            }
                        })
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap();
                    Value::Float(*sum)
                }
                _ => return Err(Error::InvalidList),
            }
        }
    },
    [Value::String(s)] => {
        if s.is_empty() {
            return Err(Error::InvalidList);
        } else {
            Value::Char(s.chars().min().unwrap())
        }
    },
});

builtin_function!(max_of => {
    [Value::List(list)] => {
        if list.is_empty() {
            return Err(Error::InvalidList);
        } else {
            match &list[0] {
                Value::Integer(_) => {
                    let sum = list
                        .iter()
                        .map(|v| {
                            if let Value::Integer(n) = v {
                                n
                            } else {
                                unreachable!()
                            }
                        })
                        .max()
                        .unwrap();
                    Value::Integer(*sum)
                }
                Value::Float(_) => {
                    let sum = list
                        .iter()
                        .map(|v| {
                            if let Value::Float(n) = v {
                                n
                            } else {
                                unreachable!()
                            }
                        })
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap();
                    Value::Float(*sum)
                }
                _ => return Err(Error::InvalidList),
            }
        }
    },
    [Value::String(s)] => {
        if s.is_empty() {
            return Err(Error::InvalidList);
        } else {
            Value::Char(s.chars().max().unwrap())
        }
    },
});

builtin_function!(sum => {
    [Value::List(list)] => {
        if list.is_empty() {
            Value::Integer(0)
        } else {
            match &list[0] {
                Value::Integer(_) => {
                    let sum = list
                        .iter()
                        .map(|v| {
                            if let Value::Integer(n) = v {
                                n
                            } else {
                                unreachable!()
                            }
                        })
                        .sum();
                    Value::Integer(sum)
                }
                Value::Float(_) => {
                    let sum = list
                        .iter()
                        .map(|v| {
                            if let Value::Float(n) = v {
                                n
                            } else {
                                unreachable!()
                            }
                        })
                        .sum();
                    Value::Float(sum)
                }
                Value::Complex(_) => {
                    let sum = list
                        .iter()
                        .map(|v| {
                            if let Value::Complex(n) = v {
                                n
                            } else {
                                unreachable!()
                            }
                        })
                        .sum();
                    Value::Complex(sum)
                }
                _ => return Err(Error::InvalidList),
            }
        }
    }
});

builtin_function!(product => {
    [Value::List(list)] => {
        if list.is_empty() {
            Value::Integer(0)
        } else {
            match &list[0] {
                Value::Integer(_) => {
                    let product = list
                        .iter()
                        .map(|v| {
                            if let Value::Integer(n) = v {
                                n
                            } else {
                                unreachable!()
                            }
                        })
                        .product();
                    Value::Integer(product)
                }
                Value::Float(_) => {
                    let product = list
                        .iter()
                        .map(|v| {
                            if let Value::Float(n) = v {
                                n
                            } else {
                                unreachable!()
                            }
                        })
                        .product();
                    Value::Float(product)
                }
                Value::Complex(_) => {
                    let product = list
                        .iter()
                        .map(|v| {
                            if let Value::Complex(n) = v {
                                n
                            } else {
                                unreachable!()
                            }
                        })
                        .product();
                    Value::Complex(product)
                }
                _ => return Err(Error::InvalidList),
            }
        }
    }
});

builtin_function!(sort => {
    [Value::List(list)] => {
        if list.is_empty() {
            Value::List(Vec::new())
        } else {
            match &list[0] {
                Value::Integer(_) => {
                    let sorted = list
                        .iter()
                        .map(|v| {
                            if let Value::Integer(n) = v {
                                n
                            } else {
                                unreachable!()
                            }
                        })
                        .sorted()
                        .copied()
                        .map(Value::Integer)
                        .collect();
                    Value::List(sorted)
                }
                Value::Float(_) => {
                    let sorted = list
                        .iter()
                        .map(|v| {
                            if let Value::Float(n) = v {
                                n
                            } else {
                                unreachable!()
                            }
                        })
                        .sorted_by(|a, b| a.partial_cmp(b).unwrap())
                        .copied()
                        .map(Value::Float)
                        .collect();
                    Value::List(sorted)
                }
                _ => return Err(Error::InvalidList),
            }
        }
    },
    [Value::String(s)] => Value::String(s.chars().sorted().collect()),
});

builtin_function!(flatten => {
    [Value::List(list)] => {
        if list.is_empty() {
            Value::List(Vec::new())
        } else {
            match list[0] {
                Value::List(_) => {
                    let flattened = list
                        .iter()
                        .map(|v| {
                            if let Value::List(l) = v {
                                l
                            } else {
                                unreachable!()
                            }
                        })
                        .flatten()
                        .cloned()
                        .collect();
                    Value::List(flattened)
                }
                _ => return Err(Error::InvalidList),
            }
        }
    },
});

builtin_function!(join => {
    [separator, Value::List(list)] => {
        match (separator, list.get(0)) {
            (Value::Char(c), Some(Value::String(_))) => {
                Value::String(list.iter().map(|v| {
                    if let Value::String(s) = v {
                        s
                    } else {
                        unreachable!()
                    }
                }).join(&c.to_string()))
            },
            _ => {
                let mut result = Vec::new();

                for (i, inner) in list.iter().enumerate() {
                    if i > 0 {
                        result.push(separator.clone());
                    }
                    match inner {
                        Value::List(inner) => result.extend(inner.clone()),
                        _ => return Err(Error::InvalidList),
                    }
                }

                let list = Value::List(result);
                list.kind()?;
                list
            }
        }
    },
});

builtin_function!(intercalate => {
    [Value::List(separator), Value::List(list)] => {
        let mut result = Vec::new();

        for (i, inner) in list.iter().enumerate() {
            if i > 0 {
                result.extend(separator.clone());
            }
            match inner {
                Value::List(inner) => result.extend(inner.iter().cloned()),
                _ => return Err(Error::InvalidList),
            }
        }

        let list = Value::List(result);
        list.kind()?;
        list
    },
    [Value::String(separator), Value::List(list)] => {
        match list.get(0) {
            Some(Value::String(_)) => {
                Value::String(list.iter().map(|v| {
                    if let Value::String(s) = v {
                        s
                    } else {
                        unreachable!()
                    }
                }).join(separator))
            }
            _ => return Err(Error::InvalidList),
        }
    },
});

builtin_function!(intersperse => {
    [value, Value::List(list)] => Value::List(list.iter().intersperse(value).cloned().collect()),
    [Value::Char(c), Value::String(s)] => Value::String(s.chars().intersperse(*c).collect()),
});
