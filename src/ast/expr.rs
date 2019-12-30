use crate::ast::{
    Column,
    Function,
    Operator,
    Value,
};
use sqlparser::ast as sql;

//TODO: Should be able to do math operations
// such as: *, +, -, /, %
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Column(Column),
    Function(Function),
    Value(Value),
    BinaryOperation(Box<BinaryOperation>),
    /// The expressions is explicitly
    /// grouped in a parenthesis
    Nested(Box<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExprRename {
    pub expr: Expr,
    pub rename: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryOperation {
    pub left: Expr,
    pub operator: Operator,
    pub right: Expr,
}

impl Into<sql::Expr> for &Expr {
    fn into(self) -> sql::Expr {
        match self {
            Expr::Column(column) => {
                sql::Expr::Identifier(sql::Ident::new(&column.name))
            }
            Expr::Function(function) => {
                sql::Expr::Function(Into::into(function))
            }
            Expr::Value(value) => sql::Expr::Value(Into::into(value)),
            Expr::BinaryOperation(binop) => {
                sql::Expr::BinaryOp {
                    left: Box::new(Into::into(&binop.left)),
                    op: Into::into(&binop.operator),
                    right: Box::new(Into::into(&binop.right)),
                }
            }
            Expr::Nested(expr) => {
                sql::Expr::Nested(Box::new(Into::into(expr.as_ref())))
            }
        }
    }
}
