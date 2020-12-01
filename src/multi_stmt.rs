use crate::StmtData;
use std::io::{
    BufRead,
    BufReader,
    Cursor,
    Read,
    Seek,
    SeekFrom,
};

/// A statement iterator, lazily parse statement with data
pub struct MultiStatement<R> {
    content: BufReader<R>,
}

impl<R> MultiStatement<R>
where
    R: Read + Send + Sync + Seek,
{
    pub fn from_reader(reader: R) -> Self {
        MultiStatement {
            content: BufReader::new(reader),
        }
    }

    pub fn statement_iter(self) -> StatementIter<R> {
        StatementIter {
            content: self.content,
        }
    }
}

pub struct StatementIter<R> {
    content: BufReader<R>,
}

impl<R> StatementIter<R>
where
    R: Read + Send + Sync + Seek,
{
    pub fn new(content: BufReader<R>) -> Self {
        StatementIter { content }
    }

    fn ignore_whitespace(&mut self) {
        let mut buffer = String::new();
        while let Ok(n) = self.content.read_line(&mut buffer) {
            if buffer.trim().is_empty() {
                //ignoring blank lines
            } else {
                // seek to the last non blank character than break the loop
                self.content
                    .seek(SeekFrom::Current(-(n as i64)))
                    .expect("must seek");
                break;
            }
            if n == 0 {
                break;
            }
        }
    }
}

impl<R> Iterator for StatementIter<R>
where
    R: Read + Send + Sync + Seek,
{
    type Item = StmtData<Cursor<Vec<u8>>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.ignore_whitespace();
        let mut buffer = vec![];
        while let Ok(n) = self.content.read_until(b'\n', &mut buffer) {
            let last_char = buffer.iter().last();
            if (n == 1 && last_char == Some(&b'\n')) || n == 0 {
                if !buffer.is_empty() {
                    let stmt = StmtData::from_reader(Cursor::new(buffer))
                        .expect("must not error");
                    return Some(stmt);
                } else {
                    return None;
                }
            }
            if n == 0 {
                break;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn test_multi_statement() {
        let data = r#"PUT /+category{*category_id:s32,name:text,description:text?,slug:text?,topic:i32?,created_at:utc,updated:utc}
1,Staff,staff,Private categories for staff discussion. Topics are only visible to admin and moderators
2,Technology,tecnology,Anything related to technology

PUT /+topic{*topic:s32,title:text,excerpt:text?,created_at:utc(now()),updated_at:utc?}
1,About Euphorum,1
3,Topic3,3
2,Welcome to Euphorum,2"#;

        let ms = MultiStatement::from_reader(Cursor::new(data.as_bytes()));
        let mut iter = ms.statement_iter();
        let stmt1 = iter.next().expect("must have a next");
        if let Statement::Create(create_cat) = stmt1.statement() {
            println!("create1: {}", create_cat);
        }
        let data1 = stmt1.rows_iter().expect("must have csv rows");
        let all_data1 = data1.collect::<Vec<_>>();
        assert_eq!(all_data1.len(), 2);

        let stmt2 = iter.next().expect("must have a next");
        if let Statement::Create(create_topic) = stmt2.statement() {
            println!("create2: {}", create_topic);
        }
        let data2 = stmt2.rows_iter().expect("must have csv rows");
        let all_data2 = dbg!(data2.collect::<Vec<_>>());
        assert_eq!(all_data2.len(), 3);
    }

    #[test]
    fn test_multi_statement_with_multiple_empty_lines() {
        let data = r#"

PUT /+category{*category_id:s32,name:text,description:text?,slug:text?,topic:i32?,created_at:utc,updated:utc}
1,Staff,staff,Private categories for staff discussion. Topics are only visible to admin and moderators
2,Technology,tecnology,Anything related to technology



PUT /+topic{*topic:s32,title:text,excerpt:text?,created_at:utc(now()),updated_at:utc?}
1,About Euphorum,1
3,Topic3,3
2,Welcome to Euphorum,2"#;

        let ms = MultiStatement::from_reader(Cursor::new(data.as_bytes()));
        let mut iter = ms.statement_iter();
        let stmt1 = iter.next().expect("must have a next");
        if let Statement::Create(create_cat) = stmt1.statement() {
            println!("create1: {}", create_cat);
        }
        let data1 = stmt1.rows_iter().expect("must have csv rows");
        let all_data1 = data1.collect::<Vec<_>>();
        assert_eq!(all_data1.len(), 2);

        let stmt2 = iter.next().expect("must have a next");
        if let Statement::Create(create_topic) = stmt2.statement() {
            println!("create2: {}", create_topic);
        }
        let data2 = stmt2.rows_iter().expect("must have csv rows");
        let all_data2 = dbg!(data2.collect::<Vec<_>>());
        assert_eq!(all_data2.len(), 3);
    }
}
