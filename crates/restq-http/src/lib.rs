use http::{
    Method,
    Request,
};
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
        tag,
        Parser,
    },
    space,
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

pub enum Prefix {
    Get,
    Put,
    Post,
    Patch,
    Delete,
}

/// Parse into SQL Statement AST from http::Request
pub fn parse_statement(request: &Request<String>) -> Result<Statement, Error> {
    let method = request.method();
    let url = extract_path_and_query(request);
    let body = request.body().as_bytes().to_vec();
    parse_statement_from_parts(&method, url, Some(body))
}

/// Parse into SQL Statement AST from separate parts
/// this is useful when using a different crate for the http request
pub fn parse_statement_from_parts(
    method: &Method,
    url: &str,
    body: Option<Vec<u8>>,
) -> Result<Statement, Error> {
    let url_chars = to_chars(url);
    match *method {
        Method::GET => parse_select_chars(&url_chars).map(Into::into),
        Method::PUT => parse_create_chars(&url_chars).map(Into::into),
        _ => todo!(),
    }
}

fn extract_path_and_query<T>(request: &Request<T>) -> &str {
    request
        .uri()
        .path_and_query()
        .map(|pnq| pnq.as_str())
        .unwrap_or("/")
}

pub fn parse_select<T>(request: &Request<T>) -> Result<Select, Error> {
    let pnq = extract_path_and_query(request);
    let input = to_chars(&pnq);
    parse_select_chars(&input)
}
pub fn parse_create<T>(request: &Request<T>) -> Result<TableDef, Error> {
    let pnq = extract_path_and_query(request);
    let input = to_chars(&pnq);
    parse_create_chars(&input)
}

pub fn parse_select_chars(input: &[char]) -> Result<Select, Error> {
    Ok(url_select().parse(input)?)
}

fn parse_create_chars(input: &[char]) -> Result<TableDef, Error> {
    Ok(url_create().parse(input)?)
}

