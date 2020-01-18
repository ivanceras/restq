use crate::{
    ast::{
        dml::{
            Insert,
            Source,
        },
        Column,
        Statement,
        Table,
        TableError,
        TableLookup,
    },
    data_type::{
        data_type,
        DataType,
    },
    data_value,
    data_value::DataValue,
    parser::{
        utils::end_or_ln,
        *,
    },
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
    pub default: Option<DataValue>,
}

#[derive(Debug, PartialEq, Clone)]
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

    pub(crate) fn matching_column_def(
        &self,
        columns: &[Column],
    ) -> Vec<ColumnDef> {
        self.columns
            .iter()
            .filter(|c| columns.contains(&c.column))
            .map(|c| c.clone())
            .collect()
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
            AlterOperation::AlterColumn(column, column_def) => todo!(),
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
        table_lookup: Option<&TableLookup>,
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

fn column_attribute<'a>() -> Parser<'a, char, ColumnAttribute> {
    sym('*').map(|_| ColumnAttribute::Primary)
        | sym('&').map(|_| ColumnAttribute::Unique)
        | sym('@').map(|_| ColumnAttribute::Index)
}

fn column_attributes<'a>() -> Parser<'a, char, Vec<ColumnAttribute>> {
    column_attribute().repeat(1..)
}

/// foreign = "(", table, ")"
fn foreign<'a>() -> Parser<'a, char, Table> {
    (sym('(') * table() - sym(')'))
}

/// parse column definition with the format
/// column_def  = [column_attribute], column, [foreign] [":" data_type];
/// example:
///     &*product_id(product):u32
fn column_def<'a>() -> Parser<'a, char, ColumnDef> {
    ((column_attributes().opt() + column() + foreign().opt() - sym(':')
        + data_type_def())
    .map(|(((attributes, column), foreign), data_type)| {
        ColumnDef {
            column,
            attributes,
            data_type_def: data_type,
            foreign,
        }
    }))
    .name("column_def")
}

fn column_def_list<'a>() -> Parser<'a, char, Vec<ColumnDef>> {
    list_fail(column_def(), sym(',')).name("column_def_list")
}

fn enclosed_column_def_list<'a>() -> Parser<'a, char, Vec<ColumnDef>> {
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
        |(table, alter_operations)| {
            AlterTable {
                table,
                alter_operations,
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::to_chars;

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
                            default: Some(DataValue::F32(0.0,),),
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
        assert!(
            ret.err()
                .unwrap()
                .to_string()
                .contains(r#"InvalidDataType("invalid32")"#)
        );
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
        assert!(
            ret.err()
                .unwrap()
                .to_string()
                .contains(r#"InvalidDataType("invalid32")"#)
        );
    }

    #[test]
    fn test_invalid_column_def_list_2() {
        let input = to_chars("id:u32,*product_id:invalid32,name:text");
        let ret = column_def_list().parse(&input);
        println!("{:#?}", ret);
        assert!(ret.is_err());
        assert!(
            ret.err()
                .unwrap()
                .to_string()
                .contains(r#"InvalidDataType("invalid32")"#)
        );
    }

    #[test]
    fn test_invalid_column_def_list_3() {
        let input = to_chars("id:u32,*product_id:u32,name:invalid_text");
        let ret = column_def_list().parse(&input);
        println!("{:#?}", ret);
        assert!(ret.is_err());
        assert!(
            ret.err()
                .unwrap()
                .to_string()
                .contains(r#"InvalidDataType("invalid_text")"#)
        );
    }

    #[test]
    fn parse_invalid_column_def() {
        let input = to_chars("*product_id:invalid32");
        let ret = column_def().parse(&input);
        println!("{:#?}", ret);
        assert!(ret.is_err());
        assert!(
            ret.err()
                .unwrap()
                .to_string()
                .contains(r#"InvalidDataType("invalid32")"#)
        );
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
                default: Some(DataValue::F32(0.0)),
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
                default: Some(DataValue::F64(11.62)),
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
        assert!(
            ret.err()
                .unwrap()
                .to_string()
                .contains(r#"InvalidDataType("timestamp")"#)
        );
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
}
