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
    parenthetic::<N, _>(&mut target.chars().filter(|c| !c.is_whitespace()).peekable(), None)
}

fn parenthetic<N, E>(yet: &mut std::iter::Peekable<E>, p: Option<char>) -> anyhow::Result<N>
where
    N: Clone + FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    match (yet.peek(), p) {
        // FIXME better match (peek and next is difficult)
        (Some(&paren), Some(pp)) if paren != pp => anyhow::bail!("expect {}, but found {}", pp, paren),
        (None, Some(pp)) => anyhow::bail!("expect {}, but found EOF", pp),
        (paren, pp) if paren == pp.as_ref() => yet.next(),
        _ => None,
    };

    let closing_paren = |p| (p as u8 + 1) as char;
    let qq = p.map(closing_paren);

    let result = expression(yet);

    match yet.next() {
        q @ (None | Some(')' | '}' | ']')) if q == qq => result,
        c => anyhow::bail!("expect end of expression, but found {:?}", c),
    }
}

fn expression<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: Clone + FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    let mut result = term::<N, _>(yet);

    while let Some('+' | '-') = yet.peek() {
        result = operate(result, yet.next().expect("peeked"), expression(yet));
    }
    match yet.peek() {
        None | Some(')' | '}' | ']') => result,
        c => anyhow::bail!("expect end of expression, but found {:?}", c),
    }
}

fn term<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: Clone + FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    let f = factor::<N, _>(yet);
    f
}

fn factor<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: Clone + FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    match yet.peek().ok_or_else(|| anyhow::anyhow!("expect factor, but found EOF"))? {
        &p @ ('(' | '{' | '[') => parenthetic(yet, Some(p)),
        n if n.is_numeric() => constant(yet),
        c => Err(anyhow::anyhow!("expect factor, but found {}", c)),
    }
}

fn constant<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: Clone + FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
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
        assert!(matches!(calculate::<i64>("1+2++3"), Err(_)));
        assert!(matches!(calculate::<i64>("+2"), Err(_))); // TODO should be Ok?
    }

    #[test]
    fn test_paren() {
        assert!(matches!(calculate::<i64>("(1+2)+3"), Ok(6)));
        assert!(matches!(calculate::<i64>("(1+2))+3"), Err(_)));
    }

    #[test]
    fn test_whitespace() {
        assert!(matches!(calculate::<i64>(" 1 +  2+3+4 +5+6+7+8+9+10  "), Ok(55)));
        assert!(matches!(calculate::<i64>("1+( 2+3)"), Ok(6)));
        assert!(matches!(calculate::<i64>("  1 + (2\t + 3\n\n  )"), Ok(6)));
        assert!(matches!(calculate::<i64>("+2"), Err(_))); // TODO should be Ok?
    }
}