fn parse_body_to_csv(
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

/// attempt to parse statement
pub fn try_parse_statement(
    url: &str,
    body: Option<Vec<u8>>,
) -> Result<Statement, Error> {
    let url_chars = to_chars(url);
    parse_statement_chars(&url_chars)
}

fn parse_statement_chars(input: &[char]) -> Result<Statement, Error> {
    Ok(statement_with_method_prefix().parse(input)?)
}

fn statement<'a>() -> Parser<'a, char, Statement> {
    url_select().map(Statement::Select)
        | url_insert().map(Statement::Insert)
        | url_create().map(Statement::Create)
        | url_delete().map(Statement::Delete)
        | url_update().map(Statement::Update)
        | url_drop_table().map(Statement::DropTable)
        | url_alter_table().map(Statement::AlterTable)
}

fn post_prefix<'a>() -> Parser<'a, char, Prefix> {
    tag("POST").map(|_| Prefix::Post)
}

fn delete_prefix<'a>() -> Parser<'a, char, Prefix> {
    tag("DELETE").map(|_| Prefix::Delete)
}

fn patch_prefix<'a>() -> Parser<'a, char, Prefix> {
    tag("PATCH").map(|_| Prefix::Patch)
}

fn get_prefix<'a>() -> Parser<'a, char, Prefix> {
    tag("GET").map(|_| Prefix::Get)
}

fn put_prefix<'a>() -> Parser<'a, char, Prefix> {
    tag("PUT").map(|_| Prefix::Put)
}

fn statement_with_method_prefix<'a>() -> Parser<'a, char, Statement> {
    (post_prefix() - space())
        * url_insert()
            .expect("insert after POST")
            .map(Statement::Insert)
        | (put_prefix() - space())
            * url_create()
                .expect("create after PUT")
                .map(Statement::Create)
        | (delete_prefix() - space())
            * (url_drop_table().map(Statement::DropTable)
                | url_delete().map(Statement::Delete))
            .expect("drop table or delete after DELETE")
        | (patch_prefix() - space())
            * (url_alter_table().map(Statement::AlterTable)
                | url_update().map(Statement::Update))
            .expect("alter or update after PATCH")
        | (get_prefix() - space())
            * url_select()
                .map(Statement::Select)
                .expect("a select after GET")
        | (get_prefix() - space()).opt()
            * url_select().map(Statement::Select).expect("a select") // GET in select is optional
}

fn url_select<'a>() -> Parser<'a, char, Select> {
    sym('/') * select() | select()
}

fn url_insert<'a>() -> Parser<'a, char, Insert> {
    sym('/') * insert() | insert()
}

fn url_create<'a>() -> Parser<'a, char, TableDef> {
    sym('/') * table_def() | table_def()
}

fn url_delete<'a>() -> Parser<'a, char, Delete> {
    sym('/') * delete() | delete()
}

fn url_update<'a>() -> Parser<'a, char, Update> {
    sym('/') * update() | update()
}

fn url_drop_table<'a>() -> Parser<'a, char, DropTable> {
    sym('/') * drop_table() | drop_table()
}

fn url_alter_table<'a>() -> Parser<'a, char, AlterTable> {
    sym('/') * alter_table() | alter_table()
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
    fn test_statement_mismatch_prefix_for_statement() {
        let select = try_parse_statement("PUT /product", None);
        assert!(
            select
                .err()
                .unwrap()
                .to_string()
                .contains("Expect create after PUT")
        );

        let create =
            try_parse_statement("GET /product{id:i32,name:text}", None);
        println!("create: {:#?}", create);

        assert!(
            create
                .err()
                .unwrap()
                .to_string()
                .contains("Expect a select after GET")
        );
    }

    #[test]
    fn test_statement_with_method_prefix() {
        let select =
            try_parse_statement("GET /product", None).expect("must be parsed");
        println!("select: {:#?}", select);
        match select {
            Statement::Select(_) => println!("ok"),
            _ => unreachable!(),
        }

        let select_no_prefix =
            try_parse_statement("/product", None).expect("must be parsed");
        println!("select no prefix: {:#?}", select_no_prefix);
        match select_no_prefix {
            Statement::Select(_) => println!("ok"),
            _ => unreachable!(),
        }

        let insert = try_parse_statement("POST /product{id,name}", None)
            .expect("must be parsed");
        println!("insert: {:#?}", insert);
        match insert {
            Statement::Insert(_) => println!("ok"),
            _ => unreachable!(),
        }

        let create =
            try_parse_statement("PUT /product{id:i32,name:text}", None)
                .expect("must be parsed");
        println!("create: {:#?}", create);

        match create {
            Statement::Create(_) => println!("ok"),
            _ => unreachable!(),
        }

        let delete = try_parse_statement("DELETE /product", None)
            .expect("must be parsed");
        println!("delete: {:#?}", delete);
        match delete {
            Statement::Delete(_) => println!("ok"),
            _ => unreachable!(),
        }

        let drop = try_parse_statement("DELETE /-product", None)
            .expect("must be parsed");
        println!("drop: {:#?}", drop);
        match drop {
            Statement::DropTable(_) => println!("ok"),
            _ => unreachable!(),
        }

        let update = try_parse_statement(
            "PATCH /product{name='new name',description='new desc'}",
            None,
        )
        .expect("must be parsed");
        println!("update: {:#?}", update);
        match update {
            Statement::Update(_) => println!("ok"),
            _ => unreachable!(),
        }

        let alter =
            try_parse_statement("PATCH /product{-name,-description}", None)
                .expect("must be parsed");
        println!("alter: {:#?}", alter);
        match alter {
            Statement::AlterTable(_) => println!("ok"),
            _ => unreachable!(),
        }

        let alter_add_column = try_parse_statement(
            "PATCH /product{-name,-description,+discount:f32?(0.0)}",
            None,
        )
        .expect("must be parsed");
        println!("alter add column: {:#?}", alter_add_column);
        match alter_add_column {
            Statement::AlterTable(_) => println!("ok"),
            _ => unreachable!(),
        }
    }

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
        let mut rows = parse_body_to_csv(req, table_def.columns.clone())
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
        let mut table_lookup = TableLookup::new();
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
        let mut table_lookup = TableLookup::new();
        table_lookup.add_table(person_table);
        table_lookup.add_table(users_table);
        assert_eq!(
            statement
                .into_sql_statement(Some(&table_lookup))
                .unwrap()
                .to_string(),
            "SELECT name, age, class FROM person JOIN users ON users.person_id = person.id WHERE (age > 42 AND student = true) OR (gender = 'M' AND is_active = true) GROUP BY sum(age), grade, gender HAVING min(age) >= 42 ORDER BY age DESC, height ASC LIMIT 10 OFFSET 10 ROWS"
        );
    }
}
