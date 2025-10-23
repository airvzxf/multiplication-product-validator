use std::env;

fn main() {
    // 1. Allow the user to send an argument via the CLI; this must be a positive integer.
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <positive_integer>", &args[0]);
        return;
    }

    let number: u64 = match args[1].trim().parse() {
        Ok(n) if n > 0 => n,
        _ => {
            eprintln!("Error: The argument must be a positive integer.");
            return;
        }
    };

    // 2. Then do all the two-factor multiplications until the number of digits in the number plus one is complete.
    let mut multiplications: Vec<(u64, u64)> = Vec::new();
    let limit: u64 = (number as f64).sqrt() as u64;

    for multiplier in 1..=limit {
        if number % multiplier == 0 {
            let multiplicand: u64 = number / multiplier;
            multiplications.push((multiplier, multiplicand));
        }
    }

    // 3. These multiplications are displayed on the screen.
    println!(
        "Found {} multiplications for the product {}. The limit was {}.",
        &multiplications.len(),
        number,
        limit
    );
    for (a, b) in &multiplications {
        println!("{} * {} = {}", a, b, a * b);
    }
}
