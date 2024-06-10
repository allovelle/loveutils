///! Skip the first N lines of STDIN

fn main() {
    match std::env::args().take(2).skip(1).next().map(|s| s.parse()) {
        Some(Ok(arg)) => {
            for line in std::io::stdin().lines().skip(arg) {
                match line {
                    Ok(ln) => println!("{ln}"),
                    Err(err) => println!("Error: {err}"),
                }
            }
        }
        _ => println!("Usage: skip <num lines>\nExample: ps | skip 1"),
    }
}
