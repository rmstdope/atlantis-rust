mod edit;
mod new;

fn main() {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let exit_code = if arguments
        .first()
        .is_some_and(|argument| atlantis_engine::text::is_equal(argument, "new"))
    {
        new::run(arguments)
    } else {
        edit::run(arguments)
    };
    std::process::exit(exit_code);
}
