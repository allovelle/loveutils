#![allow(clippy::unit_arg)]

use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

const USAGE: &str = r#"
fold - Process lines from STDIN using Python expressions with accumulator state
Usage: fold <py expr> <py expr>
Example: git -h | fold 'import os; a = os.getenv("FOO")' 'a += int(_)'
"#;

// TODO(alvl): Use RUN to do statements for both acc and tick:
// TODO https://pyo3.rs/v0.8.3/python_from_rust#want-to-run-statements-then-use-run

fn main() -> PyResult<()>
{
    let args: Vec<String> = std::env::args().skip(1).take(2).collect();
    match &args[..]
    {
        [init, cmd] => Python::with_gil(|py| {
            let locals = [("_", "")].into_py_dict_bound(py);
            let _acc =
                py.eval_bound(&format!("({init})"), None, Some(&locals))?;

            for line in std::io::stdin().lines().map_while(Result::ok)
            {
                locals.set_item("_", &line)?;
                let result =
                    py.eval_bound(&format!("str({cmd})"), None, Some(&locals))?;
                let str_expr: String = result.extract().unwrap();
                println!("{str_expr}");
            }
            Ok(())
        }),
        _ => Ok(println!("{}", USAGE.trim())),
    }
}
