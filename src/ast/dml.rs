//! DML stands for Data Manipulation Language
//! and this module contains the AST for DML operations
//! such as Insert, Delete, Update table.
mod dml_parser;

use crate::{
    ast::{
        BinaryOperation, ColumnName, Expr, Operator, Select, TableDef,
        TableLookup, TableName, Value,
    },
    parser::{column, list_fail, table, value},
    ColumnDef, Error,
};
pub use dml_parser::{bulk_delete, bulk_update, delete, insert, update};
use pom::parser::tag;
use serde::{Deserialize, Serialize};
use sql_ast::ast as sql;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Insert {
    pub into: TableName,
    pub columns: Vec<ColumnName>,
    pub source: Source,
    pub returning: Option<Vec<ColumnName>>,
}

/// Insert can get data from a set of values
/// or from a select statement
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Source {
    Select(Select),
    Values(Vec<Vec<Value>>),
    Parameterized(Vec<usize>),
}

/// DELETE /product?product_id=1
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Delete {
    pub from: TableName,
    pub condition: Option<Expr>,
}

/// DELETE /product{product_id}
/// 1
/// 2
/// 3
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BulkDelete {
    pub from: TableName,
    pub columns: Vec<ColumnName>,
    pub values: Vec<Vec<Value>>,
}

/// PATCH /product{description="I'm the new description now"}?product_id=1
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Update {
    pub table: TableName,
    pub columns: Vec<ColumnName>,
    pub values: Vec<Value>, // one value for each column
    pub condition: Option<Expr>,
}

