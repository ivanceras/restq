/// contains Row iterator for the csv data
use crate::ast::Value;
use crate::{
    data_value,
    ColumnDef,
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

    pub fn into_data_values(
        self,
        column_defs: &[ColumnDef],
    ) -> Vec<Vec<DataValue>> {
        self.map(|row| Self::cast_row_to_data_value(&row, column_defs))
            .collect()
    }

    fn cast_row_to_data_value(
        row: &[Value],
        column_defs: &[ColumnDef],
    ) -> Vec<DataValue> {
        column_defs
            .iter()
            .zip(row.iter())
            .map(|(column_def, value)| {
                data_value::cast_data_value(
                    value,
                    &column_def.data_type_def.data_type,
                )
            })
            .collect()
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
