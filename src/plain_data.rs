//! Plain csv data
//! contains only the table definition and the csv data
use crate::ast::ddl::table_def;
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
use std::{
    io,
    io::{BufRead, BufReader, Cursor, Read},
};
use thiserror::Error;

pub struct PlainData<R>
where
    R: Read + Send + Sync,
{
    pub header: TableDef,
    pub body: BufReader<R>,
}

impl<R> PlainData<R>
where
    R: Read + Send + Sync,
{
    pub fn from_reader(reader: R) -> Result<Self, crate::Error> {
        let mut bufread = BufReader::new(reader);
        let mut first_line = vec![];
        let _header_len = bufread.read_until(b'\n', &mut first_line)?;

        let header_input = bytes_to_chars(&first_line);
        let table_def = table_def().parse(&header_input)?;

        Ok(PlainData {
            header: table_def,
            body: bufread,
        })
    }

    pub fn table_def(&self) -> &TableDef {
        &self.header
    }

    /// consume self and return as csv rows iterator
    pub fn rows_iter(self) -> CsvRows<R> {
        CsvRows::new(self.body, self.header.columns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_data() {
        let data = "product(*product_id:s32,@name:text,description:text,updated:utc,created_by(users):u32,@is_active:bool)\n\
            1,go pro,a slightly used go pro, 2019-10-31 10:10:10\n\
            2,shovel,a slightly used shovel, 2019-11-11 11:11:11\n\
            ";

        let csv_data =
            PlainData::from_reader(data.as_bytes()).expect("must be valid");

        let rows: Vec<Vec<DataValue>> = csv_data.rows_iter().collect();
        println!("rows: {:#?}", rows);
        assert_eq!(rows.len(), 2);
    }
}
