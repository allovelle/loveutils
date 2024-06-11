use rustpython_parser::{ast, Parse};

fn main()
{
    let python_source = "(print('Hello world')).a.()";

    // let python_statements = ast::Suite::parse(python_source, "").unwrap(); // statements
    // println!("-1-> {python_statements:?}");

    // TODO(alvl): Find the furthest valid Python expression

    let mut index = 0;
    let mut valid_index: Option<usize> = None;
    while index < python_source.len()
    {
        if ast::Expr::parse(&python_source[0 .. index], "").is_ok()
        {
            valid_index = Some(index);
            break;
            // if valid_index.is_some()
            // {
            //     valid_index = Some(valid_index.unwrap().max(index));
            // }
            // else
            // {
            //     valid_index = Some(index);
            // }
            // break;
        }

        index += 1
    }

    if let Some(end_index) = valid_index
    {
        println!("{}", &python_source[0 .. end_index]);
    }
}
