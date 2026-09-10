//! Native fixture used by the offline npm launcher integration tests.

use std::io::{self, Read};

fn main() {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if arguments == ["--version"] {
        println!("quire 1.2.3 (distribution-test-probe)");
        return;
    }
    if arguments == ["--abort"] {
        std::process::abort();
    }
    let exit = arguments
        .iter()
        .position(|argument| argument == "--exit")
        .and_then(|index| arguments.get(index + 1))
        .and_then(|value| value.parse::<i32>().ok());
    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin).expect("read stdin");
    println!(
        "args={}",
        serde_json::to_string(&arguments).expect("JSON args")
    );
    println!("stdin={stdin}");
    eprintln!("probe-stderr");
    if let Some(code) = exit {
        std::process::exit(code);
    }
}
