use calculator::calculate;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let expression = &args[1];
    println!("{:?}", calculate::<i64>(expression))
}
