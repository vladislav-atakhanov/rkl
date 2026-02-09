mod parse;
mod tokenize;

pub use parse::Expr;

pub fn from_str<'a>(input: &'a str) -> Result<parse::Expr<'a>, ()> {
    let mut tokens = tokenize::tokenize(input);
    return parse::parse(&mut tokens);
}
