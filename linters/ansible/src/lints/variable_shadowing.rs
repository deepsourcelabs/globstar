use crate::{lints::defs::VARIABLE_SHADOWING, YAML};

use aspen::{
    tree_sitter::{Node, Query, QueryCursor},
    Context, Occurrence,
};
use once_cell::sync::Lazy;

static QUERY: Lazy<Query> = Lazy::new(|| {
    Query::new(
        *YAML,
        r#"
        (block_node) @raise
        "#,
    )
    .unwrap()
});

pub fn validate<'a>(node: Node, ctx: &Option<Context<'a>>, src: &[u8]) -> Vec<Occurrence> {
    ctx.as_ref().map_or(Vec::new(), |ctx| {
        QueryCursor::new()
            .matches(&QUERY, node, src)
            .flat_map(|m| m.captures)
            .flat_map(|capture| {
                let scope = ctx.scope_of(capture.node).unwrap();
                let local_defs = &scope.borrow().local_defs;
                let (mut duplicate_range, mut key_text) = (None, None);
                for i in 0..local_defs.len() {
                    for j in 0..i {
                        if local_defs[i].borrow().name == local_defs[j].borrow().name {
                            key_text = Some(local_defs[j].borrow().name);
                            duplicate_range = Some(local_defs[i].borrow().def_range);
                        }
                    }
                }

                if duplicate_range.is_none() {
                    return None;
                }

                let at = duplicate_range.unwrap();
                let (s_row, s_col) = { (at.start_point.row + 1, at.start_point.column + 1) };

                let message = format!(
                    "Duplicate key `{}`, already defined on line {}, col {}",
                    key_text.unwrap(),
                    s_row,
                    s_col
                );
                Some(VARIABLE_SHADOWING.raise(at, message))
            })
            .collect::<Vec<_>>()
    })
}

#[cfg(test)]
mod tests {
    use crate::YAML;

    use aspen::Linter;

    fn linter() -> Linter {
        Linter::new(*YAML)
            .validator(super::validate)
            .scopes(crate::SCOPES)
            .comment_str("#")
    }

    #[test]
    fn trivial() {
        linter().test(
            r#"
            ---
            product:
                - sku         : BL394D
                  sku         : BL394D
                # ^^^ Duplicate key `sku`, already defined on line 4, col 7
            "#,
        )
    }
}
