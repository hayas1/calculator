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
    Ok(expression::<N, _>(&mut target.chars(), None).unwrap()) // TODO
}

fn expression<N, E>(yet: &mut E, paren: Option<char>) -> anyhow::Result<N>
where
    N: Clone + FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char> + itertools::PeekingNext,
{
    let _skip_whitespace = yet.peeking_take_while(|c| c.is_whitespace()).count();
    let t = term::<N, _>(yet);
    match yet.next() {
        Some(op @ ('+' | '-')) => operate(t, op, expression(yet, paren)),
        None => t,
        op => todo!("{:?}", op),
    }
}

fn term<N, E>(yet: &mut E) -> anyhow::Result<N>
where
    N: Clone + FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char> + itertools::PeekingNext,
{
    let _skip_whitespace = yet.peeking_take_while(|c| c.is_whitespace()).count();
    let f = factor::<N, _>(yet);
    f
}

fn factor<N, E>(yet: &mut E) -> anyhow::Result<N>
where
    N: Clone + FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char> + itertools::PeekingNext,
{
    let _skip_whitespace = yet.peeking_take_while(|c| c.is_whitespace()).count();
    match yet.next().ok_or_else(|| anyhow::anyhow!("expect factor, but found EOF"))? {
        p @ ('(' | '{' | '[') => expression(yet, Some((p as u8 + 1) as char)),
        n if n.is_numeric() => constant(yet, n),
        c => Err(anyhow::anyhow!("expect factor, but found {}", c)),
    }
}

fn constant<N, E>(yet: &mut E, n: char) -> anyhow::Result<N>
where
    N: Clone + FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char> + itertools::PeekingNext,
{
    let _skip_whitespace = yet.peeking_take_while(|c| c.is_whitespace()).count();
    let integer: String = yet.peeking_take_while(|&d| d.is_numeric()).collect();
    // TODO fraction
    Ok(N::from_str(&format!("{}{}", n, integer))?)
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
