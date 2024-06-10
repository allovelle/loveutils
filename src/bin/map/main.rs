use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

const USAGE: &str = r#"
map - Execute a Python expression on each line from STDIN
Usage: map <py expr>
Example: ps | '_.upper()'
"#;

fn main() -> PyResult<()>
{
    match std::env::args().take(2).skip(1).next()
    {
        Some(cmd) => Python::with_gil(|py| {
            for line in std::io::stdin().lines().flatten()
            {
                let locals = [("_", line)].into_py_dict_bound(py);
                let result =
                    py.eval_bound(&format!("str({cmd})"), None, Some(&locals))?;
                println!("{result}");
            }
            Ok(())
        }),
        _ => Ok(println!("{}", USAGE.trim())),
    }
}
