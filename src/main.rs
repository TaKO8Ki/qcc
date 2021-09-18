use std::env;

fn main() {
    let arg = env::args().nth(1).unwrap();

    let mut chars = arg.chars();

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    let mut expr = Vec::new();

    let mut is_number = false;
    let mut number = vec![];
    loop {
        if let Some(c) = chars.next() {
            if c.is_digit(10) {
                is_number = true;
                number.push(c)
            } else if is_number {
                expr.push(number.iter().collect::<String>());
                expr.push(c.to_string());
                number.clear();
            } else {
                expr.push(c.to_string());
            }
        } else {
            expr.push(number.iter().collect::<String>());
            break;
        }
    }

    let mut expr_iter = expr.iter();
    println!("  mov rax, {}", expr_iter.next().unwrap());

    while let Some(c) = expr_iter.next() {
        let next = expr_iter.next().unwrap();

        if c.as_str() == "+" {
            println!("  add rax, {}", next);
            continue;
        }

        if c.as_str() == "-" {
            println!("  sub rax, {}", next);
            continue;
        }

        panic!("not expected value: {}", c);
    }

    println!("  ret");
}
