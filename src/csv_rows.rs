/// contains Row iterator for the csv data
use crate::{
    ast::Value,
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
}

impl<R> CsvRows<R>
where
    R: Read + Send + Sync,
{
    pub fn new(input: BufReader<R>) -> Self {
        let into_iter = ReaderBuilder::new()
            .has_headers(false)
            .from_reader(input)
            .into_records();

        CsvRows { into_iter }
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
                        let data_values: Vec<Value> = row
                            .iter()
                            .map(|record| Value::String(record.to_string()))
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
