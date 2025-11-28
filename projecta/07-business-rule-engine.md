# Project 3: Business Rule Engine with Enum-Driven Architecture

## Overview

Build a flexible business rule engine that evaluates complex conditions and actions using Rust's pattern matching capabilities. This project demonstrates advanced enum design, exhaustive matching, nested patterns, and trait-based polymorphism—all core to idiomatic Rust.

**Focus Areas:**
- Exhaustive enum matching for all rule types and conditions
- Deep destructuring for nested expressions
- Pattern guards for validation and optimization
- Or-patterns for grouping similar cases
- If-let chains for complex conditional logic
- matches! macro for quick type checks
- Let-else patterns for early returns
- Enum-driven architecture with type-safe state machines

**Real-World Application:** This pattern is used in workflow engines, policy enforcement systems, pricing calculators, fraud detection, and eligibility determination systems.

## Learning Objectives

By completing this project, you will:

1. Master exhaustive enum matching across complex domain models
2. Use pattern guards to express business logic conditions
3. Implement deep destructuring for nested rule evaluation
4. Design enum-driven architectures that enforce correctness at compile time
5. Apply or-patterns and if-let chains effectively
6. Build type-safe state machines with pattern matching
7. Create extensible rule systems without runtime errors

## Project Structure

```
business-rule-engine/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── rule.rs           # Rule definitions and AST
│   ├── condition.rs      # Condition evaluation
│   ├── action.rs         # Action execution
│   ├── context.rs        # Execution context
│   ├── engine.rs         # Rule engine
│   └── examples/
│       ├── pricing.rs    # Dynamic pricing rules
│       ├── approval.rs   # Approval workflow
│       └── fraud.rs      # Fraud detection
├── tests/
│   ├── integration.rs
│   └── examples.rs
└── benches/
    └── evaluation.rs
```

## Milestone 1: Core Data Model with Exhaustive Matching

### Goal
Define the core data model for values, conditions, and rules using enums. Implement exhaustive pattern matching for value comparison and type checking.

### Implementation Steps

#### Step 1.1: Define Value Types

Create `src/context.rs`:

```rust
use std::collections::HashMap;
use std::fmt;

// TODO: Define Value enum representing all possible value types
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    // TODO: Add variants for different value types
    // Hint: Integer(i64)
    // Hint: Float(f64)
    // Hint: String(String)
    // Hint: Boolean(bool)
    // Hint: List(Vec<Value>)
    // Hint: Object(HashMap<String, Value>)
    // Hint: Null
}

impl Value {
    // TODO: Implement type checking using pattern matching
    pub fn is_integer(&self) -> bool {
        // Pseudocode:
        // matches!(self, Value::Integer(_))
        todo!()
    }

    pub fn is_float(&self) -> bool {
        // Pseudocode:
        // matches!(self, Value::Float(_))
        todo!()
    }

    pub fn is_string(&self) -> bool {
        // Pseudocode:
        // matches!(self, Value::String(_))
        todo!()
    }

    pub fn is_boolean(&self) -> bool {
        // Pseudocode:
        // matches!(self, Value::Boolean(_))
        todo!()
    }

    pub fn is_null(&self) -> bool {
        // Pseudocode:
        // matches!(self, Value::Null)
        todo!()
    }

    // TODO: Implement numeric conversion with exhaustive matching
    pub fn as_number(&self) -> Option<f64> {
        // Pseudocode:
        // match self:
        //     Value::Integer(i) => Some(*i as f64)
        //     Value::Float(f) => Some(*f)
        //     _ => None
        todo!()
    }

    // TODO: Implement string extraction with let-else pattern
    pub fn as_string(&self) -> Option<&str> {
        // Pseudocode:
        // let Value::String(s) = self else { return None };
        // Some(s.as_str())
        todo!()
    }

    // TODO: Implement boolean extraction
    pub fn as_bool(&self) -> Option<bool> {
        // Pseudocode:
        // match self:
        //     Value::Boolean(b) => Some(*b)
        //     _ => None
        todo!()
    }

    // TODO: Implement truthiness checking using exhaustive match
    pub fn is_truthy(&self) -> bool {
        // Pseudocode:
        // match self:
        //     Value::Null => false
        //     Value::Boolean(b) => *b
        //     Value::Integer(0) | Value::Float(f) if *f == 0.0 => false
        //     Value::String(s) if s.is_empty() => false
        //     Value::List(v) if v.is_empty() => false
        //     _ => true
        todo!()
    }
}

// TODO: Define Context for storing variables and state
#[derive(Debug, Clone, Default)]
pub struct Context {
    // TODO: Add fields
    // Hint: variables: HashMap<String, Value>
}

impl Context {
    pub fn new() -> Self {
        // Pseudocode:
        // Self { variables: HashMap::new() }
        todo!()
    }

    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        // Pseudocode:
        // self.variables.insert(key.into(), value);
        todo!()
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        // Pseudocode:
        // self.variables.get(key)
        todo!()
    }

    // TODO: Get with default value using pattern matching
    pub fn get_or(&self, key: &str, default: &Value) -> &Value {
        // Pseudocode:
        // self.variables.get(key).unwrap_or(default)
        todo!()
    }

    // TODO: Extend context with another context
    pub fn extend(&mut self, other: &Context) {
        // Pseudocode:
        // for (key, value) in &other.variables:
        //     self.variables.insert(key.clone(), value.clone());
        todo!()
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Format value based on type using exhaustive match
        // Pseudocode:
        // match self:
        //     Value::Integer(i) => write!(f, "{}", i)
        //     Value::Float(fl) => write!(f, "{}", fl)
        //     Value::String(s) => write!(f, "\"{}\"", s)
        //     Value::Boolean(b) => write!(f, "{}", b)
        //     Value::Null => write!(f, "null")
        //     Value::List(items) => format list with brackets
        //     Value::Object(map) => format object with braces
        todo!()
    }
}
```

#### Step 1.2: Define Comparison Operations

Create `src/condition.rs`:

