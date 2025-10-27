use clap::Parser;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use std::fs::File;
use std::io::Write;

/// A CLI tool to find all two-factor multiplications for a given product.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The positive integer to find the multiplications for.
    number: u128,

    /// The name of the output CSV file (without extension).
    #[arg(long, default_value = "product-validated")]
    name: String,
}

fn main() {
    let cli = Cli::parse();
    let number = cli.number;

    if number == 0 {
        eprintln!("Error: The argument must be a positive integer.");
        return;
    }

    let limit: u128 = (number as f64).sqrt() as u128;
    let multiplications: Vec<(u128, u128)> = (1..=limit)
        .into_par_iter()
        .filter(|&multiplier| number % multiplier == 0)
        .map(|multiplier| (multiplier, number / multiplier))
        .collect();

    println!(
        "Found {} multiplications for the product {}. The limit was {}.",
        &multiplications.len(),
        number,
        limit
    );
    for (a, b) in &multiplications {
        println!("{} * {} = {}", a, b, a * b);
    }

    if let Err(e) = generate_csv(&multiplications, number, &cli.name) {
        eprintln!("Error generating CSV file: {}", e);
    }
}

fn generate_csv(
    multiplications: &[(u128, u128)],
    number: u128,
    file_name_base: &str,
) -> std::io::Result<()> {
    let file_name = format!("{}.csv", file_name_base);
    let mut file = File::create(&file_name)?;

    writeln!(file, "Multiplier,Multiplicand,Product")?;

    for &(multiplier, multiplicand) in multiplications {
        writeln!(file, "{},{},{}", multiplier, multiplicand, number)?;
    }

    println!("CSV file generated: {}", file_name);
    Ok(())
}
