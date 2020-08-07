/// CmdData, this contains both command and the data
use crate::{
    ast::{
        ddl,
        ddl::{ColumnDef, TableDef},
        Statement, TableLookup, Value,
    },
    data_value::cast_data_value,
    parser::utils::bytes_to_chars,
    CsvRows, DataValue,
};
use csv::{ReaderBuilder, StringRecordsIntoIter};
use parser::parse_statement_chars;
use std::{
    io,
    io::{BufRead, BufReader, Cursor, Read},
};
use thiserror::Error;

mod parser;

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
    pub header: Statement,
    pub body: BufReader<R>,
}

impl<R> CsvData<R>
where
    R: Read + Send + Sync,
{
    pub fn from_reader(reader: R) -> Result<Self, crate::Error> {
        let mut bufread = BufReader::new(reader);
        let mut first_line = vec![];
        let _header_len = bufread.read_until(b'\n', &mut first_line)?;

        let header_input = bytes_to_chars(&first_line);
        let statement = parse_statement_chars(&header_input)?;

        Ok(CsvData {
            header: statement,
            body: bufread,
        })
    }

    pub fn statement(&self) -> Statement {
        self.header.clone()
    }

    /// consume self and return as csv rows iterator
    pub fn rows_iter(
        self,
        table_lookup: Option<&TableLookup>,
    ) -> Option<CsvRows<R>> {
        match self.header {
            Statement::Create(table_def) => {
                Some(CsvRows::new(self.body, table_def.columns))
            }
            Statement::Insert(insert) => {
                let table_def = table_lookup
                    .expect("need table lookup")
                    .get_table_def(&insert.into.name)
                    .expect("must have table lookup");
                Some(CsvRows::new(
                    self.body,
                    table_def.matching_column_def(&insert.columns),
                ))
            }
            Statement::Select(_) => None,
            Statement::Delete(_) => None,
            Statement::BulkDelete(delete) => {
                let table_def = table_lookup
                    .expect("need table lookup")
                    .get_table_def(&delete.from.name)
                    .expect("must have table lookup");
                Some(CsvRows::new(
                    self.body,
                    table_def.matching_column_def(&delete.columns),
                ))
            }
            Statement::BulkUpdate(update) => {
                let table_def = table_lookup
                    .expect("need table lookup")
                    .get_table_def(&update.table.name)
                    .expect("must have table lookup");
                Some(CsvRows::new(
                    self.body,
                    table_def.matching_column_def(&update.columns),
                ))
            }
            _ => todo!("rows_iter for {:?}.. not yet", self.header),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_data() {
        let data = "PUT /product(*product_id:s32,@name:text,description:text,updated:utc,created_by(users):u32,@is_active:bool)\n\
            1,go pro,a slightly used go pro, 2019-10-31 10:10:10\n\
            2,shovel,a slightly used shovel, 2019-11-11 11:11:11\n\
            ";

        let csv_data =
            CsvData::from_reader(data.as_bytes()).expect("must be valid");

        let rows: Vec<Vec<DataValue>> = csv_data
            .rows_iter(None)
            .expect("must have iterator")
            .collect();
        println!("rows: {:#?}", rows);
        assert_eq!(rows.len(), 2);
    }
}
