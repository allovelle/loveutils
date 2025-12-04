use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use regex::Regex;
use rustpython_parser::{ast, Parse};

type ConsumedChars = usize;

const USAGE: &str = r#"
pq - Query JSON using a DSL that embeds Python expressions
Usage: pq <expr>
Example: echo '{"name":"allovelle"}' | pq 'name.(_.upper())'
"#;

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const BLUE: &str = "\x1b[34m";
const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";

#[derive(Debug)]
pub enum PqError
{
    Json(serde_json::Error),
    Query,
    Python(pyo3::PyErr),
}

#[rustfmt::skip]
impl From<serde_json::Error> for PqError
{
    fn from(value: serde_json::Error) -> Self { Self::Json(value) }
}

#[rustfmt::skip]
impl From<PyErr> for PqError
{
    fn from(value: PyErr) -> Self { Self::Python(value) }
}

fn main() -> Result<(), PqError>
{
    let stdin = std::io::stdin();
    let stdin = stdin.lock();

    let json: serde_json::Value = serde_json::from_reader(stdin)?;
    println!("{YELLOW}{json}{RESET}");

    match std::env::args().nth(1)
    {
        Some(query) =>
        {
            let queries = parse_queries(&query).or(Err(PqError::Query))?;
            println!("{GREEN} Queries: {queries:?}{RESET}");
            process_queries(json, queries)?;
        }
        _ => println!("{}", USAGE.trim()),
    }

    Ok(())
}

#[rustfmt::skip]
#[derive(Debug)]
enum Query
{
    SelectKey { key: String, },
    Index { query: isize, },
    Expression { query: String, },
    BuildObject { query: Vec<BuildObjectQuery>, },
    _Fanout,
    _Join,
    _Select,
}

#[rustfmt::skip]
#[derive(Debug)]
enum BuildObjectQuery
{
    Select(Query),
    Map(Query, Query),
}

fn parse_queries(input: &str) -> Result<Vec<Query>, PqError>
{
    println!("{RED}{input}{RESET}");

    let chars: Vec<char> = input.trim().chars().collect();
    let mut index = 0;
    let mut queries = vec![];

    let mut last_index = 0;

    // TODO(alvl): See if match can be reduced to returning (q, idx)
    while index < chars.len()
    {
        match chars[index]
        {
            '.' => index += 1,
            '{' =>
            {
                // TODO(alvl): Convert exprs to JSON, convert key names to str
                let Ok((query, consumed)) = expect_build_object(&chars, index)
                else
                {
                    return Err(PqError::Query);
                };
                queries.push(query);
                if consumed == 0
                {
                    panic!("{RED}Infinite loop!{RESET}");
                }
                index += consumed;
            }
            '(' =>
            {
                println!("    EXPR");
                let Ok((query, consumed)) = expect_expression(&chars, index)
                else
                {
                    return Err(PqError::Query);
                };
                queries.push(query);
                if consumed == 0
                {
                    panic!("{RED}Infinite loop!{RESET}");
                }
                index += consumed;
            }
            '[' =>
            {
                if accept_index(&chars, 0)
                {
                    let Ok((query, consumed)) = expect_index(&chars, index)
                    else
                    {
                        return Err(PqError::Query);
                    };
                    queries.push(query);
                    if consumed == 0
                    {
                        panic!("{RED}Infinite loop!{RESET}");
                    }
                    index += consumed;
                }
                // TODO(alvl): else if accept...
            }
            'a' ..= 'z' | 'A' ..= 'Z' | '_' =>
            {
                let Ok((query, consumed)) = expect_select_key(&chars, index)
                else
                {
                    return Err(PqError::Query);
                };
                queries.push(query);
                if consumed == 0
                {
                    panic!("{RED}Infinite loop!{RESET}");
                }
                index += consumed;
            }
            _ => panic!("Invalid syntax:\n{input}\n{}^", " ".repeat(index)),
        }

        if last_index == index
        {
            panic!("{RED}Infinite loop!{RESET}");
        }

        last_index = index;
    }

    Ok(queries)
}

