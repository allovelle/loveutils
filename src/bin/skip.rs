//! Skip the first N lines of STDIN

const USAGE: &str = r#"
skip - Skip a number of lines from STDIN
Usage: skip <num lines>
Example: ps | skip 1
"#;

fn main()
{
    match std::env::args().nth(1).map(|s| s.parse())
    {
        Some(Ok(arg)) =>
        {
            for line in std::io::stdin().lines().skip(arg)
            {
                match line
                {
                    Ok(ln) => println!("{ln}"),
                    Err(err) => println!("Error: {err}"),
                }
            }
        }
        _ => println!("{}", USAGE.trim()),
    }
}