```rust
use crate::context::{Context, Value};

// TODO: Define comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    // TODO: Add comparison operators
    // Hint: Equal, NotEqual, LessThan, LessThanOrEqual, GreaterThan, GreaterThanOrEqual
}

// TODO: Define logical operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOp {
    // TODO: Add logical operators
    // Hint: And, Or
}

// TODO: Define Condition enum for all condition types
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    // TODO: Add condition variants
    // Hint: Literal(bool) - for constant true/false
    // Hint: Compare { op: CompareOp, left: Expr, right: Expr }
    // Hint: Logical { op: LogicalOp, conditions: Vec<Condition> }
    // Hint: Not(Box<Condition>)
    // Hint: In { value: Expr, list: Expr }
    // Hint: Contains { container: Expr, value: Expr }
    // Hint: IsNull(Expr)
    // Hint: IsType { value: Expr, type_name: String }
}

// TODO: Define Expression enum for value extraction
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // TODO: Add expression variants
    // Hint: Value(Value) - literal value
    // Hint: Variable(String) - variable lookup
    // Hint: Field { object: Box<Expr>, field: String } - nested field access
    // Hint: Index { list: Box<Expr>, index: Box<Expr> } - list/object indexing
}

impl Expr {
    // TODO: Evaluate expression in context
    pub fn evaluate(&self, ctx: &Context) -> Value {
        // Pseudocode:
        // match self:
        //     Expr::Value(v) => v.clone()
        //     Expr::Variable(name) => ctx.get(name).cloned().unwrap_or(Value::Null)
        //     Expr::Field { object, field } =>
        //         let obj_value = object.evaluate(ctx)
        //         match obj_value:
        //             Value::Object(map) => map.get(field).cloned().unwrap_or(Value::Null)
        //             _ => Value::Null
        //     Expr::Index { list, index } =>
        //         let list_value = list.evaluate(ctx)
        //         let index_value = index.evaluate(ctx)
        //         match (list_value, index_value):
        //             (Value::List(items), Value::Integer(i)) if i >= 0 && (i as usize) < items.len() =>
        //                 items[i as usize].clone()
        //             (Value::Object(map), Value::String(key)) =>
        //                 map.get(&key).cloned().unwrap_or(Value::Null)
        //             _ => Value::Null
        todo!()
    }
}

impl Condition {
    // TODO: Evaluate condition to boolean
    pub fn evaluate(&self, ctx: &Context) -> bool {
        // Pseudocode:
        // match self:
        //     Condition::Literal(b) => *b
        //     Condition::Compare { op, left, right } =>
        //         let left_val = left.evaluate(ctx)
        //         let right_val = right.evaluate(ctx)
        //         self.compare(op, &left_val, &right_val)
        //     Condition::Logical { op, conditions } =>
        //         match op:
        //             LogicalOp::And => conditions.iter().all(|c| c.evaluate(ctx))
        //             LogicalOp::Or => conditions.iter().any(|c| c.evaluate(ctx))
        //     Condition::Not(cond) => !cond.evaluate(ctx)
        //     Condition::In { value, list } =>
        //         let val = value.evaluate(ctx)
        //         let list_val = list.evaluate(ctx)
        //         match list_val:
        //             Value::List(items) => items.contains(&val)
        //             _ => false
        //     Condition::Contains { container, value } =>
        //         similar to In but reversed
        //     Condition::IsNull(expr) =>
        //         matches!(expr.evaluate(ctx), Value::Null)
        //     Condition::IsType { value, type_name } =>
        //         check type using pattern matching
        todo!()
    }

    // TODO: Implement comparison logic with pattern matching
    fn compare(&self, op: &CompareOp, left: &Value, right: &Value) -> bool {
        // Pseudocode:
        // Use exhaustive matching on (op, left, right) tuples
        // For numeric comparisons, convert to f64
        // For string comparisons, use string methods
        // For boolean/null, use direct equality
        // Handle type mismatches appropriately
        todo!()
    }
}
```

### Checkpoint Tests

Add to `tests/integration.rs`:

```rust
use business_rule_engine::context::{Context, Value};
use business_rule_engine::condition::{CompareOp, Condition, Expr, LogicalOp};

#[test]
fn test_value_type_checking() {
    assert!(Value::Integer(42).is_integer());
    assert!(Value::Float(3.14).is_float());
    assert!(Value::String("hello".to_string()).is_string());
    assert!(Value::Boolean(true).is_boolean());
    assert!(Value::Null.is_null());
}

#[test]
fn test_value_truthiness() {
    assert!(Value::Integer(1).is_truthy());
    assert!(!Value::Integer(0).is_truthy());
    assert!(Value::Boolean(true).is_truthy());
    assert!(!Value::Boolean(false).is_truthy());
    assert!(!Value::Null.is_truthy());
    assert!(!Value::String("".to_string()).is_truthy());
    assert!(Value::String("hello".to_string()).is_truthy());
}

#[test]
fn test_context_operations() {
    let mut ctx = Context::new();
    ctx.set("age", Value::Integer(25));
    ctx.set("name", Value::String("Alice".to_string()));

    assert_eq!(ctx.get("age"), Some(&Value::Integer(25)));
    assert_eq!(ctx.get("missing"), None);
}

#[test]
fn test_simple_comparison() {
    let ctx = Context::new();

    let cond = Condition::Compare {
        op: CompareOp::Equal,
        left: Expr::Value(Value::Integer(5)),
        right: Expr::Value(Value::Integer(5)),
    };

    assert!(cond.evaluate(&ctx));
}

#[test]
fn test_variable_comparison() {
    let mut ctx = Context::new();
    ctx.set("age", Value::Integer(25));

    let cond = Condition::Compare {
        op: CompareOp::GreaterThanOrEqual,
        left: Expr::Variable("age".to_string()),
        right: Expr::Value(Value::Integer(18)),
    };

    assert!(cond.evaluate(&ctx));
}

#[test]
fn test_logical_and() {
    let mut ctx = Context::new();
    ctx.set("age", Value::Integer(25));
    ctx.set("member", Value::Boolean(true));

    let cond = Condition::Logical {
        op: LogicalOp::And,
        conditions: vec![
            Condition::Compare {
                op: CompareOp::GreaterThanOrEqual,
                left: Expr::Variable("age".to_string()),
                right: Expr::Value(Value::Integer(18)),
            },
            Condition::Compare {
                op: CompareOp::Equal,
                left: Expr::Variable("member".to_string()),
                right: Expr::Value(Value::Boolean(true)),
            },
        ],
    };

    assert!(cond.evaluate(&ctx));
}

#[test]
fn test_is_null_condition() {
    let mut ctx = Context::new();
    ctx.set("value", Value::Null);

    let cond = Condition::IsNull(Expr::Variable("value".to_string()));
    assert!(cond.evaluate(&ctx));

    ctx.set("value", Value::Integer(5));
    assert!(!cond.evaluate(&ctx));
}

#[test]
fn test_in_condition() {
    let ctx = Context::new();

    let cond = Condition::In {
        value: Expr::Value(Value::String("admin".to_string())),
        list: Expr::Value(Value::List(vec![
            Value::String("admin".to_string()),
            Value::String("moderator".to_string()),
        ])),
    };

    assert!(cond.evaluate(&ctx));
}
```

