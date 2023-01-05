use anyhow::Context as _;
use itertools::Itertools;
use std::ops::{Add, Div, Mul, Neg, Sub};

pub fn calculate<N>(target: &str) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
{
    let mut reader = target.chars().filter(|c| !c.is_whitespace()).peekable();
    parenthetic(&mut reader, None)
        .with_context(|| anyhow::anyhow!("fail calculate {:?}, remaining {:?}", target, reader.collect::<String>()))
}

/// parenthetic =  expression ; | '(' expression ')'
fn parenthetic<N, E>(yet: &mut std::iter::Peekable<E>, open: Option<char>) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    if open.is_some() {
        let p = yet.next();
        anyhow::ensure!(p == open, "expect {:?}, but found {:?}", open, p);
    }

    let close = |p| (p as u8 + 1) as char;
    let result = expression(yet);
    match yet.next() {
        q @ (None | Some(')' | '}' | ']')) if q == open.map(close) => result,
        c => anyhow::bail!("expect end of parenthetic expression, but found {:?}", c),
    }
}

/// expression = [ '+' | '-' ] term { ( '+' | '-' ) term }
fn expression<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    let mut result;
    if let Some('+' | '-') = yet.peek() {
        result = unop(yet.next().expect("peeked '+' or '-'"), term(yet)?);
    } else {
        result = term(yet);
    }

    while let Some('+' | '-') = yet.peek() {
        result = binop(result?, yet.next().expect("peeked '+' or '-'"), term(yet)?);
    }
    result
}

/// term = factor { ( '*' | '/' ) term }
fn term<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    let mut result = factor(yet);
    while let Some('*' | '/') = yet.peek() {
        result = binop(result?, yet.next().expect("peeked '*' or '/'"), factor(yet)?);
    }
    result
}

/// factor = constant | '(' expression ')'
fn factor<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    match yet.peek() {
        Some(n) if n == &'.' || n.is_numeric() => constant(yet),
        Some(&p @ ('(' | '{' | '[')) => parenthetic(yet, Some(p)),
        c => Err(anyhow::anyhow!("expect factor, but found {:?}", c)),
    }
}

/// constant = digit | [digit] '.' digit
/// digit = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
fn constant<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: std::str::FromStr,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    let mut result = yet.peeking_take_while(|d| d.is_numeric()).collect::<String>();
    if let Some('.') = yet.peek() {
        result.push(yet.next().expect("peeked '.'"));
        result.push_str(&yet.peeking_take_while(|d| d.is_numeric()).collect::<String>())
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
