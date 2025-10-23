# Multiplication: Product Validator

Given a number (the product), this application will find and display all the pairs of two factors that multiply to produce that number.

## For Users

### What it does

This tool takes a single positive integer as an input and calculates all the unique pairs of two numbers (factors) that, when multiplied together, equal the input number. For example, if you input `12`, the program will output:

```
1 * 12 = 12
2 * 6 = 12
3 * 4 = 12
```

### How to use it

1.  **Prerequisites:** You need to have Rust and Cargo installed on your system. If you don't have them, you can install them from [https://rustup.rs/](https://rustup.rs/).
2.  **Navigate to the project directory:**
    ```bash
    cd src/multiplication-product-validator
    ```
3.  **Run the application:**
    ```bash
    cargo run <number>
    ```
    Replace `<number>` with the positive integer you want to find the factors for.

    **Example:**
    ```bash
    cargo run 144
    ```

## For Developers

### Getting Started

To set up the development environment, you'll need to have Rust and Cargo installed.

1.  **Install Rust and Cargo:** Follow the instructions on the official Rust website: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
2.  **Clone the repository:**
    ```bash
    git clone <repository-url>
    ```
3.  **Navigate to the project directory:**
    ```bash
    cd src/multiplication-product-validator
    ```

### Project Structure

The project is a standard Cargo project:

```
.
├── Cargo.toml  # Project manifest and dependencies
├── src
│   └── main.rs # Main application logic
└── target      # Build artifacts
```

The main logic is in `src/main.rs`. It uses the `rayon` crate to parallelize the search for factors, making it efficient for large numbers.

### Building and Running

*   **To build the project:**
    ```bash
    cargo build
    ```
*   **To build in release mode (for performance):**
    ```bash
    cargo build --release
    ```
*   **To run the project:**
    ```bash
    cargo run <number>
    ```

### Testing

To run the tests for this project, you can use the following command:

```bash
cargo test
```
*(Note: No tests have been implemented yet.)*

### Contributing

Contributions are welcome! Please feel free to submit a pull request.

1.  Fork the repository.
2.  Create a new branch (`git checkout -b feature/your-feature`).
3.  Make your changes.
4.  Commit your changes (`git commit -m 'Add some feature'`).
5.  Push to the branch (`git push origin feature/your-feature`).
6.  Open a pull request.