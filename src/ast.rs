pub mod ddl;
pub mod dml;
mod expr;
mod operator;
pub mod parser;
mod table;

use crate::Error;
pub use ddl::{
    AlterTable,
    DropTable,
    TableDef,
};
pub use dml::{
    BulkDelete,
    BulkUpdate,
    Delete,
    Insert,
    Update,
};
pub use expr::{
    BinaryOperation,
    Expr,
    ExprRename,
};
pub use operator::Operator;
use sql_ast::ast as sql;
use std::fmt;
pub use table::{
    FromTable,
    JoinType,
    Table,
    TableError,
    TableLookup,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Select(Select),
    Insert(Insert),
    Update(Update),
    BulkUpdate(BulkUpdate),
    Delete(Delete),
    BulkDelete(BulkDelete),
    Create(TableDef),
    DropTable(DropTable),
    AlterTable(AlterTable),
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Select {
    pub from_table: FromTable,
    pub filter: Option<Expr>,
    pub group_by: Option<Vec<Expr>>,
    pub having: Option<Expr>,
    pub projection: Option<Vec<ExprRename>>, // column selection
    pub order_by: Option<Vec<Order>>,
    pub range: Option<Range>,
}

#[derive(Debug, PartialEq, Default, Clone, PartialOrd, Hash, Eq, Ord)]
pub struct Column {
    pub name: String,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Expr>,
}

/// coarse value from the parsing
/// this is close to the json values
///
/// TODO: include Integer variant
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Null,
    String(String),
    Number(f64),
    Bool(bool),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Order {
    pub expr: Expr,
    pub direction: Option<Direction>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Direction {
    Asc,
    Desc,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Range {
    Page(Page),
    Limit(Limit),
}

impl Range {
    pub(crate) fn limit(&self) -> i64 {
        match self {
            Range::Page(page) => page.page_size,
            Range::Limit(limit) => limit.limit,
        }
    }

    pub(crate) fn offset(&self) -> Option<i64> {
        match self {
            Range::Page(page) => Some((page.page - 1) * page.page_size),
            Range::Limit(limit) => limit.offset,
        }
    }
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Page {
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Limit {
    pub limit: i64,
    pub offset: Option<i64>,
}

impl Statement {
    pub fn into_sql_statement(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<sql::Statement, Error> {
        match self {
            Statement::Select(select) => {
                select.into_sql_statement(table_lookup)
            }
            Statement::Insert(insert) => {
                insert.into_sql_statement(table_lookup)
            }
            Statement::Update(update) => Ok(Into::into(update)),
            Statement::Delete(delete) => Ok(Into::into(delete)),
            Statement::BulkUpdate(_update) => todo!(),
            Statement::BulkDelete(_delete) => todo!(),
            Statement::Create(create) => {
                Ok(create.into_sql_statement(table_lookup)?)
            }
            Statement::DropTable(drop_table) => Ok(Into::into(drop_table)),
            Statement::AlterTable(alter_table) => {
                let mut statements =
                    alter_table.into_sql_statements(table_lookup)?;
                if statements.len() == 1 {
                    Ok(statements.remove(0))
                } else {
                    Err(Error::MoreThanOneStatement)
                }
            }
        }
    }
}

impl Into<Statement> for Select {
    fn into(self) -> Statement {
        Statement::Select(self)
    }
}

impl Select {
    pub fn set_page(&mut self, page: i64, page_size: i64) {
        self.range = Some(Range::Page(Page { page, page_size }));
    }

    pub fn get_page(&self) -> Option<i64> {
        if let Some(Range::Page(page)) = &self.range {
            Some(page.page)
        } else {
            None
        }
    }

    pub fn get_page_size(&self) -> Option<i64> {
        if let Some(Range::Page(page)) = &self.range {
            Some(page.page_size)
        } else {
            None
        }
    }

    pub fn add_simple_filter(
        &mut self,
        column: Column,
        operator: Operator,
        search_key: &str,
    ) {
        let simple_filter = Expr::BinaryOperation(Box::new(BinaryOperation {
            left: Expr::Column(column),
            operator,
            right: Expr::Value(Value::String(search_key.to_string())),
        }));

        //TODO: need to deal with existing filters
        self.filter = Some(simple_filter);
    }

    pub fn into_sql_select(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<sql::Select, Error> {
        let select = sql::Select {
            distinct: false,
            projection: if let Some(projection) = self.projection.as_ref() {
                projection
                    .iter()
                    .map(|proj| {
                        if let Some(rename) = &proj.rename {
                            sql::SelectItem::ExprWithAlias {
                                expr: Into::into(&proj.expr),
                                alias: sql::Ident::new(rename),
                            }
                        } else {
                            sql::SelectItem::UnnamedExpr(Into::into(&proj.expr))
                        }
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![sql::SelectItem::Wildcard]
            },
            from: vec![self.from_table.into_table_with_joins(table_lookup)?],
            selection: self.filter.as_ref().map(|expr| Into::into(expr)),
            group_by: match &self.group_by {
                Some(group_by) => {
                    group_by.iter().map(|expr| Into::into(expr)).collect()
                }
                None => vec![],
            },
            having: self.having.as_ref().map(|expr| Into::into(expr)),
        };
        Ok(select)
    }

    pub fn into_sql_query(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<sql::Query, Error> {
        let query = sql::Query {
            ctes: vec![],
            body: sql::SetExpr::Select(Box::new(
                self.into_sql_select(table_lookup)?,
            )),
            order_by: match &self.order_by {
                Some(order_by) => {
                    order_by.iter().map(|expr| Into::into(expr)).collect()
                }
                None => vec![],
            },
            limit: self.range.as_ref().map(|range| {
                sql::Expr::Value(sql::Value::Number(range.limit().to_string()))
            }),
            offset: match &self.range {
                Some(range) => {
                    range.offset().map(|offset| {
                        sql::Expr::Value(sql::Value::Number(offset.to_string()))
                    })
                }
                None => None,
            },
            fetch: None,
        };

        Ok(query)
    }

    pub fn into_sql_statement(
        &self,
        table_lookup: Option<&TableLookup>,
    ) -> Result<sql::Statement, Error> {
        Ok(sql::Statement::Query(Box::new(
            self.into_sql_query(table_lookup)?,
        )))
    }
}

impl fmt::Display for Select {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.from_table.fmt(f)?;
        if let Some(projection) = &self.projection {
            write!(f, "(")?;
            for (i, exprr) in projection.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                exprr.fmt(f)?;
            }
            write!(f, ")")?;
        }

        if let Some(filter) = &self.filter {
            write!(f, "?")?;
            filter.fmt(f)?;
        }

        if let Some(group_by) = &self.group_by {
            write!(f, "&group_by=")?;
            for (i, expr) in group_by.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                expr.fmt(f)?;
            }
        }

        if let Some(having) = &self.having {
            write!(f, "&having=")?;
            having.fmt(f)?;
        }
        if let Some(order_by) = &self.order_by {
            write!(f, "&order_by=")?;
            for (i, ord) in order_by.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                ord.fmt(f)?;
            }
        }
        if let Some(range) = &self.range {
            write!(f, "&")?;
            range.fmt(f)?;
        }

        Ok(())
    }
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Asc
    }
}

impl Into<sql::Function> for &Function {
    fn into(self) -> sql::Function {
        sql::Function {
            name: sql::ObjectName(vec![sql::Ident::new(&self.name)]),
            args: self.params.iter().map(|expr| Into::into(expr)).collect(),
            over: None,
            distinct: false,
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", self.name)?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", param)?;
        }
        write!(f, ")")
    }
}

impl Into<sql::Ident> for &Column {
    fn into(self) -> sql::Ident {
        sql::Ident::new(&self.name)
    }
}

impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Into<sql::Value> for &Value {
    fn into(self) -> sql::Value {
        match self {
            Value::Null => sql::Value::Null,
            Value::String(v) => sql::Value::SingleQuotedString(v.to_string()),
            Value::Number(v) => sql::Value::Number(format!("{}", v)),
            Value::Bool(v) => sql::Value::Boolean(*v),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::String(v) => write!(f, "'{}'", v),
            Value::Number(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
        }
    }
}

impl Into<sql::OrderByExpr> for &Order {
    fn into(self) -> sql::OrderByExpr {
        sql::OrderByExpr {
            expr: Into::into(&self.expr),
            asc: self.direction.as_ref().map(|direction| {
                match direction {
                    Direction::Asc => true,
                    Direction::Desc => false,
                }
            }),
        }
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.expr.fmt(f)?;
        if let Some(direction) = &self.direction {
            write!(f, ".")?;
            direction.fmt(f)?;
        }
        Ok(())
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Direction::Asc => write!(f, "asc"),
            Direction::Desc => write!(f, "desc"),
        }
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Range::Page(page) => page.fmt(f),
            Range::Limit(limit) => limit.fmt(f),
        }
    }
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "page={}&page_size={}", self.page, self.page_size)
    }
}

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "limit={}", self.limit)?;
        if let Some(offset) = &self.offset {
            write!(f, "&offset={}", offset)?;
        }
        Ok(())
    }
}
