use calculator_core::{
    eval::{Environment, Eval},
    expr::Expr,
    parse::parse_from_str,
};
use clap::Parser;
use std::io::{stdin, stdout, Write};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Use verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut input = String::new();
    let mut env = Environment::default();
    loop {
        input.clear();
        print!("> ");
        stdout().flush()?;
        stdin().read_line(&mut input)?;
        let expr: Expr = match parse_from_str(&input) {
            Ok(expr) => expr,
            Err(err) => {
                eprintln!("error: {}", err);
                continue;
            }
        };
        if args.verbose {
            eprintln!("expr = {:?}", expr);
        }
        match expr.eval(&mut env) {
            Ok(e) => println!("{}", e),
            Err(err) => eprintln!("error: {}", err),
        }
    }
}
