//! dml parser contains algorithm for parsing restq DML syntax into
//! a DML AST.
//!
use super::*;

fn returning<'a>() -> Parser<'a, char, Vec<Column>> {
    tag("returning=") * columns()
}

fn columns<'a>() -> Parser<'a, char, Vec<Column>> {
    list_fail(column(), sym(','))
}

/// product{product_id,created_by,created,is_active}?returning=product_id,name
pub fn insert<'a>() -> Parser<'a, char, Insert> {
    (table() - sym('{') + columns() - sym('}') + (sym('?') * returning()).opt())
        .map(|((into, columns), returning)| {
            Insert {
                into,
                columns,
                returning,
                source: Source::Values(vec![]),
            }
        })
}

fn column_value<'a>() -> Parser<'a, char, (Column, Value)> {
    column() - sym('=') + value()
}

fn column_values<'a>() -> Parser<'a, char, Vec<(Column, Value)>> {
    list_fail(column_value(), sym(','))
}

/// product{description="I'm the new description now",is_active=false}?product_id=1
pub fn update<'a>() -> Parser<'a, char, Update> {
    (table() - sym('{') + column_values() - sym('}')
        + (sym('?') * filter_expr()).opt())
    .map(|((table, column_values), condition)| {
        let (columns, values) = column_values.into_iter().unzip();
        Update {
            table,
            columns,
            values,
            condition,
        }
    })
}

///  product?product_id=1
pub fn delete<'a>() -> Parser<'a, char, Delete> {
    (table() + (sym('?') * filter_expr()).opt())
        .map(|(from, condition)| Delete { from, condition })
}

/// bulk delete
pub fn bulk_delete<'a>() -> Parser<'a, char, BulkDelete> {
    (table() - sym('{') + columns() - sym('}')).map(|(from, columns)| {
        BulkDelete {
            from,
            columns,
            values: vec![],
        }
    })
}

/// bulk update
pub fn bulk_update<'a>() -> Parser<'a, char, BulkUpdate> {
    (table() - sym('{') + columns() - sym('}')).map(|(table, columns)| {
        BulkUpdate {
            table,
            columns,
            values: vec![],
        }
    })
}
