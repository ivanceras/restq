use crate::{
    ast::{
        ddl,
        ddl::{
            ColumnDef,
            TableDef,
        },
        Value,
    },
    data_value::cast_data_value,
    parser::utils::bytes_to_chars,
    DataValue,
};
use csv::{
    ReaderBuilder,
    StringRecordsIntoIter,
};
use std::{
    io,
    io::{
        BufRead,
        BufReader,
        Cursor,
        Read,
    },
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CsvError {
    #[error("error parsing header {0}")]
    HeaderParseError(pom::Error),
    #[error("io error {0}")]
    HeaderIoError(io::Error),
}

pub struct CsvData<R>
where
    R: Read + Send + Sync,
{
    pub table_def: TableDef,
    pub rows_iter: CsvRows<R>,
}

impl<R> CsvData<R>
where
    R: Read + Send + Sync,
{
    pub fn from_reader(reader: R) -> Result<Self, CsvError> {
        let mut bufread = BufReader::new(reader);
        let mut first_line = vec![];
        let _header_len = bufread
            .read_until(b'\n', &mut first_line)
            .map_err(|e| CsvError::HeaderIoError(e))?;

        let header_input = bytes_to_chars(&first_line);
        let table_def = ddl::table_def()
            .parse(&header_input)
            .map_err(|e| CsvError::HeaderParseError(e))?;

        let column_defs = table_def.columns.clone();
        let rows_iter = CsvRows::new(bufread, column_defs);
        Ok(CsvData {
            table_def,
            rows_iter,
        })
    }

    pub fn rows_iter(&mut self) -> &mut CsvRows<R> {
        &mut self.rows_iter
    }
}

pub struct CsvRows<R>
where
    R: Read + Send + Sync,
{
    into_iter: StringRecordsIntoIter<BufReader<R>>,
    column_defs: Vec<ColumnDef>,
}

impl<R> CsvRows<R>
where
    R: Read + Send + Sync,
{
    pub fn new(input: BufReader<R>, column_defs: Vec<ColumnDef>) -> Self {
        let into_iter = ReaderBuilder::new()
            .has_headers(false)
            .from_reader(input)
            .into_records();

        CsvRows {
            into_iter,
            column_defs,
        }
    }
}

impl<R> Iterator for CsvRows<R>
where
    R: Read + Send + Sync,
{
    type Item = Vec<DataValue>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.into_iter.next() {
            Some(row) => {
                match row {
                    Ok(row) => {
                        let data_values: Vec<DataValue> = self
                            .column_defs
                            .iter()
                            .zip(row.iter())
                            .map(|(column_def, record)| {
                                cast_data_value(
                                    &Value::String(record.to_string()),
                                    &column_def.data_type_def.data_type,
                                )
                            })
                            .collect();
                        Some(data_values)
                    }
                    Err(_e) => None,
                }
            }
            None => None,
        }
    }
}
