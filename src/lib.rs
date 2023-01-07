use anyhow::Context as _;
use itertools::Itertools;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// calculate string expression consisted of only four arithmetic operations.
pub fn calculate<N>(target: &str) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
{
    let mut reader = target.chars().filter(|c| !c.is_whitespace()).peekable();
    expression(&mut reader)
        .and_then(|n| reader.peek().map_or(Ok(n), |_| anyhow::bail!("success calculated, but expression is remaining")))
        .with_context(|| anyhow::anyhow!("fail calculate {:?}, remaining {:?}", target, reader.collect::<String>()))
}

/// expression = [ '+' | '-' ] term [ ( '+' | '-' ) recursive_expression ]
fn expression<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    let t = if let Some('+' | '-') = yet.peek() {
        unop(yet.next().expect("peeked '+' or '-'"), term(yet)?)?
    } else {
        term(yet)?
    };
    match yet.peek() {
        Some('+' | '-') => recursive_expression(yet, t),
        _ => Ok(t),
    }
}

/// recursive_expression = ( '+' | '-' ) term  [ ( '+' | '-' ) recursive_expression ]
fn recursive_expression<N, E>(yet: &mut std::iter::Peekable<E>, val: N) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    let op = yet.next().ok_or_else(|| anyhow::anyhow!("expression must start with '+' or '-'"))?;
    let val = binop(val, op, term(yet)?)?;
    match yet.peek() {
        Some('+' | '-') => recursive_expression(yet, val),
        _ => Ok(val),
    }
}

/// term = factor [ ( '*' | '/' ) recursive_term ]
fn term<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    let val = factor(yet)?;
    match yet.peek() {
        Some('*' | '/') => recursive_term(yet, val),
        _ => Ok(val),
    }
}

/// recursive_term = ( '*' | '/' ) factor [ ( '*' | '/' ) recursive_term ]
fn recursive_term<N, E>(yet: &mut std::iter::Peekable<E>, val: N) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    let op = yet.next().ok_or_else(|| anyhow::anyhow!("term must start with '*' or '/'"))?;
    let val = binop(val, op, factor(yet)?)?;
    match yet.peek() {
        Some('*' | '/') => recursive_term(yet, val),
        _ => Ok(val),
    }
}

/// factor = constant | '(' expression ')'
fn factor<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    match yet.peek() {
        Some('(' | '{' | '[') => parenthetic(yet),
        Some(n) if n == &'.' || n.is_numeric() => constant(yet),
        c => Err(anyhow::anyhow!("expect factor, but found {:?}", c)),
    }
}

/// parenthetic = '(' expression ')'
fn parenthetic<N, E>(yet: &mut std::iter::Peekable<E>) -> anyhow::Result<N>
where
    N: std::str::FromStr + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N> + Neg<Output = N>,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: Iterator<Item = char>,
{
    let close = |p| if p == '(' { p as u8 + 1 } else { p as u8 + 2 } as char;
    let p = yet.next().ok_or_else(|| anyhow::anyhow!("parenthetic must start with with '(' or '{{' or '['"))?;

    let val = expression(yet)?;
    match yet.next() {
        Some(q @ (')' | '}' | ']')) if q == close(p) => Ok(val),
        c => anyhow::bail!("expect end of parenthetic, but found {:?}", c),
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
        assert!(matches!(calculate::<i64>("[{(1+2)}]+(3)"), Ok(6)));
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

    #[test]
    fn test_large1() {
        assert_eq!(calculate::<i128>(&vec!["1"; 1000000].join("+")).unwrap(), 1000000);
    }

    #[test]
    fn test_large2() {
        assert_eq!(calculate::<i128>(&vec!["1-1"; 500000].join("+")).unwrap(), 0);
    }

    #[test]
    fn test_large3() {
        assert_eq!(calculate::<i128>(&vec!["2/2"; 500000].join("*")).unwrap(), 1);
    }
}
