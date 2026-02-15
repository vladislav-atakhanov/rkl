use std::collections::HashSet;

use super::template;
use super::unwrap::unwrap;
use s_expression::Expr;

pub fn preprocess<'a>(expr: &Expr<'a>) -> Result<Expr<'a>, String> {
    let mut templates = template::Templates::new();
    let root = expr.list()?;

    root.iter().try_for_each(|item| -> Result<(), String> {
        let lst = item.list()?;
        let name = lst.first().ok_or("Not found".to_string())?;
        if name.atom()? == "deftemplate" {
            templates.extend(template::deftemplate(lst[1..].to_vec())?)
        }
        Ok(())
    })?;
    let root = template::expand(expr, &templates);
    let root = unwrap(&root, Some(&HashSet::from(["deftemplate"])));
    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert<'a, F>(input: &'a str, output: &'a str, f: F)
    where
        F: Fn(&Expr<'a>) -> Expr<'a>,
    {
        let input = s_expression::from_str(input).unwrap();
        let output = s_expression::from_str(output).unwrap();
        assert_eq!(f(&input).to_string(), output.to_string())
    }

    #[test]
    fn preprocess_aliases() {
        assert(
            r#"(
                (deftemplate app ($x) (multi meta $x))
                (defalias
                    a0 (app 0)
                    a1 (app 1)
                    a2 (app 2)
                )
            )"#,
            r#"(
                (defalias
                    a0 (multi meta 0)
                    a1 (multi meta 1)
                    a2 (multi meta 2)
                )
            )"#,
            |e| preprocess(e).unwrap(),
        );
    }
}
