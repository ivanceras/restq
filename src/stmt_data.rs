/// StmtData, this contains both statement and the data
use crate::{
    ast::parser::utils::bytes_to_chars,
    ast::{
        Statement,
        TableLookup,
    },
    CsvRows,
};
use parser::parse_statement_chars;
pub use parser::{
    parse_select_chars,
    parse_statement,
};
use std::io::{
    BufRead,
    BufReader,
    Read,
};

mod parser;

/// Contains both the statement commands and the data
pub struct StmtData<R>
where
    R: Read + Send + Sync,
{
    pub header: Statement,
    pub body: BufReader<R>,
}

impl<R> StmtData<R>
where
    R: Read + Send + Sync,
{
    pub fn from_reader(reader: R) -> Result<Self, crate::Error> {
        let mut bufread = BufReader::new(reader);
        let mut first_line = vec![];
        let _header_len = bufread.read_until(b'\n', &mut first_line)?;

        let header_input = bytes_to_chars(&first_line);
        let statement = parse_statement_chars(&header_input)?;

        Ok(StmtData {
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
            Statement::Create(table_def) => Some(CsvRows::new(self.body)),
            Statement::Insert(insert) => Some(CsvRows::new(self.body)),
            Statement::Select(_) => None,
            Statement::Delete(_) => None,
            Statement::BulkDelete(delete) => Some(CsvRows::new(self.body)),
            Statement::BulkUpdate(update) => Some(CsvRows::new(self.body)),
            _ => todo!("rows_iter for {:?}.. not yet", self.header),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Value;

    #[test]
    fn test_csv_data() {
        let data = "PUT /product(*product_id:s32,@name:text,description:text,updated:utc,created_by(users):u32,@is_active:bool)\n\
            1,go pro,a slightly used go pro, 2019-10-31 10:10:10\n\
            2,shovel,a slightly used shovel, 2019-11-11 11:11:11\n\
            ";

        let csv_data =
            StmtData::from_reader(data.as_bytes()).expect("must be valid");

        let rows: Vec<Vec<Value>> = csv_data
            .rows_iter(None)
            .expect("must have iterator")
            .collect();
        println!("rows: {:#?}", rows);
        assert_eq!(rows.len(), 2);
    }
}
