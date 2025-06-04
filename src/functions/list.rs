#[cfg(feature = "no-std")]
use alloc::{vec, vec::Vec};

use crate::builtin_function;
use crate::error::{Error, Result};
use crate::interpreter::Value;

use itertools::Itertools;
use noise::Perlin;
use rand_chacha::ChaCha8Rng;

builtin_function!(range => {
    [Value::Integer(from), Value::Integer(to)] => {
        Value::List((*from..*to).map(Value::Integer).collect())
    },
    [Value::Float(from), Value::Float(to)] => {
        Value::List((*from as i32..*to as i32).map(|i| Value::Float(i as f32)).collect())
    }
});

builtin_function!(rangei => {
    [Value::Integer(from), Value::Integer(to)] => {
        Value::List((*from..=*to).map(Value::Integer).collect())
    },
    [Value::Float(from), Value::Float(to)] => {
        Value::List((*from as i32..=*to as i32).map(|i| Value::Float(i as f32)).collect())
    }
});

builtin_function!(concat => {
    [Value::List(a), Value::List(b)] => {
        let mut list = Vec::with_capacity(a.len() + b.len());
        list.extend(a.clone());
        list.extend(b.clone());

        let value = Value::List(list);
        value.kind()?;
        value
    }
});

builtin_function!(prepend => {
    [a, Value::List(b)] => {
        let mut list = Vec::with_capacity(b.len() + 1);
        list.push(a.clone());
        list.extend(b.clone());

        let value = Value::List(list);
        value.kind()?;
        value
    }
});

builtin_function!(append => {
    [Value::List(a), b] => {
        let mut list = Vec::with_capacity(a.len() + 1);
        list.extend(a.clone());
        list.push(b.clone());

        let value = Value::List(list);
        value.kind()?;
        value
    }
});

builtin_function!(nth => {
    [Value::Integer(index), Value::List(list)] => {
        let len = list.len() as i32;
        let index = if *index >= 0 { *index } else { len + *index };
        if index < 0 || index >= len {
            return Err(Error::OutOfBounds);
        }
        list.get(index as usize).unwrap().clone()
    }
});

builtin_function!(set => {
    [Value::Integer(index), value, Value::List(list)] => {
        let len = list.len() as i32;
        let i = if *index >= 0 { *index } else { len + *index };
        if i < 0 || i >= len {
            return Err(Error::OutOfBounds);
        }
        let mut list = list.clone();
        list[i as usize] = value.clone();
        let list = Value::List(list);
        list.kind()?;
        list
    }
});

builtin_function!(length => {
    [Value::List(list)] => Value::Integer(list.len() as i32)
});

builtin_function!(is_empty => {
    [Value::List(list)] => Value::Boolean(list.is_empty())
});

builtin_function!(head => {
    [Value::List(list)] => {
        if list.is_empty() {
            return Err(Error::OutOfBounds);
        }
        list[0].clone()
    }
});

builtin_function!(tail => {
    [Value::List(list)] => {
        if list.is_empty() {
            return Err(Error::OutOfBounds);
        }
        Value::List(list[1..].to_vec())
    }
});

builtin_function!(init => {
    [Value::List(list)] => {
        if list.is_empty() {
            return Err(Error::OutOfBounds);
        }
        Value::List(list[..list.len()-1].to_vec())
    }
});

builtin_function!(last => {
    [Value::List(list)] => {
        if list.is_empty() {
            return Err(Error::OutOfBounds);
        }
        list[list.len()-1].clone()
    }
});

builtin_function!(contains => {
    [value, Value::List(list)] => Value::Boolean(list.contains(value))
});

builtin_function!(take => {
    [Value::Integer(count), Value::List(list)] => {
        if *count >= 0 {
            Value::List(list.iter().take(*count as usize).cloned().collect())
        } else {
            Value::List(
                list.iter()
                    .rev()
                    .take(*count as usize)
                    .rev()
                    .cloned()
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
                list.iter()
                    .rev()
                    .skip(*count as usize)
                    .rev()
                    .cloned()
                    .collect(),
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
    }
});

builtin_function!(reverse => {
    [Value::List(list)] => Value::List(list.iter().rev().cloned().collect::<Vec<_>>()),
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
    }
});

builtin_function!(min_of => {
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
    }
});

builtin_function!(max_of => {
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
    }
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
    }
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

builtin_function!(intersperse => {
    [value, Value::List(list)] => Value::List(list.iter().intersperse(value).cloned().collect()),
});
