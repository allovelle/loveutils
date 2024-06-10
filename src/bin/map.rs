use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

const USAGE: &str = r#"
map - Print lines from STDIN using a string Python expression
Usage: map <py expr>
Example: ps | map '_.upper()'
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
                let str_expr: String = result.extract()?;
                println!("{str_expr}");
            }
            Ok(())
        }),
        _ => Ok(println!("{}", USAGE.trim())),
    }
}
