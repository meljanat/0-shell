mod utils;
use std::io::{self, Write};
use utils::*;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).unwrap() == 0 {
            println!("\nexit");
            break;
        }
        let input = skip_quotes(&input.trim());

        let args = input.split_whitespace().collect::<Vec<&str>>();
        if args.is_empty() {
            continue;
        }
        let cmd = args[0];
        let args = &args[1..];

        match cmd {
            "echo" => echo(args),
            "cd" => cd(args),
            "pwd" => pwd(),
            "ls" => ls(args),
            "cat" => cat(args),
            "cp" => cp(args),
            "rm" => rm(args),
            "mv" => mv(args),
            "mkdir" => mkdir(args),
            "exit" => {
                print!("exit\n");
                break;
            }
            _ => println!("Command '{}' not found", cmd),
        }
    }
}
