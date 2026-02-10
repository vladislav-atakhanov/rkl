use std::collections::HashSet;

use s_expression::Expr::{self, *};

pub fn unwrap<'a>(expr: &'a Expr<'a>, ignore: Option<&HashSet<&str>>) -> Expr<'a> {
    let List(list) = expr else {
        return expr.clone();
    };
    List(
        list.iter()
            .flat_map(|item| {
                let List(list) = item else {
                    return vec![unwrap(item, ignore)];
                };
                let [Atom(name), inner @ ..] = list.as_slice() else {
                    return vec![unwrap(item, ignore)];
                };
                match *name {
                    "unwrap" => inner.iter().map(|arg| unwrap(arg, ignore)).collect(),
                    _ if ignore.is_some_and(|set| set.contains(name)) => vec![],
                    _ => vec![unwrap(item, ignore)],
                }
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert(input: &str, output: &str, ignore: Option<&HashSet<&str>>) {
        let input = s_expression::from_str(input).unwrap();
        let output = s_expression::from_str(output).unwrap();
        assert_eq!(unwrap(&input, ignore).to_string(), output.to_string())
    }

    #[test]
    fn unwrap_list() {
        assert("(outer (unwrap (arg a)))", "(outer (arg a))", None);
    }

    #[test]
    fn unwrap_atoms() {
        assert("(outer (unwrap a b c d) e f)", "(outer a b c d e f)", None);
    }

    #[test]
    fn real_unwrap() {
        assert(
            r#"(
                (defsrc a b c d)
                (unwrap
                    (defalias a b)
                    (deflayer c (unwrap d e))
                )
            )"#,
            r#"(
                (defsrc a b c d)
                (defalias a b)
                (deflayer c d e)
            )"#,
            None,
        );
    }

    #[test]
    fn ignore() {
        assert(
            "(outer (ignore some) (unwrap a b c d))",
            "(outer a b c d)",
            Some(&HashSet::from(["ignore"])),
        );
    }
}
