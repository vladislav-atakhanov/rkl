use std::collections::HashSet;

use s_expression::Expr::{self, *};

pub fn unwrap<'a>(expr: &'a Expr<'a>, ignore: Option<&HashSet<&str>>) -> Expr<'a> {
    let List(list) = expr else {
        return expr.clone();
    };
    if ignore.is_some_and(|set| matches!(list.first(), Some(Atom(name)) if set.contains(name))) {
        return List(vec![]);
    }
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
                    "unwrap" => inner
                        .iter()
                        .flat_map(|arg| match unwrap(arg, ignore) {
                            List(xs) => xs,
                            x => vec![x],
                        })
                        .collect(),
                    _ => match unwrap(item, ignore) {
                        List(xs) => xs,
                        x => vec![x],
                    },
                }
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwrap_list() {
        let expr = s_expression::from_str("(outer (unwrap (arg a)))").unwrap();
        assert_eq!(unwrap(&expr, None).to_string(), "(outer arg a)".to_string())
    }

    #[test]
    fn unwrap_atoms() {
        let expr = s_expression::from_str("(outer (unwrap a b c d))").unwrap();
        assert_eq!(
            unwrap(&expr, None).to_string(),
            "(outer a b c d)".to_string()
        )
    }
    #[test]
    fn unwrap_ignore() {
        let expr = s_expression::from_str("(outer (unwrap a b c d) (ignore some))").unwrap();
        assert_eq!(
            unwrap(&expr, Some(&HashSet::from(["ignore"]))).to_string(),
            "(outer a b c d)".to_string()
        )
    }
    #[test]
    fn unwrap_ignore2() {
        let expr = s_expression::from_str("(ignore (unwrap a b c d))").unwrap();
        assert_eq!(
            unwrap(&expr, Some(&HashSet::from(["ignore"]))).to_string(),
            "()".to_string()
        )
    }
}
