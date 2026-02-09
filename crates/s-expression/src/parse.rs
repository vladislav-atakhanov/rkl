#[derive(Debug, Clone)]
pub enum Expr<'a> {
    Atom(&'a str),
    List(Vec<Expr<'a>>),
}
impl Default for Expr<'static> {
    fn default() -> Self {
        Self::List(vec![])
    }
}

impl<'a> Expr<'a> {
    pub fn list(&'a self) -> Result<&'a Vec<Expr<'a>>, String> {
        match self {
            Expr::List(list) => Ok(list),
            _ => return Err(format!("Expected list, found {:?}", self)),
        }
    }
    pub fn atom(&'a self) -> Result<&'a str, String> {
        match self {
            Expr::Atom(s) => Ok(*s),
            _ => return Err(format!("Expected atom, found {:?}", self)),
        }
    }
}

pub fn parse<'a>(tokens: &mut Vec<&'a str>) -> Result<Expr<'a>, ()> {
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
        ")" => Err(()),
        _ => Ok(Expr::Atom(token)),
    }
}
