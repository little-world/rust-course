//! Pattern 5: Advanced Enum Techniques
//! Example: Recursive Enums with Box
//!
//! Run with: cargo run --example p5_recursive_enum

// Binary tree - recursive enum needs Box to break infinite size
enum Tree<T> {
    Leaf(T),
    Node {
        value: T,
        left: Box<Tree<T>>,
        right: Box<Tree<T>>,
    },
}

impl<T: std::fmt::Debug> Tree<T> {
    fn depth(&self) -> usize {
        match self {
            Tree::Leaf(_) => 1,
            Tree::Node { left, right, .. } => 1 + left.depth().max(right.depth()),
        }
    }

    fn print_inorder(&self) {
        match self {
            Tree::Leaf(v) => print!("{:?} ", v),
            Tree::Node { value, left, right } => {
                left.print_inorder();
                print!("{:?} ", value);
                right.print_inorder();
            }
        }
    }
}

// AST nodes often use Box for recursion
enum Expr {
    Number(i32),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

impl Expr {
    fn eval(&self) -> i32 {
        match self {
            Expr::Number(n) => *n,
            Expr::Add(l, r) => l.eval() + r.eval(),
            Expr::Mul(l, r) => l.eval() * r.eval(),
        }
    }

    fn to_string(&self) -> String {
        match self {
            Expr::Number(n) => n.to_string(),
            Expr::Add(l, r) => format!("({} + {})", l.to_string(), r.to_string()),
            Expr::Mul(l, r) => format!("({} * {})", l.to_string(), r.to_string()),
        }
    }
}

fn main() {
    // Build a binary tree
    //       5
    //      / \
    //     3   7
    //    / \
    //   1   4
    let tree = Tree::Node {
        value: 5,
        left: Box::new(Tree::Node {
            value: 3,
            left: Box::new(Tree::Leaf(1)),
            right: Box::new(Tree::Leaf(4)),
        }),
        right: Box::new(Tree::Leaf(7)),
    };

    println!("Tree depth: {}", tree.depth());
    print!("Inorder traversal: ");
    tree.print_inorder();
    println!();

    // Build an expression: (2 + 3) * 4
    let expr = Expr::Mul(
        Box::new(Expr::Add(
            Box::new(Expr::Number(2)),
            Box::new(Expr::Number(3)),
        )),
        Box::new(Expr::Number(4)),
    );

    println!("\nExpression: {}", expr.to_string());
    println!("Result: {}", expr.eval());
    assert_eq!(expr.eval(), 20); // (2 + 3) * 4 = 20

    // More complex expression: (1 + 2) * (3 + 4)
    let complex = Expr::Mul(
        Box::new(Expr::Add(
            Box::new(Expr::Number(1)),
            Box::new(Expr::Number(2)),
        )),
        Box::new(Expr::Add(
            Box::new(Expr::Number(3)),
            Box::new(Expr::Number(4)),
        )),
    );
    println!("\nExpression: {}", complex.to_string());
    println!("Result: {}", complex.eval()); // (1 + 2) * (3 + 4) = 21
}
