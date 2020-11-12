//! DML stands for Data Manipulation Language
//! and this module contains the AST for DML operations
//! such as Insert, Delete, Update table.
mod dml_parser;

pub use dml_parser::*;

use crate::{
    ast::parser::*,
    ast::{Column, Expr, Select, Table, TableLookup, Value},
    Error,
};
use pom::parser::{sym, tag, Parser};
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

/// DELETE /product?product_id=1
#[derive(Debug, PartialEq, Clone)]
pub struct Delete {
    pub from: Table,
    pub condition: Option<Expr>,
}

/// DELETE /product{product_id}
/// 1
/// 2
/// 3
#[derive(Debug, PartialEq, Clone)]
pub struct BulkDelete {
    pub from: Table,
    pub columns: Vec<Column>,
    pub values: Vec<Vec<Value>>,
}

/// PATCH /product{description="I'm the new description now"}?product_id=1
#[derive(Debug, PartialEq, Clone)]
pub struct Update {
    pub table: Table,
    pub columns: Vec<Column>,
    pub values: Vec<Value>, // one value for each column
    pub condition: Option<Expr>,
}

/// PATCH /product{*product_id,name}
/// 1,go pro,1,go pro hero4
/// 2,shovel,2,slightly used shovel
#[derive(Debug, PartialEq, Clone)]
pub struct BulkUpdate {
    pub table: Table,
    pub columns: Vec<Column>,
    pub values: Vec<Vec<Value>>,
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
                .map(|(column, value)| sql::Assignment {
                    id: Into::into(column),
                    value: Into::into(value),
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
            Source::Select(select) => sql::SetExpr::Select(Box::new(
                select.into_sql_select(table_lookup)?,
            )),
            Source::Values(rows) => sql::SetExpr::Values(sql::Values(
                rows.iter()
                    .map(|record| {
                        record.iter().map(|v| Into::into(v)).collect()
                    })
                    .collect(),
            )),
            Source::Parameterized(params) => {
                println!("parameterized params: {:?}", params);
                sql::SetExpr::ParameterizedValue(params.to_owned())
            }
        };
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::BinaryOperation;
    use crate::ast::parser::utils::to_chars;
    use crate::ast::Operator;

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
            BulkDelete {
                from: Table {
                    name: "product".into()
                },
                columns: vec![
                    Column {
                        name: "name".into()
                    },
                    Column {
                        name: "is_active".into()
                    }
                ],
                values: vec![]
            }
        );
    }

    #[test]
    fn test_bulk_update() {
        let input = to_chars("product{name,is_active}");
        let ret = bulk_update().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            BulkUpdate {
                table: Table {
                    name: "product".into()
                },
                columns: vec![
                    Column {
                        name: "name".into()
                    },
                    Column {
                        name: "is_active".into()
                    }
                ],
                values: vec![]
            }
        );
    }
}
