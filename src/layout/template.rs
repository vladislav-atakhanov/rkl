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
        let names = args
            .iter()
            .try_fold(Vec::with_capacity(args.len()), |mut acc, expr| {
                let x = expr.atom()?;
                if !x.starts_with("$") {
                    return Err(format!("Argument should start from $, found {:?}", x));
                }
                if acc.contains(&x) {
                    return Err(format!("Argument {:?} already defined", x));
                }
                acc.push(x);
                Ok(acc)
            })?;

        templates.insert(*x, Template(names, value.clone().clone()));
        Ok(())
    })?;
    Ok(templates)
}

pub fn expand<'a>(expr: &Expr<'a>, templates: &Templates<'a>) -> Expr<'a> {
    let List(list) = expr else {
        return expr.clone();
    };
    let Some(Atom(name)) = list.first() else {
        return List(list.iter().map(|e| expand(e, templates)).collect());
    };
    let Some(template) = templates.get(name) else {
        return List(list.iter().map(|e| expand(e, templates)).collect());
    };
    let args = &list[1..];
    let mut env = HashMap::new();
    let Template(params, body) = template;

    if params.is_empty() {
        return body.clone();
    }
    let (regular_params, extra_param) = params.split_at(params.len() - 1);
    for (param, arg) in regular_params.iter().zip(args.iter()) {
        env.insert(*param, expand(arg, templates));
    }
    let extra_args = &args[regular_params.len()..];
    let extra_expr_list: Vec<Expr> = extra_args.iter().map(|e| expand(e, templates)).collect();
    env.insert(
        *extra_param.first().unwrap(),
        if extra_expr_list.len() == 1 {
            extra_expr_list[0].clone()
        } else {
            List(extra_expr_list)
        },
    );
    return substitute(&body, &env, templates);
}

fn substitute<'a>(
    expr: &Expr<'a>,
    env: &HashMap<&'a str, Expr<'a>>,
    templates: &Templates<'a>,
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
        let expr = s_expression::from_str("(a ($b $c) $c)").unwrap();
        let list = expr.list().unwrap();
        let templates = deftemplate(list.clone()).unwrap();

        assert_eq!(
            templates,
            HashMap::from([("a", Template(vec!["$b", "$c"], Atom("$c")))]),
        );
    }

    fn assert<'a>(input: &'a str, output: &'a str, templates: &'a Templates<'a>) {
        let input = s_expression::from_str(input).unwrap();
        let output = s_expression::from_str(output).unwrap();
        assert_eq!(expand(&input, templates).to_string(), output.to_string())
    }

    #[test]
    fn apply_test() {
        let expr = s_expression::from_str("(a ($b $c) $c)").unwrap();
        let list = expr.list().unwrap();
        let templates = &deftemplate(list.clone()).unwrap();

        assert("(a 1 1)", "1", templates);
    }

    #[test]
    fn apply_template_args() {
        let expr = s_expression::from_str("(a ($b $c) $c)").unwrap();
        let list = expr.list().unwrap();
        let templates = &deftemplate(list.clone()).unwrap();

        assert("(a b c d)", "(c d)", templates);
    }

    #[test]
    fn args_duplicates() {
        let expr = s_expression::from_str("(a ($a $a) $a)").unwrap();
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
