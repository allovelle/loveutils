use regex::Regex;
use rustpython_parser::{ast, Parse};
use std::os::unix::process;

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
}

impl From<serde_json::Error> for PqError
{
    fn from(value: serde_json::Error) -> Self
    {
        Self::Json(value)
    }
}

fn main() -> Result<(), PqError>
{
    let stdin = std::io::stdin();
    let stdin = stdin.lock();

    let json: serde_json::Value = serde_json::from_reader(stdin)?;
    println!("{YELLOW}{json}{RESET}");

    match std::env::args().skip(1).next()
    {
        Some(query) =>
        {
            let queries = parse_queries(&query).or(Err(PqError::Query))?;
            println!("{GREEN}{queries:?}{RESET}");
            process_queries(json, queries)?;
        }
        _ => println!("{}", USAGE.trim()),
    }

    Ok(())
}

fn exe_expr2()
{
    let python_source = "(print('Hello world')).a.()";

    // let python_statements = ast::Suite::parse(python_source, "").unwrap(); // statements
    // println!("-1-> {python_statements:?}");

    let mut index = 0;
    let mut valid_index: Option<usize> = None;
    while index < python_source.len()
    {
        if ast::Expr::parse(&python_source[0 .. index], "").is_ok()
        {
            valid_index = Some(index);
            break;
        }

        index += 1
    }

    if let Some(end_index) = valid_index
    {
        println!("{}", &python_source[0 .. end_index]);
    }
}

#[rustfmt::skip]
#[derive(Debug)]
enum Query
{
    SelectKey { key: String, },
    Index { query: isize, },
    BuildObject { query: String, },
    Expression { query: String, },
    Fanout,
    Join,
    Select,
}

fn parse_queries(input: &str) -> Result<Vec<Query>, PqError>
{
    println!("{RED}{input}{RESET}");

    let chars: Vec<char> = input.trim().chars().collect();
    let mut index = 0;
    let mut queries = vec![];

    let mut last_index = 0;

    while index < chars.len()
    {
        match chars[index]
        {
            '{' => index += 1,
            '(' => index += 1,
            '[' =>
            {
                if accept_index(&chars, 0)
                {
                    if let Ok((query, end_index)) = expect_index(&chars, index)
                    {
                        queries.push(query);
                        if end_index == index
                        {
                            panic!("{RED}Infinite loop!{RESET}");
                        }
                        index += end_index;
                    }
                }
            }
            'a' ..= 'z' | 'A' ..= 'Z' | '_' =>
            {
                if let Ok((query, end_index)) = expect_select_key(&chars, index)
                {
                    queries.push(query);
                    if end_index == index
                    {
                        panic!("{RED}Infinite loop!{RESET}");
                    }
                    index += end_index;
                }
                else
                {
                    return Err(PqError::Query);
                }
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

fn process_queries(
    json: serde_json::Value,
    queries: Vec<Query>,
) -> Result<(), PqError>
{
    let mut json_state = json;

    for query in queries
    {
        match query
        {
            Query::SelectKey { key } =>
            {
                json_state = json_state[key].clone();
            }
            Query::Index { query } => todo!(),
            Query::BuildObject { query } => todo!(),
            Query::Expression { query } => todo!(),
            Query::Fanout => todo!(),
            Query::Join => todo!(),
            Query::Select => todo!(),
        }
    }

    println!("{}", json_state);

    Ok(())
}

fn expect_select_key(
    chars: &Vec<char>,
    index: usize,
) -> Result<(Query, usize), PqError>
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

    let key: String = chars[index .. end].into_iter().collect();
    Ok((Query::SelectKey { key }, end))
}

fn accept_index(chars: &Vec<char>, index: usize) -> bool
{
    let input: String = chars[index ..].into_iter().collect();
    let re = Regex::new(r"\[\s*(-?)\s*(\d+)\s*\]").unwrap();
    re.is_match(&input)
}

fn expect_index(
    chars: &Vec<char>,
    index: usize,
) -> Result<(Query, usize), PqError>
{
    let input: &String = &chars[index ..].into_iter().collect();
    let re = Regex::new(r"\[\s*(-?)\s*(\d+)\s*\]").or(Err(PqError::Query))?;

    if let (Some(caps), Some(end_index)) =
        (re.captures(input), re.shortest_match(input))
    {
        if let Some(num) = caps.get(2)
        {
            let number: isize = num.as_str().parse().or(Err(PqError::Query))?;
            let negative =
                -caps.get(1).map(|g| g.as_str().len() as isize).unwrap_or(-1);

            println!("{YELLOW}{:?}{RESET}", re.shortest_match(input));

            return Ok((Query::Index { query: number * negative }, end_index));
        }
    }

    Err(PqError::Query)
}

fn accept_fanout(chars: &Vec<char>, index: usize) -> bool
{
    false
}
fn accept_join(chars: &Vec<char>, index: usize) -> bool
{
    false
}
fn accept_select(chars: &Vec<char>, index: usize) -> bool
{
    false
}