### Check Your Understanding

1. Why is exhaustive pattern matching on `Value` important for the rule engine's correctness?
2. How does the `matches!` macro simplify type checking compared to a full `match` expression?
3. What are the advantages of using enums for operators instead of strings?
4. How does the let-else pattern help with early returns in value extraction?

---

## Milestone 2: Rule Definition and Action System

### Goal
Define rules with conditions and actions. Implement pattern matching for action execution and rule evaluation.

### Implementation Steps

#### Step 2.1: Define Actions

Create `src/action.rs`:

```rust
use crate::context::{Context, Value};

// TODO: Define Action enum for all action types
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    // TODO: Add action variants
    // Hint: SetVariable { name: String, value: Expr }
    // Hint: Log { message: String }
    // Hint: Return(Value)
    // Hint: Sequence(Vec<Action>) - execute multiple actions
    // Hint: Conditional { condition: Condition, then_action: Box<Action>, else_action: Option<Box<Action>> }
    // Hint: NoOp - do nothing
}

// TODO: Define ActionResult for action execution outcomes
#[derive(Debug, Clone, PartialEq)]
pub enum ActionResult {
    // TODO: Add result variants
    // Hint: Continue - continue to next rule
    // Hint: Return(Value) - return and stop
    // Hint: Modified(Context) - context was modified, continue
}

impl Action {
    // TODO: Execute action and return result
    pub fn execute(&self, ctx: &mut Context) -> ActionResult {
        // Pseudocode:
        // match self:
        //     Action::SetVariable { name, value } =>
        //         let val = value.evaluate(ctx)
        //         ctx.set(name.clone(), val)
        //         ActionResult::Continue
        //
        //     Action::Log { message } =>
        //         println!("{}", message)
        //         ActionResult::Continue
        //
        //     Action::Return(value) =>
        //         ActionResult::Return(value.clone())
        //
        //     Action::Sequence(actions) =>
        //         for action in actions:
        //             let result = action.execute(ctx)
        //             match result:
        //                 ActionResult::Return(v) => return ActionResult::Return(v)
        //                 _ => continue
        //         ActionResult::Continue
        //
        //     Action::Conditional { condition, then_action, else_action } =>
        //         if condition.evaluate(ctx):
        //             then_action.execute(ctx)
        //         else if let Some(else_act) = else_action:
        //             else_act.execute(ctx)
        //         else:
        //             ActionResult::Continue
        //
        //     Action::NoOp =>
        //         ActionResult::Continue
        todo!()
    }
}

use crate::condition::{Condition, Expr};
```

#### Step 2.2: Define Rules

Create `src/rule.rs`:

```rust
use crate::action::{Action, ActionResult};
use crate::condition::Condition;
use crate::context::Context;

// TODO: Define Rule priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    // TODO: Add priority levels
    // Hint: Low, Medium, High, Critical
}

impl Default for Priority {
    fn default() -> Self {
        // Pseudocode:
        // Priority::Medium
        todo!()
    }
}

// TODO: Define Rule structure
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    // TODO: Add fields
    // Hint: id: String
    // Hint: name: String
    // Hint: description: Option<String>
    // Hint: priority: Priority
    // Hint: condition: Condition
    // Hint: action: Action
    // Hint: enabled: bool
}

impl Rule {
    // TODO: Create new rule with builder pattern
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> RuleBuilder {
        // Pseudocode:
        // RuleBuilder {
        //     id: id.into(),
        //     name: name.into(),
        //     description: None,
        //     priority: Priority::default(),
        //     condition: Condition::Literal(true),
        //     action: Action::NoOp,
        //     enabled: true,
        // }
        todo!()
    }

    // TODO: Evaluate rule and execute action if condition matches
    pub fn evaluate(&self, ctx: &mut Context) -> RuleResult {
        // Pseudocode:
        // if !self.enabled:
        //     return RuleResult::Skipped
        //
        // if self.condition.evaluate(ctx):
        //     let result = self.action.execute(ctx)
        //     match result:
        //         ActionResult::Return(v) => RuleResult::Executed(Some(v))
        //         _ => RuleResult::Executed(None)
        // else:
        //     RuleResult::NotMatched
        todo!()
    }
}

// TODO: Define RuleResult for evaluation outcomes
#[derive(Debug, Clone, PartialEq)]
pub enum RuleResult {
    // TODO: Add result variants
    // Hint: Executed(Option<Value>) - condition matched, action executed
    // Hint: NotMatched - condition didn't match
    // Hint: Skipped - rule was disabled
}

// TODO: Define RuleBuilder for fluent API
#[derive(Debug)]
pub struct RuleBuilder {
    // TODO: Same fields as Rule
}

impl RuleBuilder {
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        // Pseudocode:
        // self.description = Some(desc.into());
        // self
        todo!()
    }

    pub fn priority(mut self, priority: Priority) -> Self {
        // Pseudocode:
        // self.priority = priority;
        // self
        todo!()
    }

    pub fn condition(mut self, condition: Condition) -> Self {
        // Pseudocode:
        // self.condition = condition;
        // self
        todo!()
    }

    pub fn action(mut self, action: Action) -> Self {
        // Pseudocode:
        // self.action = action;
        // self
        todo!()
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        // Pseudocode:
        // self.enabled = enabled;
        // self
        todo!()
    }

    pub fn build(self) -> Rule {
        // Pseudocode:
        // Rule {
        //     id: self.id,
        //     name: self.name,
        //     description: self.description,
        //     priority: self.priority,
        //     condition: self.condition,
        //     action: self.action,
        //     enabled: self.enabled,
        // }
        todo!()
    }
}

use crate::context::Value;
```

### Checkpoint Tests

Add to `tests/integration.rs`:

