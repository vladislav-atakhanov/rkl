#[derive(Debug)]
pub enum Expr<'a> {
    Atom(&'a str),
    List(Vec<Expr<'a>>),
}

fn tokenize(input: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut start = None;

    for (i, c) in input.char_indices() {
        match c {
            '(' | ')' => {
                if let Some(s) = start {
                    tokens.push(&input[s..i]);
                    start = None;
                }
                tokens.push(&input[i..i + c.len_utf8()]);
            }
            c if c.is_whitespace() => {
                if let Some(s) = start {
                    tokens.push(&input[s..i]);
                    start = None;
                }
            }
            _ => {
                if start.is_none() {
                    start = Some(i);
                }
            }
        }
    }

    if let Some(s) = start {
        tokens.push(&input[s..]);
    }

    tokens
}

fn parse<'a>(tokens: &mut Vec<&'a str>) -> Result<Expr<'a>, Error> {
    let token = tokens.remove(0);

    match token {
        "(" => {
            let mut list = Vec::new();
            while tokens[0] != ")" {
                match parse(tokens) {
                    Ok(t) => list.push(t),
                    Err(error) => return Err(error),
                }
            }
            tokens.remove(0); // ')'
            Ok(Expr::List(list))
        }
        ")" => Err(Error {}),
        _ => Ok(Expr::Atom(token)),
    }
}

pub struct Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Some")
    }
}

pub fn from_str<'a>(input: &'a str) -> Result<Expr<'a>, Error> {
    let mut tokens = tokenize(input);
    return parse(&mut tokens);
}
