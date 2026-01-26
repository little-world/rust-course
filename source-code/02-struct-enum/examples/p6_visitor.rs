//! Pattern 6: Visitor Pattern with Enums
//! Example: AST with PrettyPrinter and Evaluator visitors
//!
//! Run with: cargo run --example p6_visitor

use std::collections::HashMap;

// ============================================
// 1. The Data Structure (AST)
// ============================================

// AST for a simple expression language
enum AstExpr {
    Number(f64),
    Variable(String),
    BinaryOp {
        op: BinOp,
        left: Box<AstExpr>,
        right: Box<AstExpr>,
    },
    UnaryOp {
        op: UnOp,
        expr: Box<AstExpr>,
    },
}

enum BinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

enum UnOp {
    Negate,
    Abs,
}

// ============================================
// 2. The Visitor Trait
// ============================================

// Visitor trait - each operation implements this
trait AstVisitor {
    type Output;

    fn visit(&mut self, expr: &AstExpr) -> Self::Output {
        match expr {
            AstExpr::Number(n) => self.visit_number(*n),
            AstExpr::Variable(name) => self.visit_variable(name),
            AstExpr::BinaryOp { op, left, right } => self.visit_binary_op(op, left, right),
            AstExpr::UnaryOp { op, expr } => self.visit_unary_op(op, expr),
        }
    }

    fn visit_number(&mut self, n: f64) -> Self::Output;
    fn visit_variable(&mut self, name: &str) -> Self::Output;
    fn visit_binary_op(
        &mut self,
        op: &BinOp,
        left: &AstExpr,
        right: &AstExpr,
    ) -> Self::Output;
    fn visit_unary_op(&mut self, op: &UnOp, expr: &AstExpr) -> Self::Output;
}

// ============================================
// 3. Visitor Implementations
// ============================================

// Pretty printer visitor
struct PrettyPrinter;

impl AstVisitor for PrettyPrinter {
    type Output = String;

    fn visit_number(&mut self, n: f64) -> String {
        n.to_string()
    }

    fn visit_variable(&mut self, name: &str) -> String {
        name.to_string()
    }

    fn visit_binary_op(&mut self, op: &BinOp, left: &AstExpr, right: &AstExpr) -> String {
        let op_str = match op {
            BinOp::Add => "+",
            BinOp::Subtract => "-",
            BinOp::Multiply => "*",
            BinOp::Divide => "/",
        };
        let l = self.visit(left);
        let r = self.visit(right);
        format!("({} {} {})", l, op_str, r)
    }

    fn visit_unary_op(&mut self, op: &UnOp, expr: &AstExpr) -> String {
        let op_str = match op {
            UnOp::Negate => "-",
            UnOp::Abs => "abs",
        };
        format!("{}({})", op_str, self.visit(expr))
    }
}

// Evaluator visitor
struct Evaluator {
    variables: HashMap<String, f64>,
}

impl AstVisitor for Evaluator {
    type Output = Result<f64, String>;

    fn visit_number(&mut self, n: f64) -> Self::Output {
        Ok(n)
    }

    fn visit_variable(&mut self, name: &str) -> Self::Output {
        self.variables
            .get(name)
            .copied()
            .ok_or_else(|| format!("Undefined variable: {}", name))
    }

    fn visit_binary_op(&mut self, op: &BinOp, left: &AstExpr, right: &AstExpr) -> Self::Output {
        let l = self.visit(left)?;
        let r = self.visit(right)?;
        match op {
            BinOp::Add => Ok(l + r),
            BinOp::Subtract => Ok(l - r),
            BinOp::Multiply => Ok(l * r),
            BinOp::Divide => {
                if r == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(l / r)
                }
            }
        }
    }

    fn visit_unary_op(&mut self, op: &UnOp, expr: &AstExpr) -> Self::Output {
        let val = self.visit(expr)?;
        match op {
            UnOp::Negate => Ok(-val),
            UnOp::Abs => Ok(val.abs()),
        }
    }
}

fn main() {
    println!("=== Visitor Pattern Demo ===\n");

    // Build an AST: 2 + 3
    let expr1 = AstExpr::BinaryOp {
        op: BinOp::Add,
        left: Box::new(AstExpr::Number(2.0)),
        right: Box::new(AstExpr::Number(3.0)),
    };

    // Pretty print
    let mut printer = PrettyPrinter;
    let formatted = printer.visit(&expr1);
    println!("Expression: {}", formatted);
    assert_eq!(formatted, "(2 + 3)");

    // Evaluate
    let mut eval = Evaluator {
        variables: HashMap::new(),
    };
    let result = eval.visit(&expr1);
    println!("Result: {:?}", result);
    assert_eq!(result, Ok(5.0));

    // More complex: x * 10 where x = 5
    println!("\n--- Expression with variable ---");
    let expr_with_var = AstExpr::BinaryOp {
        op: BinOp::Multiply,
        left: Box::new(AstExpr::Variable("x".to_string())),
        right: Box::new(AstExpr::Number(10.0)),
    };

    println!("Expression: {}", printer.visit(&expr_with_var));

    let mut eval_with_vars = Evaluator {
        variables: HashMap::from([("x".to_string(), 5.0)]),
    };
    let result = eval_with_vars.visit(&expr_with_var);
    println!("With x=5: {:?}", result);
    assert_eq!(result, Ok(50.0));

    // Unary operation: abs(-42)
    println!("\n--- Unary operation ---");
    let unary_expr = AstExpr::UnaryOp {
        op: UnOp::Abs,
        expr: Box::new(AstExpr::UnaryOp {
            op: UnOp::Negate,
            expr: Box::new(AstExpr::Number(42.0)),
        }),
    };
    println!("Expression: {}", printer.visit(&unary_expr));
    println!("Result: {:?}", eval.visit(&unary_expr));

    // Error case: undefined variable
    println!("\n--- Error handling ---");
    let undefined = AstExpr::Variable("undefined_var".to_string());
    let err_result = eval.visit(&undefined);
    println!("Undefined variable result: {:?}", err_result);

    // Complex expression: (a + b) * (c - d)
    println!("\n--- Complex expression ---");
    let complex = AstExpr::BinaryOp {
        op: BinOp::Multiply,
        left: Box::new(AstExpr::BinaryOp {
            op: BinOp::Add,
            left: Box::new(AstExpr::Variable("a".to_string())),
            right: Box::new(AstExpr::Variable("b".to_string())),
        }),
        right: Box::new(AstExpr::BinaryOp {
            op: BinOp::Subtract,
            left: Box::new(AstExpr::Variable("c".to_string())),
            right: Box::new(AstExpr::Variable("d".to_string())),
        }),
    };

    println!("Expression: {}", printer.visit(&complex));

    let mut eval_complex = Evaluator {
        variables: HashMap::from([
            ("a".to_string(), 2.0),
            ("b".to_string(), 3.0),
            ("c".to_string(), 10.0),
            ("d".to_string(), 4.0),
        ]),
    };
    println!("With a=2, b=3, c=10, d=4: {:?}", eval_complex.visit(&complex));
    // (2 + 3) * (10 - 4) = 5 * 6 = 30
}