```rust
#[test]
fn test_set_variable_action() {
    use business_rule_engine::action::Action;
    use business_rule_engine::condition::Expr;

    let mut ctx = Context::new();
    ctx.set("x", Value::Integer(10));

    let action = Action::SetVariable {
        name: "y".to_string(),
        value: Expr::Variable("x".to_string()),
    };

    action.execute(&mut ctx);
    assert_eq!(ctx.get("y"), Some(&Value::Integer(10)));
}

#[test]
fn test_sequence_action() {
    use business_rule_engine::action::Action;
    use business_rule_engine::condition::Expr;

    let mut ctx = Context::new();

    let action = Action::Sequence(vec![
        Action::SetVariable {
            name: "a".to_string(),
            value: Expr::Value(Value::Integer(1)),
        },
        Action::SetVariable {
            name: "b".to_string(),
            value: Expr::Value(Value::Integer(2)),
        },
    ]);

    action.execute(&mut ctx);
    assert_eq!(ctx.get("a"), Some(&Value::Integer(1)));
    assert_eq!(ctx.get("b"), Some(&Value::Integer(2)));
}

#[test]
fn test_conditional_action() {
    use business_rule_engine::action::Action;
    use business_rule_engine::condition::{CompareOp, Condition, Expr};

    let mut ctx = Context::new();
    ctx.set("age", Value::Integer(25));

    let action = Action::Conditional {
        condition: Condition::Compare {
            op: CompareOp::GreaterThanOrEqual,
            left: Expr::Variable("age".to_string()),
            right: Expr::Value(Value::Integer(18)),
        },
        then_action: Box::new(Action::SetVariable {
            name: "status".to_string(),
            value: Expr::Value(Value::String("adult".to_string())),
        }),
        else_action: Some(Box::new(Action::SetVariable {
            name: "status".to_string(),
            value: Expr::Value(Value::String("minor".to_string())),
        })),
    };

    action.execute(&mut ctx);
    assert_eq!(ctx.get("status"), Some(&Value::String("adult".to_string())));
}

#[test]
fn test_rule_evaluation() {
    use business_rule_engine::action::Action;
    use business_rule_engine::condition::{CompareOp, Condition, Expr};
    use business_rule_engine::rule::{Priority, Rule, RuleResult};

    let mut ctx = Context::new();
    ctx.set("price", Value::Integer(100));

    let rule = Rule::new("discount_rule", "Apply Discount")
        .priority(Priority::High)
        .condition(Condition::Compare {
            op: CompareOp::GreaterThan,
            left: Expr::Variable("price".to_string()),
            right: Expr::Value(Value::Integer(50)),
        })
        .action(Action::SetVariable {
            name: "discount".to_string(),
            value: Expr::Value(Value::Float(0.1)),
        })
        .build();

    let result = rule.evaluate(&mut ctx);
    assert!(matches!(result, RuleResult::Executed(_)));
    assert_eq!(ctx.get("discount"), Some(&Value::Float(0.1)));
}

#[test]
fn test_disabled_rule() {
    use business_rule_engine::rule::{Rule, RuleResult};

    let mut ctx = Context::new();

    let rule = Rule::new("test", "Test")
        .enabled(false)
        .build();

    let result = rule.evaluate(&mut ctx);
    assert_eq!(result, RuleResult::Skipped);
}
```

### Check Your Understanding

1. How does the `ActionResult` enum enable control flow in rule execution?
2. Why is pattern matching on `ActionResult` more type-safe than using exceptions?
3. How does the builder pattern improve rule creation ergonomics?
4. What happens when a `Return` action is encountered in a `Sequence`?

---

## Milestone 3: Rule Engine with Priority and Execution Control

### Goal
Build the core rule engine that evaluates multiple rules in priority order, with support for different execution strategies.

### Implementation Steps

#### Step 3.1: Define Execution Strategies

Create `src/engine.rs`:

```rust
use crate::context::Context;
use crate::rule::{Rule, RuleResult};
use crate::context::Value;

// TODO: Define execution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStrategy {
    // TODO: Add strategy variants
    // Hint: FirstMatch - stop after first matching rule
    // Hint: AllMatch - execute all matching rules
    // Hint: HighestPriority - execute only highest priority matching rule
}

// TODO: Define RuleEngine structure
#[derive(Debug)]
pub struct RuleEngine {
    // TODO: Add fields
    // Hint: rules: Vec<Rule>
    // Hint: strategy: ExecutionStrategy
}

impl RuleEngine {
    pub fn new(strategy: ExecutionStrategy) -> Self {
        // Pseudocode:
        // Self { rules: Vec::new(), strategy }
        todo!()
    }

    pub fn add_rule(&mut self, rule: Rule) {
        // Pseudocode:
        // self.rules.push(rule);
        // Sort by priority (highest first)
        todo!()
    }

    pub fn add_rules(&mut self, rules: impl IntoIterator<Item = Rule>) {
        // Pseudocode:
        // for rule in rules:
        //     self.add_rule(rule)
        todo!()
    }

    // TODO: Execute rules based on strategy
    pub fn execute(&self, ctx: &mut Context) -> EngineResult {
        // Pseudocode:
        // match self.strategy:
        //     ExecutionStrategy::FirstMatch =>
        //         for rule in &self.rules:
        //             let result = rule.evaluate(ctx)
        //             match result:
        //                 RuleResult::Executed(Some(value)) =>
        //                     return EngineResult::Stopped(value)
        //                 RuleResult::Executed(None) =>
        //                     return EngineResult::Success
        //                 _ => continue
        //         EngineResult::NoMatch
        //
        //     ExecutionStrategy::AllMatch =>
        //         let mut executed_count = 0
        //         for rule in &self.rules:
        //             let result = rule.evaluate(ctx)
        //             match result:
        //                 RuleResult::Executed(Some(value)) =>
        //                     return EngineResult::Stopped(value)
        //                 RuleResult::Executed(None) =>
        //                     executed_count += 1
        //                 _ => continue
        //         if executed_count > 0:
        //             EngineResult::Success
        //         else:
        //             EngineResult::NoMatch
        //
        //     ExecutionStrategy::HighestPriority =>
        //         // Rules already sorted by priority
        //         for rule in &self.rules:
        //             let result = rule.evaluate(ctx)
        //             match result:
        //                 RuleResult::Executed(v) => return EngineResult from v
        //                 RuleResult::NotMatched => continue
        //                 RuleResult::Skipped => continue
        //         EngineResult::NoMatch
        todo!()
    }

    // TODO: Get rules by priority using pattern matching
    pub fn rules_by_priority(&self, priority: crate::rule::Priority) -> Vec<&Rule> {
        // Pseudocode:
        // self.rules.iter()
        //     .filter(|r| r.priority == priority)
        //     .collect()
        todo!()
    }

    // TODO: Get enabled rules using if-let or pattern matching
    pub fn enabled_rules(&self) -> Vec<&Rule> {
        // Pseudocode:
        // self.rules.iter()
        //     .filter(|r| r.enabled)
        //     .collect()
        todo!()
    }
}

// TODO: Define EngineResult for execution outcomes
#[derive(Debug, Clone, PartialEq)]
pub enum EngineResult {
    // TODO: Add result variants
    // Hint: Success - rules executed successfully
    // Hint: Stopped(Value) - execution stopped with return value
    // Hint: NoMatch - no rules matched
}
```

