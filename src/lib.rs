use itertools::Itertools;
use std::{
    ops::{Add, Div, Mul, Sub},
    str::FromStr,
};

pub fn calculate<N>(target: &str) -> anyhow::Result<N>
where
    N: Clone + FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
{
    Ok(expression::<N, _>(&mut target.chars().peekable()).unwrap())
}

fn expression<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: Clone + FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char> + itertools::PeekingNext,
{
    let _skip_whitespace = yet.peeking_take_while(|c| c.is_whitespace()).count();
    let closing_paren = |p| (p as u8 + 1) as char;
    let q = if let Some('(' | '{' | '[') = yet.peek() { Some(closing_paren(yet.next().unwrap())) } else { None };
    dbg!(q);

    let _skip_whitespace = yet.peeking_take_while(|c| c.is_whitespace()).count();
    let mut result = term::<N, _>(yet);

    let _skip_whitespace = yet.peeking_take_while(|c| c.is_whitespace()).count();
    while let Some('+' | '-') = yet.peek() {
        result = operate(result, yet.next().expect("peeked"), expression(yet));
    }
    match yet.peek() {
        p @ (None | Some(')' | '}' | ']')) if p == q.as_ref() => yet.next(),
        op => todo!("operator {:?}, q {:?}", op, q),
    };
    result
}

fn term<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: Clone + FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char> + itertools::PeekingNext,
{
    let _skip_whitespace = yet.peeking_take_while(|c| c.is_whitespace()).count();
    let f = factor::<N, _>(yet);
    let _skip_whitespace = yet.peeking_take_while(|c| c.is_whitespace()).count();
    f
}

fn factor<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: Clone + FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char> + itertools::PeekingNext,
{
    let _skip_whitespace = yet.peeking_take_while(|c| c.is_whitespace()).count();
    match yet.peek().ok_or_else(|| anyhow::anyhow!("expect factor, but found EOF"))? {
        '(' | '{' | '[' => expression(yet),
        &n if n.is_numeric() => constant(yet),
        c => Err(anyhow::anyhow!("expect factor, but found {}", c)),
    }
}

fn constant<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: Clone + FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char> + itertools::PeekingNext,
{
    let _skip_whitespace = yet.peeking_take_while(|c| c.is_whitespace()).count();
    let integer: String = yet.peeking_take_while(|&d| d.is_numeric()).collect();
    // TODO fraction
    Ok(N::from_str(&integer)?)
}

fn operate<N>(a: anyhow::Result<N>, op: char, b: anyhow::Result<N>) -> anyhow::Result<N>
where
    N: Clone + FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
{
    match op {
        '+' => Ok(a? + b?),
        '-' => Ok(a? - b?),
        _ => anyhow::bail!("unimplemented operator: {}", op),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert!(matches!(calculate::<i64>("1+2+3+4+5+6+7+8+9+10"), Ok(55)));
    }

    #[test]
    fn test_paren() {
        assert!(matches!(calculate::<i64>("(1+2)+3"), Ok(6)));
    }
}