/// PATCH /product{*product_id,name}
/// 1,go pro,1,go pro hero4
/// 2,shovel,2,slightly used shovel
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BulkUpdate {
    pub table: TableName,
    pub columns: Vec<ColumnName>,
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

impl Delete {
    pub fn into_sql_statement(&self) -> Result<sql::Statement, Error> {
        Ok(sql::Statement::Delete {
            table_name: Into::into(&self.from),
            selection: self.condition.as_ref().map(|expr| Into::into(expr)),
        })
    }
}

impl Update {
    pub fn into_sql_statement(&self) -> Result<sql::Statement, Error> {
        Ok(sql::Statement::Update {
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
        })
    }
}

/// a common code for building filter from columns old value and primary columns
fn build_filter_from_columns(
    columns: &[ColumnName],
    old_values: &[&Value],
    primary_columns: &[&ColumnDef],
) -> Option<Expr> {
    let pk_values: Vec<&Value> = primary_columns
        .iter()
        .filter_map(|pk| {
            columns.iter().zip(old_values.iter()).find_map(
                |(col, old_value)| {
                    if col == &pk.column {
                        Some(*old_value)
                    } else {
                        None
                    }
                },
            )
        })
        .collect();

    let pk_column_values: Vec<(&ColumnDef, &Value)> = primary_columns
        .into_iter()
        .zip(pk_values.into_iter())
        .map(|(column_def, value)| (*column_def, value))
        .collect();

    if let Some((column0, value0)) = pk_column_values.first() {
        let mut filter0 = Expr::BinaryOperation(Box::new(BinaryOperation {
            left: Expr::Column(column0.column.clone()),
            operator: Operator::Eq,
            right: Expr::Value((*value0).clone()),
        }));
        for (column, value) in pk_column_values.iter().skip(1) {
            let next_filter =
                Expr::BinaryOperation(Box::new(BinaryOperation {
                    left: Expr::Column(column.column.clone()),
                    operator: Operator::Eq,
                    right: Expr::Value((*value).clone()),
                }));

            filter0 = Expr::BinaryOperation(Box::new(BinaryOperation {
                left: filter0,
                operator: Operator::And,
                right: next_filter,
            }));
        }
        Some(filter0)
    } else {
        None
    }
}

impl BulkUpdate {
    /// convert bulk update into sql statements
    pub fn into_sql_statements(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<Vec<sql::Statement>, Error> {
        let table_def = table_lookup
            .expect("must have a table lookup")
            .get_table_def(&self.table.name)
            .expect("must have a table_def");
        let updates = self.into_updates(table_def)?;
        Ok(updates
            .into_iter()
            .map(|update| update.into_sql_statement().expect("must convert"))
            .collect())
    }

    /// convert BulkUpdate into multiple Update AST
    fn into_updates(&self, table_def: &TableDef) -> Result<Vec<Update>, Error> {
        let columns_len = self.columns.len();

        let updates = self
            .values
            .iter()
            .map(|row| {
                let old_values: Vec<&Value> =
                    row.iter().take(columns_len).collect();

                let new_values: Vec<&Value> =
                    row.iter().skip(columns_len).collect();

                assert_eq!(
                    old_values.len(),
                    new_values.len(),
                    "must the same number of records"
                );

                // column and values that are changed
                let column_new_values: Vec<(ColumnName, Value)> = self
                    .columns
                    .iter()
                    .zip(old_values.clone().iter().zip(new_values.iter()))
                    .filter_map(|(column, (old_value, new_value))| {
                        if old_value != new_value {
                            Some((column.clone(), (*new_value).clone()))
                        } else {
                            None
                        }
                    })
                    .collect();

                let (columns, new_values): (Vec<ColumnName>, Vec<Value>) =
                    column_new_values.into_iter().unzip();

                Update {
                    table: self.table.clone(),
                    columns,
                    values: new_values,
                    condition: build_filter_from_columns(
                        &self.columns,
                        &old_values,
                        &table_def.get_primary_columns(),
                    ),
                }
            })
            .collect();

        Ok(updates)
    }
}

impl BulkDelete {
    /// convert bulk delete into sql statements
    pub fn into_multiple_sql_statements(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<Vec<sql::Statement>, Error> {
        let table_def = table_lookup
            .expect("must have a table lookup")
            .get_table_def(&self.from.name)
            .expect("must have a table_def");
        //TODO: create a separate branch for building delete with no Lookup table needed
        let deletes = self.into_multiple_deletes(table_def)?;
        Ok(deletes
            .into_iter()
            .map(|delete| delete.into_sql_statement().expect("must convert"))
            .collect())
    }

    /// convert BulkDelete into multiple Delete AST
    fn into_multiple_deletes(
        &self,
        table_def: &TableDef,
    ) -> Result<Vec<Delete>, Error> {
        let deletes = self
            .values
            .iter()
            .map(|row| {
                let old_values: Vec<&Value> = row.iter().collect();
                Delete {
                    from: self.from.clone(),
                    condition: build_filter_from_columns(
                        &self.columns,
                        &old_values,
                        &table_def.get_primary_columns(),
                    ),
                }
            })
            .collect();

        Ok(deletes)
    }

    /// when there is a primary of this table, use the in filter
    pub fn into_single_sql_statement(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<sql::Statement, Error> {
        let table_def = table_lookup
            .expect("must have a table lookup")
            .get_table_def(&self.from.name)
            .expect("must have a table_def");

        let primary_columns = table_def.get_primary_columns();
        if primary_columns.len() == 1 {
            let pk_column = &primary_columns[0];
            let pk_values: Vec<Value> = self
                .values
                .iter()
                .flat_map(|row| {
                    self.columns.iter().zip(row.iter()).filter_map(
                        |(col, value)| {
                            if pk_column.column.name == col.name {
                                Some(value.clone())
                            } else {
                                None
                            }
                        },
                    )
                })
                .collect();

            assert!(!pk_values.is_empty());

            let delete = Delete {
                from: self.from.clone(),
                condition: Some(Expr::BinaryOperation(Box::new(
                    BinaryOperation {
                        left: Expr::Column(ColumnName {
                            name: pk_column.column.name.clone(),
                        }),
                        operator: Operator::In,
                        right: Expr::MultiValue(pk_values),
                    },
                ))),
            };
            delete.into_sql_statement()
        } else {
            //TODO: must return an Err where pirmary key is not found on this table
            panic!("This is only applicable for table with primary column and only 1 primary column");
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
    use crate::ast::{
        expr::BinaryOperation, parser::utils::to_chars, Operator,
    };

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
                into: TableName {
                    name: "product".into()
                },
                columns: vec![
                    ColumnName {
                        name: "product_id".into()
                    },
                    ColumnName {
                        name: "created_by".into()
                    },
                    ColumnName {
                        name: "created".into()
                    },
                    ColumnName {
                        name: "is_active".into()
                    },
                ],
                source: Source::Values(vec![]),
                returning: Some(vec![
                    ColumnName {
                        name: "product_id".into()
                    },
                    ColumnName {
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
        let statement: sql::Statement = ret.into_sql_statement().unwrap();
        assert_eq!(
            statement.to_string(),
            r#"UPDATE product SET description = 'I''m the new description now', is_active = false WHERE product_id = 1"#
        );
        assert_eq!(
            ret,
            Update {
                table: TableName {
                    name: "product".into()
                },
                columns: vec![
                    ColumnName {
                        name: "description".into(),
                    },
                    ColumnName {
                        name: "is_active".into()
                    },
                ],
                values: vec![
                    Value::String("I'm the new description now".into(),),
                    Value::Bool(false,),
                ],
                condition: Some(Expr::BinaryOperation(Box::new(
                    BinaryOperation {
                        left: Expr::Column(ColumnName {
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
        let statement: sql::Statement = ret.into_sql_statement().unwrap();
        assert_eq!(
            statement.to_string(),
            "DELETE FROM product WHERE product_id = 1"
        );
        assert_eq!(
            ret,
            Delete {
                from: TableName {
                    name: "product".into()
                },
                condition: Some(Expr::BinaryOperation(Box::new(
                    BinaryOperation {
                        left: Expr::Column(ColumnName {
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
                from: TableName {
                    name: "product".into()
                },
                columns: vec![
                    ColumnName {
                        name: "name".into()
                    },
                    ColumnName {
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
                table: TableName {
                    name: "product".into()
                },
                columns: vec![
                    ColumnName {
                        name: "name".into()
                    },
                    ColumnName {
                        name: "is_active".into()
                    }
                ],
                values: vec![]
            }
        );
    }
}
