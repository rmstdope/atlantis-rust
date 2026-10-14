mod edit;

fn main() {
    std::process::exit(edit::run(std::env::args().skip(1).collect()));
}