fn expect_build_object(
    chars: &[char],
    index: usize,
) -> Result<(Query, ConsumedChars), PqError>
{
    #[rustfmt::skip]
    enum State { Select, Map }

    let input: &String = &chars[index ..].iter().collect();
    let mut start = 1; // Skip initial `{`
    let mut stack = vec![];
    let mut result = vec![];

    let mut state = State::Select; // Refers to what happens at `,` or `}`

    while start < chars.len()
    {
        match chars[start]
        {
            '}' =>
            {
                println!("  <END");
                start += 1;
                match state  // TODO(alvl): Danger, same as below
                {
                    State::Select =>
                    {
                        let key = stack.pop().unwrap();
                        if !matches!(key, Query::SelectKey { .. }) {
                            return Err(PqError::Query);
                        }
                        result.push(BuildObjectQuery::Select(key));
                    }
                    State::Map =>
                    {
                        let key = stack.pop().unwrap();
                        let value = stack.pop().unwrap();
                        result.push(BuildObjectQuery::Map(key, value));
                    }
                }
                break;
            }
            ',' =>
            {
                println!("  <COMMA");
                assert!(
                    stack.len() < 2,
                    "Invalid syntax:\n{input}\n{}^",
                    " ".repeat(start)
                );
                start += 1;
                match state  // TODO(alvl): Danger, same as above
                {
                    State::Select =>
                    {
                        let key = stack.pop().unwrap();
                        if !matches!(key, Query::SelectKey { .. }) {
                            return Err(PqError::Query);
                        }
                        result.push(BuildObjectQuery::Select(key));
                    }
                    State::Map =>
                    {
                        let value = stack.pop().unwrap();
                        let key = stack.pop().unwrap();
                        result.push(BuildObjectQuery::Map(key, value));
                    }
                }
                state = State::Select;
            }
            ':' =>
            {
                println!("  <COLON");
                start += 1;
                state = State::Map;
            }
            '"' =>
            {
                println!("  <STRING");
                let Ok((query, consumed)) = expect_string(chars, start)
                else
                {
                    return Err(PqError::Query);
                };
                stack.push(query);
                if consumed == 0
                {
                    panic!("{RED}Infinite loop!{RESET}");
                }
                start += consumed;
            }
            '(' =>
            {
                println!("  <EXPR");
                let Ok((query, consumed)) = expect_expression(chars, start)
                else
                {
                    return Err(PqError::Query);
                };
                stack.push(query);
                if consumed == 0
                {
                    panic!("{RED}Infinite loop!{RESET}");
                }
                start += consumed;
            }
            'a' ..= 'z' | 'A' ..= 'Z' | '_' =>
            {
                println!("  <SELECT");
                let Ok((query, consumed)) = expect_select_key(chars, start)
                else
                {
                    return Err(PqError::Query);
                };
                stack.push(query);
                if consumed == 0
                {
                    panic!("{RED}Infinite loop!{RESET}");
                }
                start += consumed;
            }
            _ => panic!("Invalid syntax:\n{input}\n{}^", " ".repeat(start)),
        }
    }

    println!("Queries :: {result:?}");

    Ok((Query::BuildObject { query: result }, start))
}

fn expect_select_key(
    chars: &[char],
    index: usize,
) -> Result<(Query, ConsumedChars), PqError>
{
    if chars[index].is_numeric()
    {
        return Err(PqError::Query);
    }

    let mut end = index;
    while end < chars.len()
        && (chars[end].is_alphanumeric() || chars[end] == '_')
    {
        end += 1;
    }

    let key: String = chars[index .. end].iter().collect();
    let consumed = end - index;
    Ok((Query::SelectKey { key }, consumed))
}

fn accept_index(chars: &[char], index: usize) -> bool
{
    let input: String = chars[index ..].iter().collect();
    let re = Regex::new(r"\[\s*(-?)\s*(\d+)\s*\]").unwrap();
    re.is_match(&input)
}

fn expect_index(
    chars: &[char],
    index: usize,
) -> Result<(Query, ConsumedChars), PqError>
{
    let input: &String = &chars[index ..].iter().collect();
    let re = Regex::new(r"\[\s*(-?)\s*(\d+)\s*\]").or(Err(PqError::Query))?;

    if let (Some(caps), Some(consumed)) =
        (re.captures(input), re.shortest_match(input))
    {
        if let Some(num) = caps.get(2)
        {
            let number: isize = num.as_str().parse().or(Err(PqError::Query))?;
            // let negative =
            //     -caps.get(1).map(|g| g.as_str().len() as isize).unwrap_or(-1);
            let negative = if let Some(cap) = caps.get(1)
            {
                if cap.as_str().is_empty()
                {
                    1
                }
                else
                {
                    -1
                }
            }
            else
            {
                1
            };

            println!("{YELLOW}{:?}{RESET}", re.shortest_match(input));

            return Ok((Query::Index { query: number * negative }, consumed));
        }
    }

    Err(PqError::Query)
}

fn expect_string(
    chars: &[char],
    index: usize,
) -> Result<(Query, ConsumedChars), PqError>
{
    let python_source: &String = &chars[index ..].iter().collect();
    let mut start = 0;
    let mut valid_index: Option<usize> = None;
    while start < python_source.len()
    {
        println!("{RED} -> {start} {} {RESET}", &python_source[..= start]);
        if ast::Expr::parse(&python_source[..= start], "").is_ok()
        {
            valid_index = Some(start);
            break;
        }
        start += 1
    }

    if let Some(consumed) = valid_index
    {
        let query = format!("({})", &python_source[0 ..= consumed]);
        println!("{} | INDEX: {}", &query, start + consumed);
        return Ok((Query::Expression { query }, consumed + 1));
    }

    Err(PqError::Query)
}

