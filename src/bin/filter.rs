#![allow(clippy::unit_arg)]
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

const USAGE: &str = r#"
filter - Filter lines from STDIN using a boolean Python expression
Usage: filter <py expr>
Example: git -h | filter 'int(_) > 77'
"#;

fn main() -> PyResult<()>
{
    match std::env::args().nth(1)
    {
        Some(cmd) => Python::with_gil(|py| {
            for line in std::io::stdin().lines().map_while(Result::ok)
            {
                let locals = [("_", &line)].into_py_dict_bound(py);
                let result = py.eval_bound(
                    &format!("bool({cmd})"),
                    None,
                    Some(&locals),
                )?;
                let bool_expr: bool = result.extract().unwrap();
                if bool_expr
                {
                    println!("{line}");
                }
            }
            Ok(())
        }),
        _ => Ok(println!("{}", USAGE.trim())),
    }
}
