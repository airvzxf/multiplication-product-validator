use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use std::env;
use std::fs::File;
use std::io::Write;

fn main() {
    // 1. Allow the user to send an argument via the CLI; this must be a positive integer.
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <positive_integer>", &args[0]);
        return;
    }

    let number: u128 = match args[1].trim().parse() {
        Ok(n) if n > 0 => n,
        _ => {
            eprintln!("Error: The argument must be a positive integer.");
            return;
        }
    };

    // 2. Then do all the two-factor multiplications until the number of digits in the number plus one is complete.
    let limit: u128 = (number as f64).sqrt() as u128;
    let multiplications: Vec<(u128, u128)> = (1..=limit)
        .into_par_iter()
        .filter(|&multiplier| number % multiplier == 0)
        .map(|multiplier| (multiplier, number / multiplier))
        .collect();

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

    // 4. Generate the CSV file.
    if let Err(e) = generate_csv(&multiplications, number) {
        eprintln!("Error generating CSV file: {}", e);
    }
}

fn generate_csv(multiplications: &[(u128, u128)], number: u128) -> std::io::Result<()> {
    let file_name = format!("product_{}.csv", number);
    let mut file = File::create(&file_name)?;

    writeln!(file, "Multiplier,Multiplicand,Product")?;

    for &(multiplier, multiplicand) in multiplications {
        writeln!(file, "{},{},{}", multiplier, multiplicand, number)?;
    }

    println!("CSV file generated: {}", file_name);
    Ok(())
}
