use calculator::calculate;

use clap::Parser;

/// Simple program to calculate simple expression
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// expression to calculate
    expression: String,

    /// use rational
    #[arg(short, long, default_value_t = false)]
    precisely: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.precisely {
        output_result(calculate::<num::Rational64>(&args.expression))
    } else if args.expression.contains('.') {
        output_result(calculate::<f64>(&args.expression))
    } else {
        output_result(calculate::<i64>(&args.expression))
    }
}

fn output_result<T>(result: anyhow::Result<T>) -> Result<(), Box<dyn std::error::Error>>
where
    T: std::fmt::Display,
{
    match result {
        Ok(n) => Ok(println!("{}", n)),
        Err(e) => Err(e.into()),
    }
}
