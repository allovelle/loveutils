use ariadne::*;

fn main()
{
    println!("Hello, world!");

    let mut colors = ColorGenerator::new();

    // Generate & choose some colours for each of our elements
    let a = colors.next();
    let b = colors.next();
    let out = Color::Fixed(81);

    Report::build(ReportKind::Warning, "_._", 0)
        .with_code(200123)
        .with_message(format!("What are you {}?", "doing".fg(b)))
        .with_config(Config::default().with_compact(false))
        .with_label(
            Label::new(("_._", 0 .. 3))
                .with_message(format!("This is of type {}", "Nat".fg(a)))
                .with_color(a),
        )
        .with_label(
            Label::new(("_._", 8 .. 10))
                .with_color(Color::Green)
                .with_message("A"),
        )
        .with_label(
            Label::new(("_._", 13 .. 15))
                .with_color(Color::Green)
                .with_message("B"),
        )
        .with_label(
            Label::new(("_._", 19 .. 21))
                .with_color(Color::Green)
                .with_message("C"),
        )
        .with_note("There is probably something")
        .finish()
        .print(("_._", Source::from("add: op.u8(a.u8, b.u8) { a + b }")))
        .unwrap();

    Report::build(ReportKind::Warning, "_._", 0)
        .with_code(200123)
        .with_message(format!("What are you {}?", "doing".fg(b)))
        .with_config(Config::default().with_compact(false))
        .with_label(
            Label::new(("_._", 0 .. 3))
                .with_message(format!("This is of type {}", "Nat".fg(a)))
                .with_color(a),
        )
        .with_label(
            Label::new(("_._", 8 .. 10))
                .with_color(Color::Green)
                .with_message("A"),
        )
        .with_label(
            Label::new(("_._", 13 .. 15))
                .with_color(Color::Green)
                .with_message("B"),
        )
        .with_label(
            Label::new(("_._", 19 .. 21))
                .with_color(Color::Green)
                .with_message("C"),
        )
        .with_note("There is probably something")
        .finish()
        .print(("_._", Source::from("add: op.u8(a.u8, b.u8) { a + b }")))
        .unwrap();
}
