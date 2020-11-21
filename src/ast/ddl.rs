//! DDL stands for Data Definition Language
//! and this module contains the AST for DDL operations
//! such as create, alter, drop table
mod ddl_parser;

use crate::{
    ast::{
        dml::{
            Insert,
            Source,
        },
        Column,
        Function,
        Statement,
        Table,
        TableError,
        TableLookup,
    },
    data_type::DataType,
    data_value::DataValue,
    Error,
};
pub use ddl_parser::{
    alter_table,
    drop_table,
    table_def,
};
use sql_ast::ast as sql;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct ColumnDef {
    pub column: Column,
    //TODO: convert it to just Vec<ColumnAttribute>
    pub attributes: Option<Vec<ColumnAttribute>>,
    pub data_type_def: DataTypeDef,
    pub foreign: Option<Table>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DataTypeDef {
    pub data_type: DataType,
    pub is_optional: bool,
    pub default: Option<DefaultValue>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DefaultValue {
    DataValue(DataValue),
    Function(Function),
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum ColumnAttribute {
    Primary,
    Unique,
    Index,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct TableDef {
    pub table: Table,
    pub columns: Vec<ColumnDef>,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct AlterTable {
    pub table: Table,
    pub alter_operations: Vec<AlterOperation>,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct DropTable {
    pub table: Table,
}

// Note: Only one alter operation is allowed
// on an alter table query.
// So, if there are multiple alter opration
// that needs to be executed, there will be multiple
// alter table query that needs to be generated and executed
#[derive(Debug, PartialEq, Clone)]
pub enum AlterOperation {
    DropColumn(Column),
    AddColumn(ColumnDef),
    AlterColumn(Column, ColumnDef),
}

impl TableDef {
    pub fn derive_insert(&self) -> Insert {
        let columns: Vec<Column> =
            self.columns.iter().map(|c| c.column.clone()).collect();
        Insert {
            into: self.table.clone(),
            columns: columns.clone(),
            source: Source::Parameterized(
                self.columns
                    .iter()
                    .enumerate()
                    .map(|(i, _c)| i + 1)
                    .collect(),
            ),
            returning: Some(columns),
        }
    }

    /// get the local columns that referes to the foreign table
    pub(crate) fn get_local_columns_to_foreign_table(
        &self,
        table_name: &str,
    ) -> Vec<&ColumnDef> {
        self.columns
            .iter()
            .filter(|column| {
                match &column.foreign {
                    Some(foreign) => foreign.name == table_name,
                    None => false,
                }
            })
            .collect()
    }

    /// get the primary columns of this table
    pub fn get_primary_columns(&self) -> Vec<&ColumnDef> {
        self.columns
            .iter()
            .filter(|column| {
                match &column.attributes {
                    Some(attributes) => {
                        attributes
                            .iter()
                            .any(|att| *att == ColumnAttribute::Primary)
                    }
                    None => false,
                }
            })
            .collect()
    }
}

impl Into<Statement> for TableDef {
    fn into(self) -> Statement {
        Statement::Create(self)
    }
}
impl Into<Statement> for DropTable {
    fn into(self) -> Statement {
        Statement::DropTable(self)
    }
}

impl Into<sql::Statement> for &DropTable {
    fn into(self) -> sql::Statement {
        sql::Statement::Drop {
            object_type: sql::ObjectType::Table,
            if_exists: true,
            names: vec![Into::into(&self.table)],
            cascade: true,
        }
    }
}

impl TableDef {
    pub(crate) fn into_sql_statement(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<sql::Statement, TableError> {
        let mut column_defs = vec![];
        for column in self.columns.iter() {
            column_defs.push(column.into_sql_column_def(table_lookup)?);
        }
        // TODO for indexes constraint

        Ok(sql::Statement::CreateTable {
            if_not_exists: true,
            name: Into::into(&self.table),
            columns: column_defs,
            constraints: vec![],
            with_options: vec![],
            external: false,
            file_format: None,
            location: None,
        })
    }
}

impl AlterTable {
    pub fn into_sql_statements(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<Vec<sql::Statement>, Error> {
        let mut statements = vec![];
        for operation in self.alter_operations.iter() {
            statements.push(sql::Statement::AlterTable {
                name: Into::into(&self.table),
                operation: operation.into_sql_alter_operation(table_lookup)?,
            });
        }
        Ok(statements)
    }
}

impl AlterOperation {
    fn into_sql_alter_operation(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<sql::AlterTableOperation, Error> {
        match self {
            AlterOperation::AddColumn(column_def) => {
                Ok(sql::AlterTableOperation::AddColumn(
                    column_def.into_sql_column_def(table_lookup)?,
                ))
            }
            AlterOperation::DropColumn(column) => {
                Ok(sql::AlterTableOperation::DropColumn {
                    column: Into::into(column),
                    if_exists: true,
                    cascade: true,
                })
            }
            // get the old column definition
            // from the table_lookup and see
            // if it needs rename, add to index, add to unique,
            // drop the index, etc.
            AlterOperation::AlterColumn(_column, _column_def) => todo!(),
        }
    }
}

impl ColumnDef {
    fn into_sql_column_options(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<Vec<sql::ColumnOption>, TableError> {
        let mut att_column_options = match &self.attributes {
            Some(attributes) => {
                attributes
                    .iter()
                    .filter_map(|att| att.into_sql_column_option())
                    .collect()
            }
            None => vec![],
        };
        att_column_options
            .extend(self.data_type_def.into_sql_column_options(table_lookup));

        if let Some(foreign) = &self.foreign {
            match table_lookup {
                None => return Err(TableError::NoSuppliedTableLookup),
                Some(table_lookup) => {
                    let foreign_table_def =
                        table_lookup.get_table_def(&foreign.name);

                    match foreign_table_def {
                        None => {
                            return Err(TableError::TableNotFound(
                                foreign.name.to_string(),
                            ));
                        }
                        Some(foreign_table_def) => {
                            let pk = foreign_table_def.get_primary_columns();
                            //TODO: supporting only 1 primary key for now.
                            assert_eq!(pk.len(), 1);
                            let fk_column_options =
                                sql::ColumnOption::ForeignKey {
                                    foreign_table: Into::into(
                                        &foreign_table_def.table,
                                    ),
                                    referred_columns: vec![Into::into(
                                        &pk[0].column,
                                    )],
                                };
                            att_column_options.push(fk_column_options);
                        }
                    }
                }
            }
        }

        Ok(att_column_options)
    }

    fn into_sql_column_options_def(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<Vec<sql::ColumnOptionDef>, TableError> {
        Ok(self
            .into_sql_column_options(table_lookup)?
            .into_iter()
            .map(|option| sql::ColumnOptionDef { name: None, option })
            .collect())
    }

    pub fn is_primary(&self) -> bool {
        if let Some(attributes) = &self.attributes {
            attributes
                .iter()
                .any(|att| ColumnAttribute::Primary == *att)
        } else {
            false
        }
    }
}

impl DataTypeDef {
    fn into_sql_column_options(
        &self,
        _table_lookup: Option<&TableLookup>,
    ) -> Vec<sql::ColumnOption> {
        vec![
            if !self.is_optional {
                Some(sql::ColumnOption::NotNull)
            } else {
                None
            },
            self.default
                .as_ref()
                .map(|default| sql::ColumnOption::Default(Into::into(default))),
        ]
        .into_iter()
        .filter_map(|v| v)
        .collect()
    }
}

impl fmt::Display for DataTypeDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.data_type.fmt(f)?;
        if self.is_optional {
            write!(f, "?")?;
        }
        if let Some(default) = &self.default {
            write!(f, "({})", default)?;
        }
        Ok(())
    }
}

impl ColumnDef {
    fn into_sql_column_def(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<sql::ColumnDef, TableError> {
        Ok(sql::ColumnDef {
            name: Into::into(&self.column),
            data_type: Into::into(&self.data_type_def.data_type),
            collation: None,
            options: self.into_sql_column_options_def(table_lookup)?,
        })
    }

    /// create a data_value from this ColumnDef
    pub fn default_data_value() -> DataValue {
        DataValue::Nil
    }
}

impl fmt::Display for ColumnDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(attrs) = &self.attributes {
            for att in attrs {
                att.fmt(f)?;
            }
        }
        self.column.fmt(f)?;
        if let Some(foreign) = &self.foreign {
            write!(f, "({})", foreign)?;
        }
        write!(f, ":")?;
        self.data_type_def.fmt(f)?;
        Ok(())
    }
}

impl ColumnAttribute {
    fn into_sql_column_option(&self) -> Option<sql::ColumnOption> {
        match self {
            ColumnAttribute::Primary => {
                Some(sql::ColumnOption::Unique { is_primary: true })
            }

            ColumnAttribute::Unique => {
                Some(sql::ColumnOption::Unique { is_primary: false })
            }
            ColumnAttribute::Index => None,
        }
    }
}

impl fmt::Display for ColumnAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ColumnAttribute::Primary => write!(f, "*"),
            ColumnAttribute::Unique => write!(f, "&"),
            ColumnAttribute::Index => write!(f, "@"),
        }
    }
}

impl fmt::Display for DefaultValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DefaultValue::DataValue(v) => v.fmt(f),
            DefaultValue::Function(v) => v.fmt(f),
        }
    }
}

impl Into<sql::Expr> for &DefaultValue {
    fn into(self) -> sql::Expr {
        match self {
            DefaultValue::DataValue(v) => sql::Expr::Value(v.into()),
            DefaultValue::Function(v) => sql::Expr::Function(v.into()),
        }
    }
}

impl From<DataValue> for DefaultValue {
    fn from(v: DataValue) -> Self {
        DefaultValue::DataValue(v)
    }
}

impl From<Function> for DefaultValue {
    fn from(f: Function) -> Self {
        DefaultValue::Function(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::to_chars;
    use ddl_parser::*;

    #[test]
    fn parse_drop_table() {
        let input = to_chars("-product");
        let ret = drop_table().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            DropTable {
                table: Table {
                    name: "product".into()
                }
            }
        );
        let statement: Statement = Into::into(ret);
        assert_eq!(
            statement.into_sql_statement(None).unwrap().to_string(),
            "DROP TABLE IF EXISTS product CASCADE"
        );
    }

    #[test]
    fn parse_alter_operation() {
        let input = to_chars(
            "product{id=&*product_id:u32,-description,+discount:f32?(0.0)}",
        );
        let ret = alter_table().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);

        assert_eq!(
            ret,
            AlterTable {
                table: Table {
                    name: "product".into()
                },
                alter_operations: vec![
                    AlterOperation::AlterColumn(
                        Column { name: "id".into() },
                        ColumnDef {
                            column: Column {
                                name: "product_id".into()
                            },
                            attributes: Some(vec![
                                ColumnAttribute::Unique,
                                ColumnAttribute::Primary,
                            ]),
                            data_type_def: DataTypeDef {
                                data_type: DataType::U32,
                                is_optional: false,
                                default: None,
                            },
                            foreign: None,
                        },
                    ),
                    AlterOperation::DropColumn(Column {
                        name: "description".into(),
                    }),
                    AlterOperation::AddColumn(ColumnDef {
                        column: Column {
                            name: "discount".into()
                        },
                        attributes: None,
                        data_type_def: DataTypeDef {
                            data_type: DataType::F32,
                            is_optional: true,
                            default: Some(DefaultValue::DataValue(
                                DataValue::F32(0.0,)
                            )),
                        },
                        foreign: None,
                    })
                ],
            }
        );
    }

    #[test]
    fn parse_alter_operation_simple() {
        let input = to_chars("product{-description,+discount:f32?(0.1)}");
        let ret = alter_table().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);

        let statements = ret.into_sql_statements(None).unwrap();
        assert_eq!(statements.len(), 2);
        assert_eq!(
            statements[0].to_string(),
            "ALTER TABLE product DROP COLUMN IF EXISTS description CASCADE"
        );
        assert_eq!(
            statements[1].to_string(),
            "ALTER TABLE product ADD COLUMN discount float DEFAULT 0.1"
        );
    }

    #[test]
    fn parse_data_type_def() {
        let input = to_chars("u32");
        let ret = data_type_def().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            DataTypeDef {
                data_type: DataType::U32,
                is_optional: false,
                default: None,
            }
        );
    }

    #[test]
    fn test_invalid_data_type_def() {
        let input = to_chars("invalid32");
        let ret = data_type_def().parse(&input);
        println!("{:#?}", ret);
        assert!(ret.is_err());
        assert!(ret
            .err()
            .unwrap()
            .to_string()
            .contains(r#"InvalidDataType("invalid32")"#));
    }

    #[test]
    fn parse_column_def() {
        let input = to_chars("*product_id:u32");
        let ret = column_def().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            ColumnDef {
                column: Column {
                    name: "product_id".to_string()
                },
                attributes: Some(vec![ColumnAttribute::Primary]),
                data_type_def: DataTypeDef {
                    data_type: DataType::U32,
                    is_optional: false,
                    default: None,
                },
                foreign: None,
            },
        );
    }

    #[test]
    fn display_column_def() {
        assert_eq!(
            "*product_id:u32",
            ColumnDef {
                column: Column {
                    name: "product_id".to_string()
                },
                attributes: Some(vec![ColumnAttribute::Primary]),
                data_type_def: DataTypeDef {
                    data_type: DataType::U32,
                    is_optional: false,
                    default: None,
                },
                foreign: None,
            }
            .to_string(),
        );
    }

    #[test]
    fn display_column_def2() {
        assert_eq!(
            "*&@product_id(product):u32?(qr-123)",
            ColumnDef {
                column: Column {
                    name: "product_id".to_string()
                },
                attributes: Some(vec![
                    ColumnAttribute::Primary,
                    ColumnAttribute::Unique,
                    ColumnAttribute::Index
                ]),
                data_type_def: DataTypeDef {
                    data_type: DataType::U32,
                    is_optional: true,
                    default: Some(DefaultValue::DataValue(DataValue::Text(
                        "qr-123".to_string()
                    ))),
                },
                foreign: Some(Table {
                    name: "product".to_string()
                }),
            }
            .to_string(),
        );
    }

    #[test]
    fn test_column_def_list() {
        let input = to_chars("*product_id:u32,name:text");
        let ret = column_def_list().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            vec![
                ColumnDef {
                    column: Column {
                        name: "product_id".to_string()
                    },
                    attributes: Some(vec![ColumnAttribute::Primary]),
                    data_type_def: DataTypeDef {
                        data_type: DataType::U32,
                        is_optional: false,
                        default: None,
                    },
                    foreign: None,
                },
                ColumnDef {
                    column: Column {
                        name: "name".to_string()
                    },
                    attributes: None,
                    data_type_def: DataTypeDef {
                        data_type: DataType::Text,
                        is_optional: false,
                        default: None,
                    },
                    foreign: None,
                }
            ],
        );
    }

    #[test]
    fn test_invalid_column_def_list() {
        let input = to_chars("*product_id:invalid32,name:text");
        let ret = column_def_list().parse(&input);
        println!("{:#?}", ret);
        assert!(ret.is_err());
        assert!(ret
            .err()
            .unwrap()
            .to_string()
            .contains(r#"InvalidDataType("invalid32")"#));
    }

    #[test]
    fn test_invalid_column_def_list_2() {
        let input = to_chars("id:u32,*product_id:invalid32,name:text");
        let ret = column_def_list().parse(&input);
        println!("{:#?}", ret);
        assert!(ret.is_err());
        assert!(ret
            .err()
            .unwrap()
            .to_string()
            .contains(r#"InvalidDataType("invalid32")"#));
    }

    #[test]
    fn test_invalid_column_def_list_3() {
        let input = to_chars("id:u32,*product_id:u32,name:invalid_text");
        let ret = column_def_list().parse(&input);
        println!("{:#?}", ret);
        assert!(ret.is_err());
        assert!(ret
            .err()
            .unwrap()
            .to_string()
            .contains(r#"InvalidDataType("invalid_text")"#));
    }

    #[test]
    fn parse_invalid_column_def() {
        let input = to_chars("*product_id:invalid32");
        let ret = column_def().parse(&input);
        println!("{:#?}", ret);
        assert!(ret.is_err());
        assert!(ret
            .err()
            .unwrap()
            .to_string()
            .contains(r#"InvalidDataType("invalid32")"#));
    }

    #[test]
    fn parse_column_def_with_foreign() {
        let input = to_chars("*product_id(product):u32");
        let ret = column_def().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            ColumnDef {
                column: Column {
                    name: "product_id".to_string()
                },
                attributes: Some(vec![ColumnAttribute::Primary]),
                data_type_def: DataTypeDef {
                    data_type: DataType::U32,
                    is_optional: false,
                    default: None,
                },
                foreign: Some(Table {
                    name: "product".to_string()
                }),
            },
        );
    }

    #[test]
    fn parse_data_type_def_opt() {
        let input = to_chars("u32?");
        let ret = data_type_def().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            DataTypeDef {
                data_type: DataType::U32,
                is_optional: true,
                default: None,
            }
        );
    }

    #[test]
    fn parse_data_type_def_default() {
        let input = to_chars("f32(0.0)");
        let ret = data_type_def().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            DataTypeDef {
                data_type: DataType::F32,
                is_optional: false,
                default: Some(DefaultValue::DataValue(DataValue::F32(0.0))),
            }
        );
    }
    #[test]
    fn parse_data_type_def_opt_default() {
        let input = to_chars("f64?(11.62)");
        let ret = data_type_def().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            DataTypeDef {
                data_type: DataType::F64,
                is_optional: true,
                default: Some(DefaultValue::DataValue(DataValue::F64(11.62))),
            }
        );
    }

    #[test]
    fn parse_actor_table_with_invalid_data_type() {
        let input = to_chars(
            "actor{*actor_id:s32,&first_name:text,&@last_name:text,last_update:timestamp,created_by(users):u32,is_active:bool}",
        );
        let ret = table_def().parse(&input);
        println!("{:#?}", ret);
        assert!(ret.is_err());
        assert!(ret
            .err()
            .unwrap()
            .to_string()
            .contains(r#"InvalidDataType("timestamp")"#));
    }

    #[test]
    fn parse_actor_table() {
        let input = to_chars(
            "actor{*actor_id:s32,&first_name:text,&@last_name:text,last_update:utc,created_by(users):u32,is_active:bool}",
        );
        let ret = table_def().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(
            ret,
            TableDef {
                table: Table {
                    name: "actor".into()
                },
                columns: vec![
                    ColumnDef {
                        column: Column {
                            name: "actor_id".to_string()
                        },
                        attributes: Some(vec![ColumnAttribute::Primary]),
                        data_type_def: DataTypeDef {
                            data_type: DataType::S32,
                            is_optional: false,
                            default: None,
                        },
                        foreign: None,
                    },
                    ColumnDef {
                        column: Column {
                            name: "first_name".into()
                        },
                        attributes: Some(vec![ColumnAttribute::Unique]),
                        data_type_def: DataTypeDef {
                            data_type: DataType::Text,
                            is_optional: false,
                            default: None,
                        },
                        foreign: None,
                    },
                    ColumnDef {
                        column: Column {
                            name: "last_name".into()
                        },
                        attributes: Some(vec![
                            ColumnAttribute::Unique,
                            ColumnAttribute::Index
                        ]),
                        data_type_def: DataTypeDef {
                            data_type: DataType::Text,
                            is_optional: false,
                            default: None,
                        },
                        foreign: None,
                    },
                    ColumnDef {
                        column: Column {
                            name: "last_update".into()
                        },
                        attributes: None,
                        data_type_def: DataTypeDef {
                            data_type: DataType::Utc,
                            is_optional: false,
                            default: None,
                        },
                        foreign: None,
                    },
                    ColumnDef {
                        column: Column {
                            name: "created_by".into()
                        },
                        attributes: None,
                        data_type_def: DataTypeDef {
                            data_type: DataType::U32,
                            is_optional: false,
                            default: None,
                        },
                        foreign: Some(Table {
                            name: "users".into()
                        }),
                    },
                    ColumnDef {
                        column: Column {
                            name: "is_active".into()
                        },
                        attributes: None,
                        data_type_def: DataTypeDef {
                            data_type: DataType::Bool,
                            is_optional: false,
                            default: None,
                        },
                        foreign: None,
                    },
                ]
            }
        )
    }

    #[test]
    fn displa_actor_table() {
        assert_eq!(
            "actor{*actor_id:s32,&first_name:text,&@last_name:text,last_update:utc,created_by(users):u32,is_active:bool}",
            TableDef {
                table: Table {
                    name: "actor".into()
                },
                columns: vec![
                    ColumnDef {
                        column: Column {
                            name: "actor_id".to_string()
                        },
                        attributes: Some(vec![ColumnAttribute::Primary]),
                        data_type_def: DataTypeDef {
                            data_type: DataType::S32,
                            is_optional: false,
                            default: None,
                        },
                        foreign: None,
                    },
                    ColumnDef {
                        column: Column {
                            name: "first_name".into()
                        },
                        attributes: Some(vec![ColumnAttribute::Unique]),
                        data_type_def: DataTypeDef {
                            data_type: DataType::Text,
                            is_optional: false,
                            default: None,
                        },
                        foreign: None,
                    },
                    ColumnDef {
                        column: Column {
                            name: "last_name".into()
                        },
                        attributes: Some(vec![
                            ColumnAttribute::Unique,
                            ColumnAttribute::Index
                        ]),
                        data_type_def: DataTypeDef {
                            data_type: DataType::Text,
                            is_optional: false,
                            default: None,
                        },
                        foreign: None,
                    },
                    ColumnDef {
                        column: Column {
                            name: "last_update".into()
                        },
                        attributes: None,
                        data_type_def: DataTypeDef {
                            data_type: DataType::Utc,
                            is_optional: false,
                            default: None,
                        },
                        foreign: None,
                    },
                    ColumnDef {
                        column: Column {
                            name: "created_by".into()
                        },
                        attributes: None,
                        data_type_def: DataTypeDef {
                            data_type: DataType::U32,
                            is_optional: false,
                            default: None,
                        },
                        foreign: Some(Table {
                            name: "users".into()
                        }),
                    },
                    ColumnDef {
                        column: Column {
                            name: "is_active".into()
                        },
                        attributes: None,
                        data_type_def: DataTypeDef {
                            data_type: DataType::Bool,
                            is_optional: false,
                            default: None,
                        },
                        foreign: None,
                    },
                ]
            }.to_string()
        )
    }
}
