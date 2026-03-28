//! Dev utility: print bcrypt hash for a plaintext password (same cost as `utils::password`).
//! Usage: `cargo run --bin hash_password -- "your password"`

fn main() {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        eprintln!("Usage: cargo run --bin hash_password -- <password>");
        std::process::exit(1);
    }
    let plain = args.remove(0);
    match bcrypt::hash(&plain, bcrypt::DEFAULT_COST) {
        Ok(h) => println!("{}", h),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
