/// contains Row iterator for the csv data
use crate::{
    ast::{
        ddl::ColumnDef,
        Value,
    },
    data_value::cast_data_value,
    DataValue,
};
use csv::{
    ReaderBuilder,
    StringRecordsIntoIter,
};
use std::io::{
    BufReader,
    Read,
};

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
    type Item = Vec<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.into_iter.next() {
            Some(row) => {
                match row {
                    Ok(row) => {
                        // bulk_update uses 2 rows in one line
                        // so, we chain the column_defs 2 times to extract
                        // and cast the data values
                        let double_row =
                            row.iter().count() == self.column_defs.len() * 2;

                        let columns_defs: Vec<&ColumnDef> = if double_row {
                            self.column_defs
                                .iter()
                                .chain(self.column_defs.iter())
                                .collect()
                        } else {
                            self.column_defs.iter().collect()
                        };

                        let data_values: Vec<Value> = columns_defs
                            .iter()
                            .zip(row.iter())
                            .map(|(_column_def, record)| {
                                Value::String(record.to_string())
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
