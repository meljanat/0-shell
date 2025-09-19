use std::io::{self, Write};

pub fn echo(input: &[&str]) {
    let initial = input.join(" ");

    if normal(&initial, ' ') {
        println!("{}", skip_quotes(&initial));
    } else {
        let mut result = initial.clone();
        let mut quote_char = which_quotes(&initial);

        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut extra = String::new();
            if io::stdin().read_line(&mut extra).unwrap() == 0 {
                break;
            }

            result.push_str("\n");
            result.push_str(extra.trim_end());

            if normal(&extra, quote_char) {
                break;
            }
            quote_char = which_quotes(&result);
        }
        println!("{}", skip_quotes(&result));
    }
}

fn normal(input: &str, qt: char) -> bool {
    let mut quote: char = qt;
    for c in input.chars() {
        if c == '"' || c == '\'' {
            if quote == ' ' {
                quote = c;
            } else if quote == c {
                quote = ' ';
            }
        }
    }
    quote == ' '
}

fn skip_quotes(input: &str) -> String {
    let mut result = String::new();
    let mut quote: char = ' ';
    for c in input.chars() {
        if c == '"' || c == '\'' {
            if quote == ' ' {
                quote = c;
            } else if quote == c {
                quote = ' ';
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn which_quotes(input: &str) -> char {
    let mut quote: char = ' ';
    for c in input.chars() {
        if c == '"' || c == '\'' {
            if quote == ' ' {
                quote = c;
            } else if quote == c {
                quote = ' ';
            }
        }
    }
    quote
}

pub fn pwd() {
    match std::env::current_dir() {
        Ok(path) => println!("{}", path.display()),
        Err(e) => eprintln!("Error getting current directory: {}", e),
    }
}

pub fn cat(input: &[&str]) {
    if input.is_empty() {
        return;
    }

    for &file in input {
        match std::fs::read_to_string(file) {
            Ok(content) => print!("{}", content),
            Err(e) => eprintln!("cat: {}: {}", file, e),
        }
    }
}
