use http::{
    Method,
    Request,
};
pub use restq::{
    ast::{
        ddl::{
            table_def,
            ColumnDef,
        },
        Delete,
        Insert,
        Select,
        Statement,
        TableDef,
        Update,
    },
    csv_data::CsvRows,
    parser::select,
    pom::parser::{
        sym,
        Parser,
    },
    to_chars,
    DataValue,
    Error,
};
use std::io::{
    BufReader,
    Cursor,
    Read,
};
use thiserror::Error;

pub fn parse_statement(request: &Request<String>) -> Result<Statement, Error> {
    match *request.method() {
        Method::GET => parse_select(request).map(Into::into),
        Method::PUT => parse_create(request).map(Into::into),
        _ => panic!("not yet"),
    }
}

fn extract_path_and_query<T>(request: &Request<T>) -> Option<&str> {
    request.uri().path_and_query().map(|pnq| pnq.as_str())
}

pub fn parse_select(request: &Request<String>) -> Result<Select, Error> {
    let pnq = extract_path_and_query(request).unwrap();
    let input = to_chars(&pnq);
    parse_select_chars(&input)
}

fn parse_select_chars(input: &[char]) -> Result<Select, Error> {
    Ok(req_select().parse(input)?)
}

fn req_select<'a>() -> Parser<'a, char, Select> {
    sym('/') * select()
}

pub fn parse_create<T>(request: &Request<T>) -> Result<TableDef, Error> {
    let pnq = extract_path_and_query(request).unwrap();
    let input = to_chars(&pnq);
    parse_create_chars(&input)
}

fn parse_req_body(
    request: Request<Vec<u8>>,
    column_defs: Vec<ColumnDef>,
) -> Result<impl Iterator<Item = Vec<DataValue>>, Error> {
    let data = request.into_body();
    parse_csv_data(data, column_defs)
}

fn parse_csv_data(
    data: Vec<u8>,
    column_defs: Vec<ColumnDef>,
) -> Result<impl Iterator<Item = Vec<DataValue>>, Error> {
    println!("data: {}", String::from_utf8_lossy(&data));
    let csv_rows = CsvRows::new(BufReader::new(Cursor::new(data)), column_defs);
    Ok(csv_rows)
}

fn parse_create_chars(input: &[char]) -> Result<TableDef, Error> {
    Ok(req_table_def().parse(input)?)
}

fn req_table_def<'a>() -> Parser<'a, char, TableDef> {
    sym('/') * table_def()
}

#[cfg(test)]
mod tests {
    use super::*;
    use restq::{
        ast::{
            ddl::*,
            *,
        },
        *,
    };

    #[test]
    fn test_parse_create_statement_with_rows() {
        let req = Request::builder()
            .method("PUT")
            .uri("https://localhost:8000/product(*product_id:s32,@name:text,description:text,updated:utc,created_by(users):u32,@is_active:bool)")
            .body(b"1,go pro,a slightly used go pro, 2019-10-31 10:10:10\n\
                   2,shovel,a slightly used shovel, 2019-11-11 11:11:11\n\
                ".to_vec())
            .unwrap();

        let table_def = parse_create(&req).expect("must not fail");

        println!("statement: {:#?}", table_def);
        let mut rows = parse_req_body(req, table_def.columns.clone())
            .expect("must have rows");

        let row1 = rows.next().unwrap();
        let row2 = rows.next().unwrap();
        assert_eq!(row1[0], DataValue::S32(1));
        assert_eq!(row2[0], DataValue::S32(2));
        assert_eq!(row2[1], DataValue::Text("shovel".to_string()));
    }

    #[test]
    fn test_parse_create_statement() {
        let req = Request::builder()
            .method("PUT")
            .uri("https://localhost:8000/product(*product_id:s32,@name:text,description:text,updated:utc,created_by(users):u32,@is_active:bool)")
            .body("1,go pro,a slightly used go pro, 2019-10-31 10:10:10\
                   2,shovel,a slightly used shovel, 2019-11-11 11:11:11\
                ".to_string())
            .unwrap();

        let statement = parse_statement(&req).expect("must not fail");

        println!("statement: {:#?}", statement);

        let users_table = TableDef {
            table: Table {
                name: "users".into(),
            },
            columns: vec![ColumnDef {
                column: Column {
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
        let mut table_lookup = TableLookup::new(); //no table lookup
        table_lookup.add_table(users_table);
        assert_eq!(
            statement
                .into_sql_statement(Some(&table_lookup))
                .expect("must not fail")
                .to_string(),
            "CREATE TABLE product (product_id int PRIMARY KEY NOT NULL, name text NOT NULL, description text NOT NULL, updated timestamp NOT NULL, created_by int NOT NULL REFERENCES users (user_id), is_active boolean NOT NULL)"
        );
    }

    #[test]
    fn test_parse_select_statement() {
        let req = Request::builder()
            .method("GET")
            .uri("https://localhost:8000/person-^^-users(name,age,class)?(age=gt.42&student=eq.true)|(gender=eq.`M`&is_active=true)&group_by=sum(age),grade,gender&having=min(age)=gte.42&order_by=age.desc,height.asc&page=2&page_size=10")
            .body("body rocks".to_string())
            .unwrap();
        let statement = parse_statement(&req).expect("must not fail");
        println!("statement: {:#?}", statement);

        let person_table = TableDef {
            table: Table {
                name: "person".into(),
            },
            columns: vec![ColumnDef {
                column: Column { name: "id".into() },
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
            table: Table {
                name: "users".into(),
            },
            columns: vec![ColumnDef {
                column: Column {
                    name: "person_id".into(),
                },
                attributes: None,
                data_type_def: DataTypeDef {
                    data_type: DataType::U64,
                    is_optional: false,
                    default: None,
                },
                foreign: Some(Table {
                    name: "person".into(),
                }),
            }],
        };
        let mut table_lookup = TableLookup::new(); //no table lookup
        table_lookup.add_table(person_table);
        table_lookup.add_table(users_table);
        assert_eq!(
            statement.into_sql_statement(Some(&table_lookup)).unwrap().to_string(),
            "SELECT name, age, class FROM person JOIN users ON users.person_id = person.id WHERE (age > 42 AND student = true) OR (gender = 'M' AND is_active = true) GROUP BY sum(age), grade, gender HAVING min(age) >= 42 ORDER BY age DESC, height ASC LIMIT 10 OFFSET 10 ROWS"
        );
    }
}