fn expect_expression(
    chars: &[char],
    index: usize,
) -> Result<(Query, ConsumedChars), PqError>
{
    let python_source: &String = &chars[index ..].iter().collect();
    let mut start = 0;
    let mut valid_index: Option<usize> = None;
    while start < python_source.len()
    {
        println!("{RED} -> {start} {} {RESET}", &python_source[..= start]);
        if ast::Expr::parse(&python_source[..= start], "").is_ok()
        {
            valid_index = Some(start);
            break;
        }
        start += 1
    }

    if let Some(consumed) = valid_index
    {
        println!(
            "{} | INDEX: {}",
            &python_source[0 ..= consumed],
            start + consumed
        );
        let query = python_source[0 ..= consumed].to_string();
        return Ok((Query::Expression { query }, consumed + 1));
    }

    Err(PqError::Query)
}

fn _accept_fanout(_chars: &[char], _index: usize) -> bool
{
    false
}

fn _accept_join(_chars: &[char], _index: usize) -> bool
{
    false
}

fn _accept_select(_chars: &[char], _index: usize) -> bool
{
    false
}

// use pyo3::prelude::*;
// use pyo3::types::{PyDict, PyNone};
// use pyo3::*;
// use serde_json::{Map, Value};

// fn value_to_py_dict<'a>(
//     py: Python<'a>,
//     value: &Value,
// ) -> PyResult<Bound<'a, PyAny>>
// {
//     let dict = PyDict::new_bound(py);

//     // TODO(alvl): Just convert the value to string and parse it w/Python lol

//     let none: Bound<'a, PyAny> =
//         py.eval_bound(&format!("None"), None, None)?.extract()?;

//     // [("_", line)].into_py_dict_bound(py);

//     if let Value::Object(map) = value
//     {
//         for (key, value) in map.iter()
//         {
//             let py_val: Bound<'a, PyAny> = match value
//             {
//                 Value::Null => none,
//                 Value::Bool(_) => todo!(),
//                 Value::Number(_) => todo!(),
//                 Value::String(_) => todo!(),
//                 Value::Array(_) => todo!(),
//                 Value::Object(_) => value_to_py_dict(py, value)?,
//             };
//         }
//     }
//     Ok(dict)
// }

fn process_queries(
    json: serde_json::Value,
    queries: Vec<Query>,
) -> Result<(), PqError>
{
    let mut json_state = json;

    for query in queries
    {
        println!("    {BLUE}{json_state}{RESET}");

        match query
        {
            Query::SelectKey { key } =>
            {
                json_state = json_state[key].clone();
            }
            Query::Index { query } =>
            {
                let key = if query < 0
                {
                    json_state.as_array().unwrap().len() as isize + query
                }
                else
                {
                    query
                } as usize;
                json_state = json_state[key].clone();
            }
            Query::BuildObject { query } =>
            {
                let mut new_json_state = serde_json::json!({});
                for sub in query.iter()
                {
                    match sub
                    {
                        BuildObjectQuery::Select(select) =>
                        {
                            let Query::SelectKey { key } = select
                            else
                            {
                                return Err(PqError::Query);
                            };
                            new_json_state[key] = json_state[key].clone();
                        }
                        BuildObjectQuery::Map(expr_key, expr_val) =>
                        {
                            match expr_key
                            {
                                Query::SelectKey { key } =>
                                {
                                    let the_key = json_state[key].clone();
                                    let Some(result_key) = the_key.as_str()
                                    else
                                    {
                                        return Err(PqError::Query);
                                    };
                                    match expr_val
                                    {
                                        Query::SelectKey { key: val_key } =>
                                        {
                                            // let k = json_state[key].clone();
                                            let v = json_state[val_key].clone();
                                            new_json_state[result_key] = v;
                                        }
                                        Query::Expression {
                                            // TODO(alvl): Run value Python query
                                            query: _val_query,
                                        } => (),
                                        _ => return Err(PqError::Query),
                                    }
                                }
                                Query::Expression { query: _key } =>
                                {
                                    // TODO(alvl): Run both the key & value Python queries
                                    {
                                        // TODO(alvl): For all keys, add them as locals
                                    }
                                    let result_key = ""; // TODO(alvl): Run Py

                                    match expr_val
                                    {
                                        Query::SelectKey { key: val_key } =>
                                        {
                                            let v = json_state[val_key].clone();
                                            new_json_state[result_key] = v;
                                        }
                                        Query::Expression {
                                            query: _val_query,
                                        } => (),
                                        _ => return Err(PqError::Query),
                                    }
                                }
                                _ => return Err(PqError::Query),
                            }
                        }
                    }
                }
                json_state = new_json_state;
            }
            Query::Expression { query } =>
            {
                Python::with_gil::<_, Result<(), PqError>>(|py| {
                    let locals = [("json", py.import_bound("json")?)]
                        .into_py_dict_bound(py);

                    let result = py
                        .eval_bound(
                            &format!("(_ := {json_state})"),
                            None,
                            Some(&locals),
                        )
                        .and(py.eval_bound(
                            &format!("json.dumps{query}"),
                            None,
                            Some(&locals),
                        ));

                    let result = result?;
                    let str_expr: String = result.extract()?;

                    json_state = serde_json::from_str(&str_expr)?;

                    Ok(())
                })?
            }
            Query::_Fanout => todo!(),
            Query::_Join => todo!(),
            Query::_Select => todo!(),
        }
    }

    println!("{} <-- FINAL UPDATE", json_state);

    Ok(())
}
