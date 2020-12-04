use crate::ast::{
    ColumnName,
    Function,
    Operator,
    Value,
};
use sql_ast::ast as sql;
use std::fmt;

//TODO: Should be able to do math operations
// such as: *, +, -, /, %
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    ColumnName(ColumnName),
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
            Expr::ColumnName(column) => {
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

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::ColumnName(column) => column.fmt(f),
            Expr::Function(function) => function.fmt(f),
            Expr::Value(value) => value.fmt(f),
            Expr::BinaryOperation(bop) => bop.fmt(f),
            Expr::Nested(expr) => write!(f, "({})", expr),
        }
    }
}

impl fmt::Display for BinaryOperation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.operator.needs_separator() {
            write!(f, "{}={}.{}", self.left, self.operator, self.right)
        } else {
            write!(f, "{}{}{}", self.left, self.operator, self.right)
        }
    }
}

impl fmt::Display for ExprRename {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.expr.fmt(f)?;
        if let Some(rename) = &self.rename {
            write!(f, "=>{}", rename)?;
        }
        Ok(())
    }
}