#### Step 3.2: Define lib.rs

Create `src/lib.rs`:

```rust
pub mod action;
pub mod condition;
pub mod context;
pub mod engine;
pub mod rule;

// Re-export commonly used types
pub use action::{Action, ActionResult};
pub use condition::{CompareOp, Condition, Expr, LogicalOp};
pub use context::{Context, Value};
pub use engine::{EngineResult, ExecutionStrategy, RuleEngine};
pub use rule::{Priority, Rule, RuleResult};
```

### Checkpoint Tests

Add to `tests/integration.rs`:

```rust
#[test]
fn test_first_match_strategy() {
    use business_rule_engine::*;

    let mut engine = RuleEngine::new(ExecutionStrategy::FirstMatch);

    engine.add_rule(
        Rule::new("rule1", "First Rule")
            .condition(Condition::Compare {
                op: CompareOp::GreaterThan,
                left: Expr::Variable("x".to_string()),
                right: Expr::Value(Value::Integer(5)),
            })
            .action(Action::SetVariable {
                name: "result".to_string(),
                value: Expr::Value(Value::String("first".to_string())),
            })
            .build(),
    );

    engine.add_rule(
        Rule::new("rule2", "Second Rule")
            .condition(Condition::Compare {
                op: CompareOp::GreaterThan,
                left: Expr::Variable("x".to_string()),
                right: Expr::Value(Value::Integer(3)),
            })
            .action(Action::SetVariable {
                name: "result".to_string(),
                value: Expr::Value(Value::String("second".to_string())),
            })
            .build(),
    );

    let mut ctx = Context::new();
    ctx.set("x", Value::Integer(10));

    let result = engine.execute(&mut ctx);
    assert_eq!(result, EngineResult::Success);
    assert_eq!(ctx.get("result"), Some(&Value::String("first".to_string())));
}

#[test]
fn test_all_match_strategy() {
    use business_rule_engine::*;

    let mut engine = RuleEngine::new(ExecutionStrategy::AllMatch);

    engine.add_rule(
        Rule::new("rule1", "Set A")
            .condition(Condition::Literal(true))
            .action(Action::SetVariable {
                name: "a".to_string(),
                value: Expr::Value(Value::Integer(1)),
            })
            .build(),
    );

    engine.add_rule(
        Rule::new("rule2", "Set B")
            .condition(Condition::Literal(true))
            .action(Action::SetVariable {
                name: "b".to_string(),
                value: Expr::Value(Value::Integer(2)),
            })
            .build(),
    );

    let mut ctx = Context::new();
    let result = engine.execute(&mut ctx);

    assert_eq!(result, EngineResult::Success);
    assert_eq!(ctx.get("a"), Some(&Value::Integer(1)));
    assert_eq!(ctx.get("b"), Some(&Value::Integer(2)));
}

#[test]
fn test_priority_ordering() {
    use business_rule_engine::*;

    let mut engine = RuleEngine::new(ExecutionStrategy::FirstMatch);

    // Add in reverse priority order
    engine.add_rule(
        Rule::new("low", "Low Priority")
            .priority(Priority::Low)
            .condition(Condition::Literal(true))
            .action(Action::SetVariable {
                name: "result".to_string(),
                value: Expr::Value(Value::String("low".to_string())),
            })
            .build(),
    );

    engine.add_rule(
        Rule::new("critical", "Critical Priority")
            .priority(Priority::Critical)
            .condition(Condition::Literal(true))
            .action(Action::SetVariable {
                name: "result".to_string(),
                value: Expr::Value(Value::String("critical".to_string())),
            })
            .build(),
    );

    let mut ctx = Context::new();
    engine.execute(&mut ctx);

    // Critical priority should execute first
    assert_eq!(ctx.get("result"), Some(&Value::String("critical".to_string())));
}
```

### Check Your Understanding

1. How does pattern matching on `ExecutionStrategy` enable different execution behaviors?
2. Why is it important to sort rules by priority in the engine?
3. How does the `FirstMatch` strategy differ from `HighestPriority` when multiple rules match?
4. What are the trade-offs between `AllMatch` and `FirstMatch` strategies?

---

## Milestone 4: Real-World Examples - Pricing and Approval Workflows

### Goal
Implement complete real-world examples demonstrating complex rule composition and nested pattern matching.

### Implementation Steps

#### Step 4.1: Dynamic Pricing Example

Create `src/examples/pricing.rs`:

