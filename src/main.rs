use std::fs::File;
use std::io::Write;
use std::process::Command;
use sha2::{Sha256, Digest};

#[cfg(test)]
mod tests;

#[derive(Debug)]
enum Error {
    EmptyExpr,
    InconsistentParans,
}

// S -> S '*'
//    | S '+'
//    | (S)
//    | [a-z]*
//    | [A-Z]*
//    | [0-9]*
#[derive(Debug, PartialEq)]
enum Regex {
    Any(RegexRef),
    Many(RegexRef),
    Char(char),
    Multi(Vec<Regex>),
}

type RegexRef = usize;

struct Context<'a> {
    input: &'a str,
    offset: usize,
    arena: Vec<Regex>,
}

impl<'a> Context<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            offset: 0,
            arena: Vec::new(),
        }
    }

    fn current_char(&self) -> Option<char> {
        self.input[self.offset..].chars().next()
    }

    fn consume(&mut self, matcher: impl FnOnce(char) -> bool) -> bool {
        let Some(chr) = self.current_char() else {
            return false;
        };

        let matches = matcher(chr);
        self.offset += (matches as usize) * chr.len_utf8();
        matches
    }

    fn alloc(&mut self, regex: Regex) -> RegexRef {
        let offset = self.arena.len();
        self.arena.push(regex);
        offset
    }

    pub fn parse(&mut self) -> Result<Regex, Error> {
        let p1 = self.parse_single()?;
        match self.parse_single() {
            Ok(p2) => {
                let mut multi = vec![p1, p2];

                loop {
                    match self.parse_single() {
                        Err(Error::EmptyExpr) => break,
                        Err(err) => return Err(err),
                        Ok(expr) => multi.push(expr),
                    }
                }

                Ok(Regex::Multi(multi))
            }
            Err(Error::EmptyExpr) => Ok(p1),
            Err(err) => Err(err),
        }
    }

    pub fn parse_single(&mut self) -> Result<Regex, Error> {
        if self.current_char() == Some('(') {
            let expr = self.parse_param()?;

            if self.consume(|c| c == '*') {
                return Ok(Regex::Any(self.alloc(expr)));
            }

            if self.consume(|c| c == '+') {
                return Ok(Regex::Many(self.alloc(expr)));
            }

            return Ok(expr);
        }

        self.parse_char_or_any()
    }

    fn parse_param(&mut self) -> Result<Regex, Error> {
        if !self.consume(|c| c == '(') {
            return Err(Error::InconsistentParans);
        }
        let expr = self.parse()?;
        if !self.consume(|c| c == ')') {
            return Err(Error::InconsistentParans);
        }

        Ok(expr)
    }

    fn parse_char_or_any(&mut self) -> Result<Regex, Error> {
        if let Some(chr @ ('a'..='z' | 'A'..='Z' | '0'..='9')) = self.current_char() {
            self.offset += 1;

            if self.consume(|c| c == '*') {
                return Ok(Regex::Any(self.alloc(Regex::Char(chr))));
            }

            if self.consume(|c| c == '+') {
                return Ok(Regex::Many(self.alloc(Regex::Char(chr))));
            }

            return Ok(Regex::Char(chr));
        }

        Err(Error::EmptyExpr)
    }
}

