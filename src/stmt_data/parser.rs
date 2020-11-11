use crate::{
    ast::{
        ddl::{alter_table, drop_table, table_def},
        dml::{delete, insert, update},
        Select, Statement,
    },
    parser::{utils::space, *},
    to_chars,
};
use pom::parser::*;
pub enum Prefix {
    Get,
    Put,
    Post,
    Patch,
    Delete,
}

#[allow(unused)]
pub fn parse_statement(url: &str) -> Result<Statement, crate::Error> {
    let url_chars = to_chars(url);
    parse_statement_chars(&url_chars)
}

pub(crate) fn parse_statement_chars(
    input: &[char],
) -> Result<Statement, crate::Error> {
    Ok(statement_with_prefix().parse(input)?)
}

/// parses a typical http url into a select statement
pub fn parse_select_chars(input: &[char]) -> Result<Select, crate::Error> {
    let url_parser = sym('/') * select();
    Ok(url_parser.parse(input)?)
}

fn statement_with_prefix<'a>() -> Parser<'a, char, Statement> {
    (post_prefix() - space().opt() - sym('/'))
        * insert().map(Statement::Insert).expect("insert after POST")
        | (put_prefix() - space().opt() - sym('/') - sym('+').opt())
            * table_def()
                .map(Statement::Create)
                .expect("create after PUT")
        | (delete_prefix() - space().opt() - sym('/'))
            * (drop_table().map(Statement::DropTable)
                | delete().map(Statement::Delete))
            .expect("drop table or delete after DELETE")
        | (patch_prefix() - space().opt() - sym('/'))
            * (alter_table().map(Statement::AlterTable)
                | update().map(Statement::Update))
            .expect("alter or update after PATCH")
        | (get_prefix() - space().opt() - sym('/'))
            * select().map(Statement::Select).expect("a select after GET")
        | sym('/')
            * select()
                .map(Statement::Select)
                .expect("only select is allowed for no prefix")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statement_with_method_prefix() {
        let select = parse_statement("GET /product").expect("must be parsed");
        println!("select: {:#?}", select);
        match select {
            Statement::Select(_) => println!("ok"),
            _ => unreachable!(),
        }

        let select_no_prefix =
            parse_statement("/product").expect("must be parsed");
        println!("select no prefix: {:#?}", select_no_prefix);
        match select_no_prefix {
            Statement::Select(_) => println!("ok"),
            _ => unreachable!(),
        }

        let insert =
            parse_statement("POST /product{id,name}").expect("must be parsed");
        println!("insert: {:#?}", insert);
        match insert {
            Statement::Insert(_) => println!("ok"),
            _ => unreachable!(),
        }

        let create = parse_statement("PUT /product{id:i32,name:text}")
            .expect("must be parsed");
        println!("create: {:#?}", create);

        match create {
            Statement::Create(_) => println!("ok"),
            _ => unreachable!(),
        }

        let create_plus = parse_statement("PUT /+product{id:i32,name:text}")
            .expect("must be parsed");
        println!("create: {:#?}", create_plus);

        match create_plus {
            Statement::Create(_) => println!("ok"),
            _ => unreachable!(),
        }

        let delete =
            parse_statement("DELETE /product").expect("must be parsed");
        println!("delete: {:#?}", delete);
        match delete {
            Statement::Delete(_) => println!("ok"),
            _ => unreachable!(),
        }

        let drop = parse_statement("DELETE /-product").expect("must be parsed");
        println!("drop: {:#?}", drop);
        match drop {
            Statement::DropTable(_) => println!("ok"),
            _ => unreachable!(),
        }

        let update = parse_statement(
            "PATCH /product{name='new name',description='new desc'}",
        )
        .expect("must be parsed");
        println!("update: {:#?}", update);
        match update {
            Statement::Update(_) => println!("ok"),
            _ => unreachable!(),
        }

        let alter = parse_statement("PATCH /product{-name,-description}")
            .expect("must be parsed");
        println!("alter: {:#?}", alter);
        match alter {
            Statement::AlterTable(_) => println!("ok"),
            _ => unreachable!(),
        }

        let alter_add_column = parse_statement(
            "PATCH /product{-name,-description,+discount:f32?(0.0)}",
        )
        .expect("must be parsed");
        println!("alter add column: {:#?}", alter_add_column);
        match alter_add_column {
            Statement::AlterTable(_) => println!("ok"),
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_statement_with_method_prefix_no_space() {
        let select = parse_statement("GET/product").expect("must be parsed");
        println!("select: {:#?}", select);
        match select {
            Statement::Select(_) => println!("ok"),
            _ => unreachable!(),
        }

        let select_no_prefix =
            parse_statement("/product").expect("must be parsed");
        println!("select no prefix: {:#?}", select_no_prefix);
        match select_no_prefix {
            Statement::Select(_) => println!("ok"),
            _ => unreachable!(),
        }

        let insert =
            parse_statement("POST/product{id,name}").expect("must be parsed");
        println!("insert: {:#?}", insert);
        match insert {
            Statement::Insert(_) => println!("ok"),
            _ => unreachable!(),
        }

        let create = parse_statement("PUT/product{id:i32,name:text}")
            .expect("must be parsed");
        println!("create: {:#?}", create);

        match create {
            Statement::Create(_) => println!("ok"),
            _ => unreachable!(),
        }

        let create_plus = parse_statement("PUT/+product{id:i32,name:text}")
            .expect("must be parsed");
        println!("create: {:#?}", create_plus);

        match create_plus {
            Statement::Create(_) => println!("ok"),
            _ => unreachable!(),
        }

        let delete = parse_statement("DELETE/product").expect("must be parsed");
        println!("delete: {:#?}", delete);
        match delete {
            Statement::Delete(_) => println!("ok"),
            _ => unreachable!(),
        }

        let drop = parse_statement("DELETE/-product").expect("must be parsed");
        println!("drop: {:#?}", drop);
        match drop {
            Statement::DropTable(_) => println!("ok"),
            _ => unreachable!(),
        }

        let update = parse_statement(
            "PATCH/product{name='new name',description='new desc'}",
        )
        .expect("must be parsed");
        println!("update: {:#?}", update);
        match update {
            Statement::Update(_) => println!("ok"),
            _ => unreachable!(),
        }

        let alter = parse_statement("PATCH/product{-name,-description}")
            .expect("must be parsed");
        println!("alter: {:#?}", alter);
        match alter {
            Statement::AlterTable(_) => println!("ok"),
            _ => unreachable!(),
        }

        let alter_add_column = parse_statement(
            "PATCH/product{-name,-description,+discount:f32?(0.0)}",
        )
        .expect("must be parsed");
        println!("alter add column: {:#?}", alter_add_column);
        match alter_add_column {
            Statement::AlterTable(_) => println!("ok"),
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_select_and_paging() {
        let select = parse_statement("/product&page=1&page_size=20")
            .expect("must be parsed");
        println!("select: {:#?}", select);
        match select {
            Statement::Select(_) => println!("ok"),
            _ => unreachable!(),
        }
    }
}
