#![deny(warnings)]

use http::{
    Method,
    Request,
};
use percent_encoding::percent_decode_str;
pub use restq::{
    ast::{
        ddl::{
            alter_table,
            drop_table,
            table_def,
            ColumnDef,
        },
        dml::{
            delete,
            insert,
            update,
        },
        AlterTable,
        Delete,
        DropTable,
        Foreign,
        Insert,
        Select,
        Statement,
        TableDef,
        Update,
        Value,
    },
    parser::select,
    pom::parser::{
        sym,
        tag,
        Parser,
    },
    space,
    to_chars,
    CsvRows,
    DataValue,
    Error,
    StmtData,
};
use std::io::Cursor;

/// Parse into SQL Statement AST from http::Request
pub fn parse_statement(
    request: &Request<String>,
) -> Result<(Statement, Vec<Vec<Value>>), Error> {
    let method = request.method();
    let url = extract_path_and_query(request);
    let body = request.body().as_bytes().to_vec();
    parse_statement_from_parts(method, &url, Some(body))
}

fn parse_statement_from_parts(
    method: &Method,
    url: &str,
    body: Option<Vec<u8>>,
) -> Result<(Statement, Vec<Vec<Value>>), Error> {
    let csv_data = csv_data_from_parts(&method, url, body)?;
    let statement = csv_data.statement();
    let csv_rows = csv_data.rows_iter();

    let data_values: Vec<Vec<Value>> = if let Some(csv_rows) = csv_rows {
        csv_rows.into_iter().collect()
    } else {
        vec![]
    };

    Ok((statement, data_values))
}

fn extract_path_and_query<T>(request: &Request<T>) -> String {
    let pnq = request
        .uri()
        .path_and_query()
        .map(|pnq| pnq.as_str())
        .unwrap_or("/");
    percent_decode_str(pnq).decode_utf8_lossy().to_string()
}

pub fn extract_restq_from_request<T>(request: &Request<T>) -> String {
    let method = request.method();
    let url = extract_path_and_query(request);
    let prefix = method_to_prefix(method);
    format!("{} {}\n", prefix, url)
}

fn method_to_prefix(method: &Method) -> &'static str {
    match *method {
        Method::GET => "GET",
        Method::PUT => "PUT",
        Method::POST => "POST",
        Method::PATCH => "PATCH",
        Method::DELETE => "DELETE",
        Method::HEAD => "HEAD",
        Method::OPTIONS => todo!(),
        Method::TRACE => todo!("use this for database connection checking"),
        Method::CONNECT => {
            todo!("maybe used this for precaching/db_url connect")
        }
        _ => {
            let _ext = method.as_str();
            todo!("Support for DROP, PURGE, ALTER, CREATE here")
        }
    }
}

/// Parse into SQL Statement AST from separate parts
/// this is useful when using a different crate for the http request
pub fn csv_data_from_parts(
    method: &Method,
    url: &str,
    body: Option<Vec<u8>>,
) -> Result<StmtData<Cursor<Vec<u8>>>, Error> {
    let prefix = method_to_prefix(method);
    let mut prefixed_url_and_body =
        format!("{} {}\n", prefix, url).into_bytes();
    println!(
        "url_with_body: {}",
        String::from_utf8_lossy(&prefixed_url_and_body)
    );
    body.map(|body| prefixed_url_and_body.extend(body));
    Ok(StmtData::from_reader(Cursor::new(prefixed_url_and_body))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Request;
    use percent_encoding::{
        utf8_percent_encode,
        NON_ALPHANUMERIC,
    };
    use restq::{
        ast::{
            ddl::{
                ColumnAttribute,
                ColumnDef,
                DataTypeDef,
            },
            ColumnName,
            TableDef,
            TableLookup,
            TableName,
        },
        DataType,
    };

    #[test]
    fn test_parse_create_statement() {
        let url = "product{*product_id:s32,@name:text,description:text,updated:utc,created_by(users):u32,@is_active:bool}";
        let url = utf8_percent_encode(url, NON_ALPHANUMERIC).to_string();
        let url = format!("http://localhost:8000/{}", url);
        println!("url: {}", url);
        let req = Request::builder()
            .method("PUT")
            .uri(&url)
            .body(
                "1,go pro,a slightly used go pro, 2019-10-31 10:10:10.1\n\
                   2,shovel,a slightly used shovel, 2019-11-11 11:11:11.2\n\
                "
                .to_string(),
            )
            .unwrap();

        let (statement, _rows) = parse_statement(&req).expect("must not fail");

        println!("statement: {:#?}", statement);

        let users_table = TableDef {
            table: TableName {
                name: "users".into(),
            },
            columns: vec![ColumnDef {
                column: ColumnName {
                    name: "user_id".into(),
                },
                attributes: Some(vec![ColumnAttribute::Primary]),
                data_type_def: DataTypeDef {
                    data_type: DataType::U64,
                    is_optional: false,
                    default: None,
                },
                foreign: None,
            }],
        };
        let mut table_lookup = TableLookup::new();
        table_lookup.add_table(users_table);
        assert_eq!(
            statement
                .into_sql_statement(Some(&table_lookup))
                .expect("must not fail")
                .to_string(),
            "CREATE TABLE IF NOT EXISTS product (product_id SERIAL PRIMARY KEY NOT NULL, name text NOT NULL, description text NOT NULL, updated timestamp NOT NULL, created_by int NOT NULL REFERENCES users (user_id), is_active boolean NOT NULL)"
        );
    }

    #[test]
    fn test_parse_select_statement() {
        let url = "person-><-users{name,age,class}?(age=gt.42&student=eq.true)|(gender=eq.`M`&is_active=true)&group_by=sum(age),grade,gender&having=min(age)=gte.42&order_by=age.desc,height.asc&page=2&page_size=10";
        let url = utf8_percent_encode(url, NON_ALPHANUMERIC).to_string();
        let url = format!("http://localhost:8000/{}", url);
        println!("url: {}", url);
        let req = Request::builder()
            .method("GET")
            .uri(&url)
            .body("".to_string())
            .unwrap();
        let (statement, _rows) = parse_statement(&req).expect("must not fail");
        println!("statement: {:#?}", statement);

        let person_table = TableDef {
            table: TableName {
                name: "person".into(),
            },
            columns: vec![ColumnDef {
                column: ColumnName { name: "id".into() },
                attributes: Some(vec![ColumnAttribute::Primary]),
                data_type_def: DataTypeDef {
                    data_type: DataType::S64,
                    is_optional: false,
                    default: None,
                },
                foreign: None,
            }],
        };
        let users_table = TableDef {
            table: TableName {
                name: "users".into(),
            },
            columns: vec![ColumnDef {
                column: ColumnName {
                    name: "person_id".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::U64,
                    is_optional: false,
                    default: None,
                },
                foreign: Some(Foreign {
                    table: TableName {
                        name: "person".into(),
                    },
                    column: None,
                }),
            }],
        };
        let mut table_lookup = TableLookup::new();
        table_lookup.add_table(person_table);
        table_lookup.add_table(users_table);
        assert_eq!(
            statement
                .into_sql_statement(Some(&table_lookup))
                .unwrap()
                .to_string(),
            "SELECT name, age, class FROM person JOIN users ON users.person_id = person.id WHERE (age > 42 AND student = true) OR (gender = 'M' AND is_active = true) GROUP BY sum(age), grade, gender HAVING min(age) >= 42 ORDER BY age DESC, height ASC LIMIT 10 OFFSET 10"
        );
    }
}
