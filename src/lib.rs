#![deny(warnings)]

pub mod ast;
mod csv_rows;
mod data_type;
pub mod data_value;
pub mod parser;
pub mod plain_data;
pub mod stmt_data;

pub use ast::{
    ddl::{table_def, ColumnDef},
    Expr, Select, TableDef, TableError,
};
pub use chrono;
pub use csv_rows::CsvRows;
pub use data_type::DataType;
pub use data_value::DataValue;
pub use parser::{
    filter_expr, select,
    utils::{bytes_to_chars, space, to_chars},
};
pub use plain_data::PlainData;
pub use pom;
pub use stmt_data::parse_select_chars;
pub use stmt_data::parse_statement;
pub use stmt_data::StmtData;
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
