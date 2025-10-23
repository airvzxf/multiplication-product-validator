use std::env;

fn main() {
    // 1. Allow the user to send an argument via the CLI; this must be a positive integer.
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <positive_integer>", &args[0]);
        return;
    }

    // 2. You are going to count how many digits the number the user entered has.
    let num_digits: usize = args[1].trim().len();

    let number: u64 = match args[1].trim().parse() {
        Ok(n) if n > 0 => n,
        _ => {
            eprintln!("Error: The argument must be a positive integer.");
            return;
        }
    };

    // 3. Then do all the two-factor multiplications until the number of digits in the number plus one is complete.
    let mut multiplications: Vec<(u64, u64)> = Vec::new();
    let limit: u64 = 10u64.pow(num_digits as u32 + 1) - 1;
    println!("Limit: {}", limit);
    // return;

    let mut multiplier: u64 = 0u64;
    loop {
        // 3.1. The multiplicand starts from the same number as the multiplier.
        let mut multiplicand: u64 = multiplier;
        loop {
            let product: u128 = multiplier as u128 * multiplicand as u128;

            // 3.2. Whenever the result is equal to what the user passed through the CLI, then those multiplications are saved in an array.
            if product == number as u128 {
                multiplications.push((multiplier, multiplicand));
            }

            multiplicand += 1;
            if multiplicand > limit {
                // println!("- {} x {}", multiplier, multiplicand);
                break;
            }
        }

        multiplier += 1;
        if multiplier > limit {
            // println!("+ {} x {}", multiplier, multiplicand);
            break;
        }
    }

    // 4. These multiplications are displayed on the screen.
    println!("Found the following multiplications for {}:", number);
    for (a, b) in &multiplications {
        println!("= {} * {} = {}", a, b, a * b);
    }
}
