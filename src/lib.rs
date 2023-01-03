use std::{
    ops::{Add, Div, Mul, Sub},
    str::{Chars, FromStr},
};

pub trait Calculate: Clone + FromStr + Add + Sub + Mul + Div {}

pub fn calculate<N>(target: &str) -> anyhow::Result<N>
where
    N: Clone + FromStr + Add + Sub + Mul + Div,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
{
    Ok(expression::<N, _, Chars>(target.chars(), None).unwrap().1) // TODO
}

fn expression<N, E, Y>(yet: E, paren: Option<char>) -> anyhow::Result<(Y, N)>
where
    N: Clone + FromStr + Add + Sub + Mul + Div,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: IntoIterator<Item = char>,
    Y: IntoIterator<Item = char>,
{
    let mut yer = yet.into_iter();
    let t = term::<N, _, _>(yer);
    // yer.next();
    // todo!()
    t
}

fn term<N, E, Y>(yet: E) -> anyhow::Result<(Y, N)>
where
    N: Clone + FromStr + Add + Sub + Mul + Div,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: IntoIterator<Item = char>,
    Y: IntoIterator<Item = char>,
{
    let f = factor::<N, _, _>(yet);
    f
}

fn factor<N, E, Y>(yet: E) -> anyhow::Result<(Y, N)>
where
    N: Clone + FromStr + Add + Sub + Mul + Div,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: IntoIterator<Item = char>,
    Y: IntoIterator<Item = char>,
{
    let mut yer = yet.into_iter();
    match yer.next().ok_or_else(|| anyhow::anyhow!("expect factor, but found EOF"))? {
        p @ ('(' | '{' | '[') => expression(yer, Some((p as u8 + 1) as char)),
        n if n.is_numeric() => constant(yer, n),
        c => Err(anyhow::anyhow!("expect factor, but found {}", c)),
    }
}

fn constant<N, E, Y>(yet: E, n: char) -> anyhow::Result<(Y, N)>
where
    N: Clone + FromStr + Add + Sub + Mul + Div,
    <N as std::str::FromStr>::Err: 'static + std::marker::Sync + std::marker::Send + std::error::Error,
    E: IntoIterator<Item = char>,
    Y: IntoIterator<Item = char>,
{
    let (mut yer, mut buff) = (yet.into_iter(), "");
    let integer: String = yer.take_while(|d| d.is_numeric()).collect();
    Ok((yer, N::from_str(&format!("{}{}", n, integer))?))
}
