use crate::{lints::VARIABLE_SHADOWING, ELM};

use aspen::{
    tree_sitter::{Node, Query, QueryCursor, Range},
    Context, Occurrence,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref QUERY: Query = Query::new(
        *ELM,
        r#"
            [
                (function_declaration_left (lower_case_identifier))
            ] @def
            "#
    )
    .unwrap();
}

pub fn validate<'a>(node: Node, ctx: &Option<Context<'a>>, src: &[u8]) -> Vec<Occurrence> {
    let mut query_cursor = QueryCursor::new();

    let ctx = if let Some(c) = ctx { c } else { return vec![] };

    query_cursor
        .matches(&QUERY, node, |_n: Node| std::iter::empty())
        .flat_map(|m| m.captures)
        .flat_map(|capture| {
            let at = capture.node.range();
            let text = capture.node.utf8_text(src).unwrap();
            let scope_stack = ctx.scope_stack_of(capture.node).unwrap();
            let mut shadowed_range: Option<Range> = None;
            for scope in scope_stack.skip(1) {
                for local_def in scope.borrow().local_defs.iter() {
                    if local_def.borrow().name == text {
                        shadowed_range = Some(local_def.borrow().def_range);
                    }
                }
            }
            if shadowed_range.is_none() {
                return None;
            }
            let (s_row, s_col) = {
                let s = shadowed_range.unwrap();
                (s.start_point.row + 1, s.start_point.column + 1)
            };
            let message = format!(
                "Shadowing `{}`, defined on line {}, col {}",
                text, s_row, s_col
            );
            Some(VARIABLE_SHADOWING.raise(at, message))
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use crate::ELM;

    use aspen::Linter;

    fn linter() -> Linter {
        Linter::new(*ELM)
            .validator(super::validate)
            .scopes(include_str!("../scopes.scm"))
            .comment_str("--")
    }

    #[test]
    fn trivial() {
        linter().test(
            r#"
            f = 2 + 1
            g =
                let
                    a = 2
                    f = 2
                 -- ^ Shadowing `f`, defined on line 1, col 1
                in
                    t + f
            "#,
        )
    }
}
