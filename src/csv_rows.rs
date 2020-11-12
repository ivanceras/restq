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
