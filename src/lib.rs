//#![deny(warnings)]
#[macro_use]
extern crate lazy_static;

pub mod ast;
pub mod csv_data;
mod data_type;
pub mod data_value;
pub mod parser;

use ast::{
    ddl::table_def,
    Expr,
    Select,
    TableDef,
    TableError,
};

pub use data_type::DataType;
pub use data_value::DataValue;
pub use parser::{
    filter_expr,
    select,
    utils::{
        bytes_to_chars,
        to_chars,
    },
};
pub use pom;
use thiserror::Error;

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
}
