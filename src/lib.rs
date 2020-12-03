#![deny(warnings)]

pub mod ast;
mod csv_rows;
mod data_type;
pub mod data_value;
pub mod multi_stmt;
pub mod plain_data;
pub mod stmt_data;
//reexport sql-ast
pub use sql_ast as sql;

pub use ast::{
    ddl::{
        table_def,
        ColumnDef,
    },
    parser,
    parser::{
        filter_expr,
        select,
        utils::{
            bytes_to_chars,
            space,
            to_chars,
        },
    },
    Column,
    Expr,
    Operator,
    Select,
    Table,
    TableDef,
    TableError,
};
pub use chrono;
pub use csv_rows::CsvRows;
pub use data_type::DataType;
pub use data_value::DataValue;
pub use multi_stmt::MultiStatement;
pub use plain_data::PlainData;
pub use pom;
use serde::{
    Serialize,
    Serializer,
};
pub use stmt_data::{
    parse_select_chars,
    StmtData,
};
use thiserror::Error;
pub use uuid::Uuid;

#[derive(Error, Debug)]
pub enum Error {
    #[error("ParseError: {0}")]
    ParseError(#[from] pom::Error),
    #[error("Invalid DataType: {0}")]
    InvalidDataType(String),
    #[error("{0}")]
    TableError(#[from] TableError),
    #[error("GenericError: {0}")]
    GenericError(String),
    #[error("More than 1 statement is generated")]
    MoreThanOneStatement,
    #[error("{0}")]
    IoError(#[from] std::io::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Error::ParseError(e) => {
                serializer.serialize_newtype_variant(
                    "Error",
                    0,
                    "PomError",
                    &e.to_string(),
                )
            }
            Error::InvalidDataType(e) => {
                serializer.serialize_newtype_variant(
                    "Error",
                    1,
                    "InvalidDataType",
                    e,
                )
            }
            Error::TableError(e) => {
                serializer.serialize_newtype_variant(
                    "Error",
                    2,
                    "TableError",
                    e,
                )
            }
            Error::GenericError(e) => {
                serializer.serialize_newtype_variant(
                    "Error",
                    3,
                    "GenericError",
                    e,
                )
            }
            Error::MoreThanOneStatement => {
                serializer.serialize_newtype_variant(
                    "Error",
                    4,
                    "MoreThanOneStatement",
                    &(),
                )
            }
            Error::IoError(e) => {
                serializer.serialize_newtype_variant(
                    "Error",
                    5,
                    "IoError",
                    &e.to_string(),
                )
            }
        }
    }
}
