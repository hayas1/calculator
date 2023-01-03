use itertools::Itertools;
use std::ops::{Add, Div, Mul, Neg, Sub};

pub fn calculate<N>(target: &str) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
{
    parenthetic::<N, _>(&mut target.chars().filter(|c| !c.is_whitespace()).peekable(), None)
}

fn parenthetic<N, E>(yet: &mut std::iter::Peekable<E>, open: Option<char>) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    // TODO better error message
    if open.is_some() {
        let p = yet.next();
        anyhow::ensure!(p == open, "expect {:?}, but found {:?}", open, p);
    }

    let closing_paren = |p| (p as u8 + 1) as char;
    let close = open.map(closing_paren);

    let result = expression(yet)?;
    match yet.next() {
        q @ (None | Some(')' | '}' | ']')) if q == close => Ok(result),
        c => anyhow::bail!("expect end of parenthetic expression, but found {:?}", c),
    }
}

fn expression<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    let mut result;
    if let Some('+' | '-') = yet.peek() {
        result = unop(yet.next().expect("peeked"), term(yet)?);
    } else {
        result = term(yet);
    }

    while let Some('+' | '-') = yet.peek() {
        result = binop(result?, yet.next().expect("peeked"), term(yet)?);
    }

    match yet.peek() {
        None | Some(')' | '}' | ']') => Ok(result?),
        c => anyhow::bail!("expect end of expression, but found {:?}", c),
    }
}

fn term<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    let mut result = factor::<N, _>(yet);
    while let Some('*' | '/') = yet.peek() {
        result = binop(result?, yet.next().expect("peeked"), factor(yet)?);
    }

    match yet.peek() {
        None | Some('+' | '-') | Some(')' | '}' | ']') => Ok(result?),
        c => anyhow::bail!("expect end of term, but found {:?}", c),
    }
}

fn factor<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    match yet.peek().ok_or_else(|| anyhow::anyhow!("expect factor, but found EOF"))? {
        &p @ ('(' | '{' | '[') => parenthetic(yet, Some(p)),
        n if n == &'.' || n.is_numeric() => constant(yet),
        c => Err(anyhow::anyhow!("expect factor, but found {}", c)),
    }
}

fn constant<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: std::str::FromStr,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    let mut result: String = yet.peeking_take_while(|d| d.is_numeric()).collect();
    if let Some('.') = yet.peek() {
        result.push_str(&yet.peeking_take_while(|d| d == &'.' || d.is_numeric()).collect::<String>())
    }
    Ok(N::from_str(&result)?)
}

fn binop<N>(a: N, op: char, b: N) -> anyhow::Result<N>
where
    N: Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
{
    match op {
        '+' => Ok(a + b),
        '-' => Ok(a - b),
        '*' => Ok(a * b),
        '/' => Ok(a / b),
        _ => anyhow::bail!("unimplemented binary operator: {}", op),
    }
}

fn unop<N>(op: char, a: N) -> anyhow::Result<N>
where
    N: Neg<Output = N>,
{
    match op {
        '+' => Ok(a),
        '-' => Ok(-a),
        _ => anyhow::bail!("unimplemented unary operator: {}", op),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert!(matches!(calculate::<i64>("1+2+3+4+5+6+7+8+9+10"), Ok(55)));
        assert!(matches!(calculate::<i64>("1+2++3"), Err(_)));
        assert!(matches!(calculate::<i64>("+2"), Ok(2)));
        assert!(matches!(calculate::<i64>("-2"), Ok(-2)));
    }

    #[test]
    fn test_paren() {
        assert!(matches!(calculate::<i64>("(1+2)+3"), Ok(6)));
        assert!(matches!(calculate::<i64>("(1+2))+3"), Err(_)));
        assert!(matches!(calculate::<i64>("(1+2)+3))))"), Err(_)));
        assert!(matches!(calculate::<i64>("((((1+2))))+(3)"), Ok(6)));
    }

    #[test]
    fn test_whitespace() {
        assert!(matches!(calculate::<i64>(" 1 +  2+3+4 +5+6+7+8+9+10  "), Ok(55)));
        assert!(matches!(calculate::<i64>("1+( 2+3)"), Ok(6)));
        assert!(matches!(calculate::<i64>("  1 + (2\t + 3\n\n  )"), Ok(6)));
    }

    #[test]
    fn test_mul() {
        assert!(matches!(calculate::<i64>("1*2*3*4*5*6*7*8*9*10"), Ok(3628800)));
        assert!(matches!(calculate::<i64>("1*2*3*4*5*"), Err(_)));
        assert!(matches!(calculate::<i64>("*2"), Err(_)));
    }

    #[test]
    fn test_integer_expression() {
        assert!(matches!(calculate::<i64>("(1+2*3)*4+5*6"), Ok(58)));
        assert!(matches!(calculate::<i64>("1+(2+3*4+5)*6"), Ok(115)));
        assert!(matches!(calculate::<i64>("1 - 2 + 3*4/5*6"), Ok(11)));
        assert!(matches!(calculate::<i64>("(2023 + 2024 + 2025) / 2024"), Ok(3)));
        assert!(matches!(calculate::<i64>("6*5*4/(3*2*1)"), Ok(20)));
    }

    #[test]
    fn test_fraction_expression() {
        assert_eq!(calculate::<f64>("1.23 * 4 ").unwrap(), 4.92);
        assert_eq!(calculate::<f64>("1.23 * (3 + 10.2) - 0.006 ").unwrap(), 16.23);
        assert_eq!(calculate::<f64>("1 - 2 + 3*4/5*6").unwrap(), 13.399999999999999); // TODO 13.4
        assert_eq!(calculate::<f64>("(20.23 + 20.24 + 20.25) / 20.24").unwrap(), 3.);
        assert_eq!(calculate::<f64>("1.").unwrap(), 1.0);
        assert_eq!(calculate::<f64>(".1").unwrap(), 0.1);
    }

    #[test]
    fn test_expression() {
        assert!(matches!(calculate::<i64>("-123 + (-45  / 9)"), Ok(-128)));
        assert!(matches!(calculate::<i64>("-123 + (-45  / -9)"), Err(_)));
        assert!(matches!(calculate::<i64>("-123 + -----9"), Err(_)));
        assert!(matches!(calculate::<i64>("-123 + (-45  / (-9))"), Ok(-118)));

        assert_eq!(calculate::<f64>("-1.23 * 4 ").unwrap(), -4.92);
        assert!(calculate::<f64>("-1.23 * -4 ").is_err());
        assert!(calculate::<f64>("-1.23 * -----4 ").is_err());
        assert_eq!(calculate::<f64>("-1.23 * (-4) ").unwrap(), 4.92);
    }

    #[test]
    fn test_rational() {
        assert_eq!(calculate::<num::Rational64>("-123 + (-45  / 9)").unwrap(), num::Rational64::from_integer(-128));
        assert_eq!(calculate::<num::Rational64>("1/99*3*3*11+100").unwrap(), num::Rational64::from_integer(101));
        assert_eq!(calculate::<num::Rational64>("2/3*5/4").unwrap(), num::Rational64::new_raw(5, 6));
        assert_eq!(calculate::<num::Rational64>("(1/3+1/2-1)*12").unwrap(), num::Rational64::from_integer(-2));
        assert_eq!(
            calculate::<num::Rational64>("2/3 + 1/6").unwrap(),
            <num::Rational64 as std::str::FromStr>::from_str("5/6").unwrap()
        );
    }
}
