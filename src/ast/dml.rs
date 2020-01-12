use crate::{
    ast::{
        BinaryOperation,
        Column,
        Expr,
        Operator,
        Select,
        Table,
        TableLookup,
        Value,
    },
    parser::*,
    to_chars,
    Error,
};
use pom::parser::{
    call,
    end,
    is_a,
    one_of,
    sym,
    tag,
    Parser,
};
use sql_ast::ast as sql;

#[derive(Debug, PartialEq, Clone)]
pub struct Insert {
    pub into: Table,
    pub columns: Vec<Column>,
    pub source: Source,
    pub returning: Option<Vec<Column>>,
}

/// Insert can get data from a set of values
/// or from a select statement
#[derive(Debug, PartialEq, Clone)]
pub enum Source {
    Select(Select),
    Values(Vec<Vec<Value>>),
    Parameterized(Vec<usize>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Delete {
    pub from: Table,
    pub condition: Option<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Update {
    pub table: Table,
    pub columns: Vec<Column>,
    pub values: Vec<Value>, // one value for each column
    pub condition: Option<Expr>,
}

impl Insert {
    pub fn into_sql_statement(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<sql::Statement, Error> {
        Ok(sql::Statement::Insert {
            table_name: Into::into(&self.into),
            columns: self.columns.iter().map(|c| Into::into(c)).collect(),
            source: Box::new(sql::Query {
                ctes: vec![],
                body: self.source.into_sql_setexpr(table_lookup)?,
                order_by: vec![],
                limit: None,
                offset: None,
                fetch: None,
            }),
        })
    }
}

impl Into<sql::Statement> for &Delete {
    fn into(self) -> sql::Statement {
        sql::Statement::Delete {
            table_name: Into::into(&self.from),
            selection: self.condition.as_ref().map(|expr| Into::into(expr)),
        }
    }
}

impl Into<sql::Statement> for &Update {
    fn into(self) -> sql::Statement {
        sql::Statement::Update {
            table_name: Into::into(&self.table),
            assignments: self
                .columns
                .iter()
                .zip(self.values.iter())
                .map(|(column, value)| {
                    sql::Assignment {
                        id: Into::into(column),
                        value: Into::into(value),
                    }
                })
                .collect(),
            selection: self.condition.as_ref().map(|expr| Into::into(expr)),
        }
    }
}

impl Source {
    fn into_sql_setexpr(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<sql::SetExpr, Error> {
        let ret = match self {
            Source::Select(select) => {
                sql::SetExpr::Select(Box::new(
                    select.into_sql_select(table_lookup)?,
                ))
            }
            Source::Values(rows) => {
                sql::SetExpr::Values(sql::Values(
                    rows.iter()
                        .map(|record| {
                            record.iter().map(|v| Into::into(v)).collect()
                        })
                        .collect(),
                ))
            }
            Source::Parameterized(params) => {
                println!("parameterized params: {:?}", params);
                sql::SetExpr::ParameterizedValue(params.to_owned())
            }
        };
        Ok(ret)
    }
}

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

fn bulk_delete<'a>() -> Parser<'a, char, Delete> {
    (table() - sym('{') + columns() - sym('}')).map(|(from, columns)| {
        let mut columns_iter = columns.into_iter();
        let first = columns_iter.next();
        let condition = first.map(|first| {
            let first_condition =
                Expr::BinaryOperation(Box::new(BinaryOperation {
                    left: Expr::Column(first),
                    operator: Operator::Eq,
                    right: Expr::Value(Value::String("$1".to_string())),
                }));
            columns_iter.enumerate().fold(
                first_condition,
                |condition, (i, column)| {
                    Expr::BinaryOperation(Box::new(BinaryOperation {
                        left: condition,
                        operator: Operator::And,
                        right: Expr::BinaryOperation(Box::new(
                            BinaryOperation {
                                left: Expr::Column(column),
                                operator: Operator::Eq,
                                right: Expr::Value(Value::String(format!(
                                    "${}",
                                    i + 2
                                ))),
                            },
                        )),
                    }))
                },
            )
        });
        Delete { from, condition }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let input = to_chars(
            "product{product_id,created_by,created,is_active}?returning=product_id,name\n\
            1,1,2019-10-10T10:10:10.122,true
            ",
        );
        let ret = insert().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        let statement: sql::Statement =
            ret.into_sql_statement(None).expect("must not fail");
        assert_eq!(
            statement.to_string(),
            "INSERT INTO product (product_id, created_by, created, is_active) VALUES "
        );
        assert_eq!(
            ret,
            Insert {
                into: Table {
                    name: "product".into()
                },
                columns: vec![
                    Column {
                        name: "product_id".into()
                    },
                    Column {
                        name: "created_by".into()
                    },
                    Column {
                        name: "created".into()
                    },
                    Column {
                        name: "is_active".into()
                    },
                ],
                source: Source::Values(vec![]),
                returning: Some(vec![
                    Column {
                        name: "product_id".into()
                    },
                    Column {
                        name: "name".into()
                    },
                ])
            }
        );
    }

    #[test]
    fn test_update() {
        let input = to_chars(
            r#"product{description="I'm the new description now",is_active=false}?product_id=1"#,
        );
        let ret = update().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        let statement: sql::Statement = Into::into(&ret);
        assert_eq!(
            statement.to_string(),
            r#"UPDATE product SET description = 'I''m the new description now', is_active = false WHERE product_id = 1"#
        );
        assert_eq!(
            ret,
            Update {
                table: Table {
                    name: "product".into()
                },
                columns: vec![
                    Column {
                        name: "description".into(),
                    },
                    Column {
                        name: "is_active".into()
                    },
                ],
                values: vec![
                    Value::String("I'm the new description now".into(),),
                    Value::Bool(false,),
                ],
                condition: Some(Expr::BinaryOperation(Box::new(
                    BinaryOperation {
                        left: Expr::Column(Column {
                            name: "product_id".into()
                        },),
                        operator: Operator::Eq,
                        right: Expr::Value(Value::Number(1.0))
                    }
                )))
            }
        )
    }
    #[test]
    fn test_delete() {
        let input = to_chars(r#"product?product_id=1"#);
        let ret = delete().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        let statement: sql::Statement = Into::into(&ret);
        assert_eq!(
            statement.to_string(),
            "DELETE FROM product WHERE product_id = 1"
        );
        assert_eq!(
            ret,
            Delete {
                from: Table {
                    name: "product".into()
                },
                condition: Some(Expr::BinaryOperation(Box::new(
                    BinaryOperation {
                        left: Expr::Column(Column {
                            name: "product_id".into()
                        },),
                        operator: Operator::Eq,
                        right: Expr::Value(Value::Number(1.0))
                    }
                )))
            }
        );
    }

    #[test]
    fn test_bulk_delete() {
        let input = to_chars("product{name,is_active}");
        let ret = bulk_delete().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            Delete {
                from: Table {
                    name: "product".into()
                },
                condition: Some(Expr::BinaryOperation(Box::new(
                    BinaryOperation {
                        left: Expr::BinaryOperation(Box::new(
                            BinaryOperation {
                                left: Expr::Column(Column {
                                    name: "name".into()
                                }),
                                operator: Operator::Eq,
                                right: Expr::Value(Value::String(
                                    "$1".to_string()
                                ))
                            }
                        )),
                        operator: Operator::And,
                        right: Expr::BinaryOperation(Box::new(
                            BinaryOperation {
                                left: Expr::Column(Column {
                                    name: "is_active".into()
                                }),
                                operator: Operator::Eq,
                                right: Expr::Value(Value::String(
                                    "$2".to_string()
                                ))
                            }
                        )),
                    }
                )))
            }
        );
    }
}
