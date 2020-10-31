use super::*;
use crate::{
    ast::Table,
    data_type::data_type,
    data_value,
    parser::{utils::end_or_ln, *},
};
use pom::parser::{sym, Parser};

pub(crate) fn column_attribute<'a>() -> Parser<'a, char, ColumnAttribute> {
    sym('*').map(|_| ColumnAttribute::Primary)
        | sym('&').map(|_| ColumnAttribute::Unique)
        | sym('@').map(|_| ColumnAttribute::Index)
}

pub(crate) fn column_attributes<'a>() -> Parser<'a, char, Vec<ColumnAttribute>>
{
    column_attribute().repeat(1..)
}

/// foreign = "(", table, ")"
pub(crate) fn foreign<'a>() -> Parser<'a, char, Table> {
    sym('(') * table() - sym(')')
}

/// parse column definition with the format
/// column_def  = [column_attribute], column, [foreign] [":" data_type];
/// example:
///     &*product_id(product):u32
pub(crate) fn column_def<'a>() -> Parser<'a, char, ColumnDef> {
    ((column_attributes().opt() + column() + foreign().opt() - sym(':')
        + data_type_def())
    .map(|(((attributes, column), foreign), data_type)| ColumnDef {
        column,
        attributes,
        data_type_def: data_type,
        foreign,
    }))
    .name("column_def")
}

pub(crate) fn column_def_list<'a>() -> Parser<'a, char, Vec<ColumnDef>> {
    list_fail(column_def(), sym(',')).name("column_def_list")
}

pub(crate) fn enclosed_column_def_list<'a>() -> Parser<'a, char, Vec<ColumnDef>>
{
    sym('(') * column_def_list() - sym(')')
        | sym('{') * column_def_list() - sym('}')
}

/// example:
///   product{*product_id:s32,@name:text,description:text,updated:utc,created_by(users):u32,@is_active:bool}
///
///  or
///
///   product(*product_id:s32,@name:text,description:text,updated:utc,created_by(users):u32,@is_active:bool)
///
/// Note: that braces `{}` are invalid when used in the path part, but can be valid when used in
/// query part.
/// So it is safe to use the parenthesis `()` when used in actual rest api request.
pub fn table_def<'a>() -> Parser<'a, char, TableDef> {
    (table() + enclosed_column_def_list() - end_or_ln())
        .map(|(table, columns)| TableDef { table, columns })
}

/// data_type_def = data_type ["?"] "(" value ")"
/// example:
///     u32?(0.0)
pub fn data_type_def<'a>() -> Parser<'a, char, DataTypeDef> {
    (data_type() + sym('?').opt() + (sym('(') * value() - sym(')')).opt())
        .map(|((data_type, optional), default_value)| {
            let default = default_value
                .map(|dv| data_value::cast_data_value(&dv, &data_type));

            DataTypeDef {
                data_type,
                is_optional: if let Some(_) = optional { true } else { false },
                default,
            }
        })
        .name("data_type_def")
}

pub fn drop_table<'a>() -> Parser<'a, char, DropTable> {
    (sym('-') * table() - end_or_ln()).map(|table| DropTable { table })
}

fn drop_column<'a>() -> Parser<'a, char, AlterOperation> {
    (sym('-') * column()).map(AlterOperation::DropColumn)
}

fn add_column<'a>() -> Parser<'a, char, AlterOperation> {
    (sym('+') * column_def()).map(AlterOperation::AddColumn)
}

fn alter_column<'a>() -> Parser<'a, char, AlterOperation> {
    (column() - sym('=') + column_def()).map(|(column, column_def)| {
        AlterOperation::AlterColumn(column, column_def)
    })
}

fn alter_operation<'a>() -> Parser<'a, char, AlterOperation> {
    alter_column() | drop_column() | add_column()
}

fn alter_operations<'a>() -> Parser<'a, char, Vec<AlterOperation>> {
    sym('(') * list_fail(alter_operation(), sym(',')) - sym(')')
        | sym('{') * list_fail(alter_operation(), sym(',')) - sym('}')
}

pub fn alter_table<'a>() -> Parser<'a, char, AlterTable> {
    (table() + alter_operations() - end_or_ln()).map(
        |(table, alter_operations)| AlterTable {
            table,
            alter_operations,
        },
    )
}
