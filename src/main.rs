use clap::{Parser, ValueEnum};
use multiplication_product_validator::{
    FactorPair, default_threads, factor_pairs_from, factor_pairs_trial, factorize,
    format_factorization, init_thread_pool,
};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

/// A CLI tool to find all two-factor multiplications for a given product.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The positive integer to find the multiplications for.
    #[arg(value_parser = parse_positive)]
    number: u128,

    /// The name of the output CSV file (without extension).
    #[arg(long, default_value = "product-validated")]
    name: String,

    /// Do not write the CSV file.
    #[arg(long, conflicts_with = "name")]
    no_csv: bool,

    /// Number of worker threads [default: logical cores minus one].
    #[arg(short = 'j', long, value_parser = clap::value_parser!(u16).range(1..))]
    threads: Option<u16>,

    /// How to search for the factor pairs.
    #[arg(long, value_enum, default_value_t = Method::Factorize)]
    method: Method,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Method {
    /// Prime factorization (Pollard's rho), then build the divisors.
    Factorize,
    /// Test every candidate up to the square root (slow; for comparison).
    Trial,
}

fn parse_positive(value: &str) -> Result<u128, String> {
    match value.parse::<u128>() {
        Ok(0) => Err("the number must be a positive integer".to_owned()),
        Ok(n) => Ok(n),
        Err(e) => Err(e.to_string()),
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let number = cli.number;
    let threads = cli.threads.map_or_else(default_threads, usize::from);
    if let Err(e) = init_thread_pool(threads) {
        eprintln!("Error creating the thread pool: {e}");
        return ExitCode::FAILURE;
    }
    let factors = factorize(number);
    let multiplications = match cli.method {
        Method::Factorize => factor_pairs_from(number, &factors),
        Method::Trial => factor_pairs_trial(number),
    };

    if let Err(e) = print_pairs(&multiplications, number, &factors) {
        // A closed pipe (e.g. `| head`) is not an error worth reporting.
        if e.kind() != io::ErrorKind::BrokenPipe {
            eprintln!("Error writing to stdout: {e}");
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    if !cli.no_csv {
        let file_name = format!("{}.csv", cli.name);
        if let Err(e) = generate_csv(&multiplications, number, &file_name) {
            eprintln!("Error generating CSV file {file_name}: {e}");
            return ExitCode::FAILURE;
        }
        println!("CSV file generated: {file_name}");
    }

    ExitCode::SUCCESS
}

fn print_pairs(
    multiplications: &[FactorPair],
    number: u128,
    factors: &[(u128, u32)],
) -> io::Result<()> {
    let mut out = BufWriter::new(io::stdout().lock());
    writeln!(
        out,
        "Found {} multiplications for the product {number}.",
        multiplications.len()
    )?;
    writeln!(
        out,
        "Prime factorization: {}",
        format_factorization(factors)
    )?;
    for (a, b) in multiplications {
        writeln!(out, "{a} * {b} = {number}")?;
    }
    out.flush()
}

fn generate_csv(multiplications: &[FactorPair], number: u128, file_name: &str) -> io::Result<()> {
    let mut file = BufWriter::new(File::create(file_name)?);
    writeln!(file, "Multiplier,Multiplicand,Product")?;
    for (multiplier, multiplicand) in multiplications {
        writeln!(file, "{multiplier},{multiplicand},{number}")?;
    }
    file.flush()
}