fn recurse_gen_program(out: &mut String, ctx: &Context, regex: &Regex, depth: usize) -> usize {
    match regex {
        Regex::Any(regexref) => {
            let child_fn_idx = depth + 1;
            let func = format!("\
// any
#[inline(always)]
fn func_{depth}(input: &mut &str) -> bool {{
    loop {{
        let saved = *input;
        if !func_{child_fn_idx}(input) {{
            *input = saved;
            break;
        }}
    }}

    true
}}\n");
            out.push_str(&func);
            recurse_gen_program(out, ctx, &ctx.arena[*regexref], depth + 1)
        }
        Regex::Many(regexref) => {
            let child_fn_idx = depth + 1;
            let func = format!("\
// many
#[inline(always)]
fn func_{depth}(input: &mut &str) -> bool {{
    let mut did_a_match = false;
    loop {{
        let saved = *input;
        if !func_{child_fn_idx}(input) {{
            *input = saved;
            break;
        }}
        did_a_match = true;
    }}

    did_a_match
}}\n");
            out.push_str(&func);
            recurse_gen_program(out, ctx, &ctx.arena[*regexref], depth + 1)
        }
        Regex::Char(chr) => {
            let func = format!("\
// single char
#[inline(always)]
fn func_{depth}(input: &mut &str) -> bool {{
    let starts = input.starts_with('{chr}');
    *input = &input[(starts as usize)..];
    starts
}}\n");
            out.push_str(&func);
            depth + 1
        }
        Regex::Multi(regexs) => {
            let mut new_depth = depth + 1;
            let mut indices_to_call = Vec::with_capacity(regexs.len());
            for regex in regexs {
                indices_to_call.push(new_depth);
                new_depth = recurse_gen_program(out, ctx, regex, new_depth);
            }

            let mut funcs_to_call = String::new();
            for idx in indices_to_call {
                funcs_to_call.push_str(&format!("    \
    if !func_{idx}(input) {{
        *input = saved;
        return false;
    }}
"));
            }

            let func = format!("\
// multi
#[inline(always)]
fn func_{depth}(input: &mut &str) -> bool {{
    let saved = *input;
{funcs_to_call}
    true
}}\n");
            out.push_str(&func);
            new_depth + 1
        },
    }
}

fn run(program: &str, input: &str) {
    // Hash the content.
    let mut hasher = Sha256::new();
    hasher.update(program.as_bytes());
    let hash_result = hasher.finalize();
    let hash_hex = format!("{:x}", hash_result);

    let file_path = format!("/tmp/{}.rs", hash_hex);

    // Write the program to /tmp/{hash}.rs
    let mut file = File::create(&file_path).expect("Failed to create file");
    file.write_all(program.as_bytes()).expect("Failed to write to file");

    // Compile the program using rustc
    let output_binary = format!("/tmp/{}", hash_hex);
    let compile_status = Command::new("rustc")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_binary)
        .arg("-Copt-level=3")
        .arg("-Dwarnings")
        .status()
        .expect("Failed to compile the program");

    println!("Binary written to {output_binary:?}");

    if compile_status.success() {
        // Execute the compiled program.
        let execution_status = Command::new(output_binary)
            .arg(input)
            .status()
            .expect("Failed to execute the compiled program.");

        if execution_status.success() {
            eprintln!("Input string matched!");
        } else {
            eprintln!("Input failed matched.");
            std::process::exit(1);
        }
    } else {
        eprintln!("Compilation failed (should not happen).");
    }
}

fn gen_program(ctx: &Context, regex: &Regex) -> String {
    let mut rust_progam = "
fn main() {{
    let Some(input) = std::env::args().skip(1).next() else {{
        eprintln!(\"Program expects input string.\");
        std::process::exit(1);
    }};
    if !func_0(&mut (&input as &str)) {{
        std::process::exit(1);
    }}
}}".to_string();

    recurse_gen_program(&mut rust_progam, ctx, regex, 0);
    rust_progam
}

fn main() {
    let Some(regex) = std::env::args().nth(1) else {
        eprintln!("Program expects regex expression.");
        std::process::exit(1);
    };

    let Some(input) = std::env::args().nth(2) else {
        eprintln!("Program expects input string.");
        std::process::exit(1);
    };

    let mut ctx = Context::new(&regex);
    let regex = match ctx.parse() {
        Ok(regex) => {
            println!("Generated regex: {regex:?}.");
            regex
        },
        Err(err) => {
            eprintln!("Failed to compile regex: {err:?}.");
            std::process::exit(1);
        }
    };

    let program = gen_program(&ctx, &regex);
    run(&program, &input);
}