```rust
use crate::*;
use std::collections::HashMap;

// TODO: Define pricing tier enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PricingTier {
    // TODO: Add tier variants
    // Hint: Basic, Premium, Enterprise
}

// TODO: Define customer segment enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomerSegment {
    // TODO: Add segment variants
    // Hint: New, Regular, VIP, Churned
}

// TODO: Create pricing rule engine
pub fn create_pricing_engine() -> RuleEngine {
    // Pseudocode:
    // Create engine with AllMatch strategy
    // Add rules for:
    // 1. Base discount by tier (Premium: 10%, Enterprise: 20%)
    // 2. Volume discounts (>100 items: +5%, >500 items: +10%)
    // 3. VIP customer bonus (additional 5%)
    // 4. New customer promotion (15% off first purchase)
    // 5. Seasonal promotions
    // 6. Bundle discounts
    // 7. Loyalty rewards
    //
    // Each rule should:
    // - Check specific conditions using pattern guards
    // - Calculate discount contribution
    // - Store in context for final calculation
    todo!()
}

// TODO: Calculate final price with all discounts
pub fn calculate_price(
    base_price: f64,
    quantity: i64,
    tier: PricingTier,
    segment: CustomerSegment,
    is_first_purchase: bool,
    is_seasonal_promo: bool,
) -> f64 {
    // Pseudocode:
    // Create context with all inputs
    // Execute pricing engine
    // Sum all discount percentages
    // Apply maximum cap (e.g., 50% max discount)
    // Calculate final price
    // Return final_price
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tier_discount() {
        let price = calculate_price(
            100.0,
            10,
            PricingTier::Premium,
            CustomerSegment::Regular,
            false,
            false,
        );
        assert_eq!(price, 90.0); // 10% premium discount
    }

    #[test]
    fn test_vip_customer_stacking() {
        let price = calculate_price(
            100.0,
            10,
            PricingTier::Premium,
            CustomerSegment::VIP,
            false,
            false,
        );
        assert_eq!(price, 85.0); // 10% premium + 5% VIP
    }

    #[test]
    fn test_volume_discount() {
        let price = calculate_price(
            100.0,
            150,
            PricingTier::Basic,
            CustomerSegment::Regular,
            false,
            false,
        );
        assert_eq!(price, 95.0); // 5% volume discount (>100 items)
    }

    #[test]
    fn test_new_customer_promo() {
        let price = calculate_price(
            100.0,
            10,
            PricingTier::Basic,
            CustomerSegment::New,
            true,
            false,
        );
        assert_eq!(price, 85.0); // 15% new customer discount
    }

    #[test]
    fn test_maximum_discount_cap() {
        // All discounts: Enterprise(20%) + VIP(5%) + Volume(10%) + Seasonal(15%) = 50%
        let price = calculate_price(
            100.0,
            600,
            PricingTier::Enterprise,
            CustomerSegment::VIP,
            false,
            true,
        );
        assert_eq!(price, 50.0); // Capped at 50% max
    }
}
```

#### Step 4.2: Approval Workflow Example

Create `src/examples/approval.rs`:

```rust
use crate::*;

// TODO: Define approval status enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalStatus {
    // TODO: Add status variants
    // Hint: Pending, Approved, Rejected, NeedsReview, Escalated
}

// TODO: Define approval level enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ApprovalLevel {
    // TODO: Add level variants
    // Hint: Manager, Director, VP, CEO
}

// TODO: Create approval workflow engine
pub fn create_approval_engine() -> RuleEngine {
    // Pseudocode:
    // Create engine with FirstMatch strategy (stop on first decision)
    // Add rules in priority order:
    //
    // 1. Auto-reject if amount > budget * 2 (Critical priority)
    // 2. Auto-approve if amount < 1000 and requester_level >= Manager
    // 3. Require Director approval if 1000 <= amount < 10000
    // 4. Require VP approval if 10000 <= amount < 100000
    // 5. Require CEO approval if amount >= 100000
    // 6. Auto-reject if department is over budget
    // 7. Escalate if high_risk flag is set
    // 8. Auto-approve if emergency and requester_level >= Director
    //
    // Use pattern guards for amount ranges
    // Use or-patterns for grouping conditions
    // Return appropriate ApprovalStatus
    todo!()
}

// TODO: Process approval request
pub fn process_approval(
    amount: f64,
    department_budget: f64,
    department_spent: f64,
    requester_level: ApprovalLevel,
    high_risk: bool,
    emergency: bool,
) -> ApprovalStatus {
    // Pseudocode:
    // Create context with all inputs
    // Execute approval engine
    // Extract result from context
    // Return ApprovalStatus
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_approve_small_amount() {
        let status = process_approval(
            500.0,
            100000.0,
            50000.0,
            ApprovalLevel::Manager,
            false,
            false,
        );
        assert_eq!(status, ApprovalStatus::Approved);
    }

    #[test]
    fn test_director_approval_required() {
        let status = process_approval(
            5000.0,
            100000.0,
            50000.0,
            ApprovalLevel::Manager,
            false,
            false,
        );
        assert_eq!(status, ApprovalStatus::NeedsReview);
        // In real system, would check required_level == Director
    }

    #[test]
    fn test_over_budget_rejection() {
        let status = process_approval(
            60000.0,
            100000.0,
            95000.0, // Only 5k left
            ApprovalLevel::Manager,
            false,
            false,
        );
        assert_eq!(status, ApprovalStatus::Rejected);
    }

    #[test]
    fn test_high_risk_escalation() {
        let status = process_approval(
            5000.0,
            100000.0,
            50000.0,
            ApprovalLevel::Manager,
            true, // high risk
            false,
        );
        assert_eq!(status, ApprovalStatus::Escalated);
    }

    #[test]
    fn test_emergency_override() {
        let status = process_approval(
            5000.0,
            100000.0,
            50000.0,
            ApprovalLevel::Director,
            false,
            true, // emergency
        );
        assert_eq!(status, ApprovalStatus::Approved);
    }

    #[test]
    fn test_ceo_approval_required() {
        let status = process_approval(
            150000.0,
            200000.0,
            50000.0,
            ApprovalLevel::VP,
            false,
            false,
        );
        assert_eq!(status, ApprovalStatus::NeedsReview);
        // Would check required_level == CEO
    }
}
```

### Checkpoint Tests

The tests are embedded in each example module above.

### Check Your Understanding

1. How do or-patterns simplify the pricing tier discount logic?
2. Why is `FirstMatch` strategy appropriate for approval workflows?
3. How would you add a new discount rule without modifying existing rules?
4. What pattern matching features make the approval level checks type-safe?

---

## Milestone 5: Advanced Features - Fraud Detection and Optimizations

### Goal
Implement a complex fraud detection example with nested conditions, state tracking, and performance optimizations using pattern matching.

### Implementation Steps

#### Step 5.1: Fraud Detection Example

Create `src/examples/fraud.rs`:

