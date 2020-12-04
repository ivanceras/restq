use crate::ast::{
    ddl::TableDef,
    BinaryOperation,
    ColumnName,
    Expr,
    Operator,
};
use sql_ast::ast as sql;
use std::{
    collections::BTreeMap,
    fmt,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TableError {
    #[error("Table join is specified, but no table lookup is supplied")]
    NoSuppliedTableLookup,
    #[error("Table: `{0}` not found in the supplied TableLookup")]
    TableNotFound(String),
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct FromTable {
    pub from: TableName,
    pub join: Option<(JoinType, Box<FromTable>)>,
}

#[derive(Debug, PartialEq, Default, Clone, Hash, Eq)]
pub struct TableName {
    pub name: String,
}

/// Only 3 join types is supported
/// - left join
///     product<-users
/// - right join
///     product->users
/// - inner_join
///     product-><-users
///
#[derive(Debug, PartialEq, Clone)]
pub enum JoinType {
    InnerJoin,
    LeftJoin,
    RightJoin,
    FullJoin,
}

impl Into<sql::TableFactor> for &TableName {
    fn into(self) -> sql::TableFactor {
        sql::TableFactor::Table {
            name: Into::into(self),
            alias: None,
            args: vec![],
            with_hints: vec![],
        }
    }
}
impl Into<sql::ObjectName> for &TableName {
    fn into(self) -> sql::ObjectName {
        sql::ObjectName(vec![sql::Ident::new(&self.name)])
    }
}

impl fmt::Display for TableName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl JoinType {
    fn into_sql_join_operator(&self, constraint: Expr) -> sql::JoinOperator {
        let on_constraint = sql::JoinConstraint::On(Into::into(&constraint));
        match self {
            JoinType::InnerJoin => sql::JoinOperator::Inner(on_constraint),
            JoinType::LeftJoin => sql::JoinOperator::LeftOuter(on_constraint),
            JoinType::RightJoin => sql::JoinOperator::RightOuter(on_constraint),
            JoinType::FullJoin => sql::JoinOperator::FullOuter(on_constraint),
        }
    }
}

impl fmt::Display for JoinType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JoinType::InnerJoin => write!(f, "-><-"),
            JoinType::LeftJoin => write!(f, "<-"),
            JoinType::RightJoin => write!(f, "->"),
            JoinType::FullJoin => write!(f, "<-->"),
        }
    }
}

pub struct TableLookup(BTreeMap<String, TableDef>);

impl TableLookup {
    pub fn new() -> Self {
        TableLookup(BTreeMap::new())
    }

    pub fn add_table(&mut self, table_def: TableDef) -> Option<TableDef> {
        self.0.insert(table_def.table.name.to_string(), table_def)
    }

    /// get the table definition with name
    pub fn get_table_def(&self, name: &str) -> Option<&TableDef> {
        self.0.get(name)
    }

    pub fn find_table(&self, table: &TableName) -> Option<&TableDef> {
        self.get_table_def(&table.name)
    }
}

impl FromTable {
    /// The table lookup is used for supplying the foreign keys in the join
    pub(crate) fn into_table_with_joins(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<sql::TableWithJoins, TableError> {
        Ok(sql::TableWithJoins {
            relation: Into::into(&self.from),
            joins: self.maybe_extract_join(table_lookup)?,
        })
    }

    fn maybe_extract_join(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<Vec<sql::Join>, TableError> {
        match table_lookup {
            Some(table_lookup) => self.extract_join(table_lookup),
            None => {
                match self.join {
                    Some(_) => Err(TableError::NoSuppliedTableLookup),
                    None => Ok(vec![]),
                }
            }
        }
    }

    /// If there is join definition, but no lookup table is supplied
    /// it will return an error
    fn extract_join(
        &self,
        table_lookup: &TableLookup,
    ) -> Result<Vec<sql::Join>, TableError> {
        match &self.join {
            Some((join_type, joined_table)) => {
                let joined_table_def =
                    table_lookup.get_table_def(&joined_table.from.name);

                let this_table_def =
                    table_lookup.get_table_def(&self.from.name);

                match (this_table_def, joined_table_def) {
                    (None, _) => {
                        Err(TableError::TableNotFound(
                            self.from.name.to_string(),
                        ))
                    }
                    (_, None) => {
                        Err(TableError::TableNotFound(
                            joined_table.from.name.to_string(),
                        ))
                    }
                    (Some(this_table_def), Some(joined_table_def)) => {
                        let this_primary_keys =
                            this_table_def.get_primary_columns();

                        let joined_local_columns = joined_table_def
                            .get_local_columns_to_foreign_table(
                                &this_table_def.table.name,
                            );

                        // TODO: we only support 1 primary key for the referred table
                        // TODO: try to support joining all the appropriate column foreign keys
                        assert_eq!(joined_local_columns.len(), 1);
                        assert_eq!(this_primary_keys.len(), 1);

                        let constraint =
                            Expr::BinaryOperation(Box::new(BinaryOperation {
                                left: Expr::ColumnName(ColumnName {
                                    name: format!(
                                        "{}.{}",
                                        joined_table_def.table.name,
                                        joined_local_columns[0].column.name
                                    ),
                                }),
                                operator: Operator::Eq,
                                right: Expr::ColumnName(ColumnName {
                                    name: format!(
                                        "{}.{}",
                                        this_table_def.table.name,
                                        this_primary_keys[0].column.name
                                    ),
                                }),
                            }));

                        let mut ret = vec![sql::Join {
                            relation: Into::into(&joined_table.from),
                            join_operator: join_type
                                .into_sql_join_operator(constraint),
                        }];

                        // if there are some more
                        ret.extend(joined_table.extract_join(table_lookup)?);
                        Ok(ret)
                    }
                }
            }
            None => Ok(vec![]),
        }
    }
}

impl fmt::Display for FromTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.from.fmt(f)?;
        if let Some((join_type, from_table)) = &self.join {
            join_type.fmt(f)?;
            from_table.fmt(f)?;
        }
        Ok(())
    }
}

impl fmt::Display for TableDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.table.fmt(f)?;
        write!(f, "{{")?;
        for (i, col) in self.columns.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            col.fmt(f)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
