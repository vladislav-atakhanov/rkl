use s_expression::Expr::{self, *};
use std::collections::HashMap;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Template<'a>(Vec<&'a str>, Expr<'a>);

pub type Templates<'a> = HashMap<&'a str, Template<'a>>;

pub fn deftemplate<'a>(list: Vec<Expr<'a>>) -> Result<Templates<'a>, String> {
    let mut templates: HashMap<&'a str, Template<'a>> = HashMap::new();

    list.chunks(3).try_for_each(|r| {
        let [Atom(x), List(args), value] = r else {
            return Err(format!("SyntaxError: {:?}", r));
        };
        let mut names: Vec<&str> = args
            .iter()
            .filter_map(|a| if let Atom(x) = a { Some(*x) } else { None })
            .collect();
        names.dedup();
        if names.len() == args.len() {
            templates.insert(*x, Template(names, value.clone().clone()));
            Ok(())
        } else {
            Err(format!("Expected unique atoms, found {:?}", args))
        }
    })?;
    Ok(templates)
}

pub fn expand<'a>(expr: &Expr<'a>, templates: &'a Templates<'a>) -> Expr<'a> {
    match expr {
        Expr::Atom(a) => Expr::Atom(a),

        Expr::List(list) => {
            if let Some(Expr::Atom(name)) = list.first() {
                if let Some(template) = templates.get(name) {
                    // создаём окружение параметр -> аргумент
                    let args = &list[1..];
                    let mut env = std::collections::HashMap::new();
                    let Template(params, body) = template;
                    for (param, arg) in params.iter().zip(args.iter()) {
                        env.insert(*param, expand(arg, templates));
                    }
                    // рекурсивно подставляем тело шаблона
                    return substitute(&body, &env, templates);
                }
            }

            // обычный список, рекурсивно раскрываем элементы
            Expr::List(list.iter().map(|e| expand(e, templates)).collect())
        }
    }
}

fn substitute<'a>(
    expr: &Expr<'a>,
    env: &HashMap<&'a str, Expr<'a>>,
    templates: &'a Templates<'a>,
) -> Expr<'a> {
    match expr {
        Expr::Atom(a) => env.get(a).cloned().unwrap_or_else(|| Expr::Atom(a)),

        Expr::List(list) => {
            let expanded_list: Vec<Expr> =
                list.iter().map(|e| substitute(e, env, templates)).collect();
            expand(&Expr::List(expanded_list), templates)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let expr = s_expression::from_str("(a (b c) c)").unwrap();
        let list = expr.list().unwrap();
        let templates = deftemplate(list.clone()).unwrap();

        assert_eq!(
            templates,
            HashMap::from([("a", Template(vec!["b", "c"], Atom("c")))]),
        );
    }

    #[test]
    fn apply_test() {
        let expr = s_expression::from_str("(a (b c) c)").unwrap();
        let list = expr.list().unwrap();
        let templates = &deftemplate(list.clone()).unwrap();

        let input = &s_expression::from_str("(a 1 1)").unwrap();
        assert_eq!(expand(input, templates).to_string(), "1".to_string())
    }

    #[test]
    fn args_duplicates() {
        let expr = s_expression::from_str("(a (a a) a)").unwrap();
        let list = expr.list().unwrap();
        assert!(deftemplate(list.clone()).is_err())
    }

    #[test]
    fn args_should_be_list() {
        let expr = s_expression::from_str("(a arg a)").unwrap();
        let list = expr.list().unwrap();
        assert!(deftemplate(list.clone()).is_err())
    }
}