```rust
use crate::*;
use std::collections::HashMap;

// TODO: Define risk score enum with pattern matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskScore {
    // TODO: Add score levels
    // Hint: Low, Medium, High, Critical
}

impl RiskScore {
    // TODO: Convert numeric score to enum using range patterns
    pub fn from_score(score: u32) -> Self {
        // Pseudocode:
        // match score:
        //     0..=30 => RiskScore::Low
        //     31..=60 => RiskScore::Medium
        //     61..=85 => RiskScore::High
        //     _ => RiskScore::Critical
        todo!()
    }

    pub fn to_score(&self) -> u32 {
        // Pseudocode:
        // match self:
        //     RiskScore::Low => 15
        //     RiskScore::Medium => 45
        //     RiskScore::High => 75
        //     RiskScore::Critical => 95
        todo!()
    }
}

// TODO: Define transaction pattern enum
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionPattern {
    // TODO: Add pattern variants
    // Hint: Normal
    // Hint: HighFrequency { count: u32, window_minutes: u32 }
    // Hint: UnusualAmount { amount: f64, avg_amount: f64 }
    // Hint: NewLocation { country: String }
    // Hint: NewDevice { device_id: String }
    // Hint: SuspiciousMerchant { merchant_id: String }
}

// TODO: Create fraud detection engine
pub fn create_fraud_detection_engine() -> RuleEngine {
    // Pseudocode:
    // Create engine with AllMatch strategy (accumulate risk score)
    //
    // Add rules for pattern detection:
    // 1. High frequency transactions (Critical priority)
    //    - More than 5 transactions in 10 minutes: +30 points
    //    - More than 10 transactions in 30 minutes: +50 points
    //
    // 2. Unusual amount patterns (High priority)
    //    - Amount > 3x average: +25 points
    //    - Amount > 10x average: +40 points
    //    - Multiple small amounts (structuring): +35 points
    //
    // 3. Location anomalies (High priority)
    //    - New country: +15 points
    //    - High-risk country: +30 points
    //    - Impossible travel (two locations too far apart): +50 points
    //
    // 4. Device/account anomalies (Medium priority)
    //    - New device: +10 points
    //    - New device + new location: +25 points
    //    - Account age < 30 days: +15 points
    //
    // 5. Merchant patterns (High priority)
    //    - Suspicious merchant category: +20 points
    //    - Merchant flagged before: +35 points
    //
    // 6. Behavioral patterns (Medium priority)
    //    - Transaction outside normal hours: +10 points
    //    - Unusual merchant for user: +15 points
    //
    // Use pattern guards for threshold checks
    // Use or-patterns for grouping similar conditions
    // Accumulate scores in context
    todo!()
}

// TODO: Analyze transaction for fraud
pub fn analyze_transaction(
    amount: f64,
    avg_amount: f64,
    transaction_count_10min: u32,
    transaction_count_30min: u32,
    country: &str,
    is_new_country: bool,
    is_high_risk_country: bool,
    is_new_device: bool,
    account_age_days: u32,
    merchant_category: &str,
    is_suspicious_merchant: bool,
    is_unusual_hour: bool,
) -> (RiskScore, Vec<TransactionPattern>) {
    // Pseudocode:
    // Create context with all inputs
    // Execute fraud detection engine
    // Calculate total risk score
    // Convert to RiskScore enum
    // Extract detected patterns from context
    // Return (RiskScore, patterns)
    todo!()
}

// TODO: Define fraud action based on risk score
pub fn determine_action(risk_score: RiskScore) -> FraudAction {
    // Pseudocode:
    // match risk_score:
    //     RiskScore::Low => FraudAction::Allow
    //     RiskScore::Medium => FraudAction::Review
    //     RiskScore::High => FraudAction::Challenge
    //     RiskScore::Critical => FraudAction::Block
    todo!()
}

// TODO: Define fraud action enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FraudAction {
    // TODO: Add action variants
    // Hint: Allow - allow transaction
    // Hint: Review - flag for manual review
    // Hint: Challenge - require additional authentication
    // Hint: Block - block transaction
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_transaction() {
        let (risk, patterns) = analyze_transaction(
            50.0,
            45.0,
            1,
            2,
            "US",
            false,
            false,
            false,
            365,
            "retail",
            false,
            false,
        );
        assert_eq!(risk, RiskScore::Low);
        assert_eq!(determine_action(risk), FraudAction::Allow);
    }

    #[test]
    fn test_high_frequency_attack() {
        let (risk, patterns) = analyze_transaction(
            50.0,
            45.0,
            12, // 12 transactions in 10 min
            15,
            "US",
            false,
            false,
            false,
            365,
            "retail",
            false,
            false,
        );
        assert!(risk >= RiskScore::High);
        assert!(patterns.iter().any(|p| matches!(p, TransactionPattern::HighFrequency { .. })));
    }

    #[test]
    fn test_unusual_amount() {
        let (risk, patterns) = analyze_transaction(
            1000.0, // 20x average
            50.0,
            1,
            2,
            "US",
            false,
            false,
            false,
            365,
            "retail",
            false,
            false,
        );
        assert!(risk >= RiskScore::Medium);
        assert!(patterns.iter().any(|p| matches!(p, TransactionPattern::UnusualAmount { .. })));
    }

    #[test]
    fn test_new_country_and_device() {
        let (risk, patterns) = analyze_transaction(
            50.0,
            45.0,
            1,
            2,
            "CN",
            true, // new country
            false,
            true, // new device
            365,
            "retail",
            false,
            false,
        );
        assert!(risk >= RiskScore::Medium);
        assert_eq!(determine_action(risk), FraudAction::Challenge);
    }

    #[test]
    fn test_high_risk_merchant() {
        let (risk, patterns) = analyze_transaction(
            200.0,
            45.0,
            1,
            2,
            "US",
            false,
            false,
            false,
            365,
            "gambling",
            true, // suspicious merchant
            false,
        );
        assert!(risk >= RiskScore::High);
        assert_eq!(determine_action(risk), FraudAction::Challenge);
    }

    #[test]
    fn test_critical_risk_combination() {
        let (risk, patterns) = analyze_transaction(
            1000.0,     // 20x average
            50.0,
            8,          // high frequency
            12,
            "NG",       // new country
            true,
            true,       // high risk country
            true,       // new device
            10,         // new account
            "gambling",
            true,       // suspicious merchant
            true,       // unusual hour
        );
        assert_eq!(risk, RiskScore::Critical);
        assert_eq!(determine_action(risk), FraudAction::Block);
        assert!(patterns.len() >= 3); // Multiple patterns detected
    }
}
```

#### Step 5.2: Performance Optimizations

Add to `src/engine.rs`:

```rust
// TODO: Add caching for condition evaluation
impl Condition {
    // TODO: Implement short-circuit evaluation for logical operators
    pub fn evaluate_optimized(&self, ctx: &Context) -> bool {
        // Pseudocode:
        // match self:
        //     Condition::Logical { op: LogicalOp::And, conditions } =>
        //         // Short-circuit: return false on first false
        //         for cond in conditions:
        //             if !cond.evaluate_optimized(ctx):
        //                 return false
        //         return true
        //
        //     Condition::Logical { op: LogicalOp::Or, conditions } =>
        //         // Short-circuit: return true on first true
        //         for cond in conditions:
        //             if cond.evaluate_optimized(ctx):
        //                 return true
        //         return false
        //
        //     _ => self.evaluate(ctx)
        todo!()
    }
}

// TODO: Add rule filtering optimizations
impl RuleEngine {
    // TODO: Pre-filter rules by enabled status
    pub fn execute_optimized(&self, ctx: &mut Context) -> EngineResult {
        // Pseudocode:
        // Filter to only enabled rules first
        // Group by priority for early exit optimizations
        // Use short-circuit evaluation in conditions
        // Same logic as execute() but with optimizations
        todo!()
    }
}
```

### Benchmark

Create `benches/evaluation.rs`:

```rust
use business_rule_engine::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn create_complex_ruleset() -> RuleEngine {
    let mut engine = RuleEngine::new(ExecutionStrategy::AllMatch);

    // Add 100 rules with various complexities
    for i in 0..100 {
        engine.add_rule(
            Rule::new(format!("rule_{}", i), format!("Rule {}", i))
                .condition(Condition::Logical {
                    op: LogicalOp::And,
                    conditions: vec![
                        Condition::Compare {
                            op: CompareOp::GreaterThan,
                            left: Expr::Variable("x".to_string()),
                            right: Expr::Value(Value::Integer(i as i64)),
                        },
                        Condition::Compare {
                            op: CompareOp::LessThan,
                            left: Expr::Variable("y".to_string()),
                            right: Expr::Value(Value::Integer((i + 50) as i64)),
                        },
                    ],
                })
                .action(Action::SetVariable {
                    name: format!("result_{}", i),
                    value: Expr::Value(Value::Integer(i as i64)),
                })
                .build(),
        );
    }

    engine
}

fn benchmark_rule_evaluation(c: &mut Criterion) {
    let engine = create_complex_ruleset();

    c.bench_function("evaluate_100_rules", |b| {
        b.iter(|| {
            let mut ctx = Context::new();
            ctx.set("x", Value::Integer(black_box(50)));
            ctx.set("y", Value::Integer(black_box(75)));
            engine.execute(&mut ctx)
        });
    });
}

fn benchmark_condition_evaluation(c: &mut Criterion) {
    let condition = Condition::Logical {
        op: LogicalOp::And,
        conditions: vec![
            Condition::Compare {
                op: CompareOp::GreaterThan,
                left: Expr::Variable("x".to_string()),
                right: Expr::Value(Value::Integer(10)),
            },
            Condition::Compare {
                op: CompareOp::LessThan,
                left: Expr::Variable("y".to_string()),
                right: Expr::Value(Value::Integer(100)),
            },
            Condition::In {
                value: Expr::Variable("category".to_string()),
                list: Expr::Value(Value::List(vec![
                    Value::String("A".to_string()),
                    Value::String("B".to_string()),
                    Value::String("C".to_string()),
                ])),
            },
        ],
    };

    c.bench_function("evaluate_complex_condition", |b| {
        b.iter(|| {
            let mut ctx = Context::new();
            ctx.set("x", Value::Integer(black_box(50)));
            ctx.set("y", Value::Integer(black_box(75)));
            ctx.set("category", Value::String("B".to_string()));
            condition.evaluate(&ctx)
        });
    });
}

criterion_group!(benches, benchmark_rule_evaluation, benchmark_condition_evaluation);
criterion_main!(benches);
```

Update `Cargo.toml`:

```toml
[package]
name = "business-rule-engine"
version = "0.1.0"
edition = "2021"

[dependencies]

[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "evaluation"
harness = false
```

### Checkpoint Tests

The tests are embedded in `src/examples/fraud.rs` above.

### Check Your Understanding

1. How do range patterns simplify risk score classification?
2. Why is exhaustive matching important for fraud detection decisions?
3. How does pattern matching on `TransactionPattern` enable extensibility?
4. What are the performance benefits of short-circuit evaluation in logical conditions?
5. How would you add a new fraud pattern without modifying existing code?

---

## Extensions and Challenges

### Extension 1: Rule Serialization
- Implement `Serialize` and `Deserialize` for all types
- Load rules from JSON/YAML files
- Support rule hot-reloading

### Extension 2: Rule Debugging
- Add rule execution tracing
- Show which conditions matched/failed
- Calculate coverage metrics

### Extension 3: Advanced Patterns
- Add pattern matching on custom types
- Implement wildcard/regex matching in conditions
- Support nested object path queries (e.g., "user.address.city")

### Extension 4: Temporal Rules
- Add time-based conditions (weekday, hour, date ranges)
- Implement rule expiration
- Support scheduled rule activation

### Extension 5: Rule Conflict Detection
- Detect contradictory rules
- Find redundant conditions
- Suggest rule optimizations

### Extension 6: Performance Monitoring
- Add metrics for rule execution time
- Identify slow rules
- Optimize rule ordering based on execution patterns

### Extension 7: Visual Rule Builder
- Create a DSL for rule definition
- Build a parser for natural language rules
- Generate rules from decision tables

---

## Summary

In this project, you built a comprehensive business rule engine demonstrating:

1. **Exhaustive Enum Matching**: Complete coverage of all value types, conditions, and actions
2. **Deep Destructuring**: Nested pattern matching for complex expressions
3. **Pattern Guards**: Business logic encoded in match arm conditions
4. **Or-Patterns**: Grouping similar cases for cleaner code
5. **If-Let Chains**: Complex conditional logic with pattern matching
6. **matches! Macro**: Quick type checking without full match expressions
7. **Let-Else Patterns**: Early returns with pattern matching
8. **Enum-Driven Architecture**: Type-safe state machines and workflows

**Key Takeaways:**
- Enums and pattern matching enable compile-time correctness
- Exhaustive matching prevents runtime errors
- Pattern guards express complex business rules clearly
- Enum-driven design scales better than boolean flags or magic strings
- Pattern matching is both safer and more performant than runtime type checking

This architecture is production-ready and used in real-world systems for pricing, fraud detection, workflow automation, and policy enforcement.
