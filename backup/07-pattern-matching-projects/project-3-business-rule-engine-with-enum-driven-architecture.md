## Project 3: Business Rule Engine with Enum-Driven Architecture

### Problem Statement

Build a business rule engine that:
- Represents business rules as enums and pattern matching
- Evaluates complex conditional logic using exhaustive matching
- Supports rule composition and chaining
- Implements rule priorities and conflict resolution
- Provides rule validation and analysis
- Demonstrates all pattern matching features (guards, ranges, destructuring, if-let chains)
- Handles temporal rules (time-based, date-based conditions)
- Supports dynamic rule loading and hot-reload

The engine must showcase enum-driven design where business logic is encoded in types and pattern matching ensures correctness.

### Why It Matters

Business rule engines are critical for:
- **E-commerce**: Pricing, discounts, promotions, shipping rules
- **Finance**: Credit scoring, fraud detection, compliance
- **Insurance**: Policy underwriting, claims processing
- **Healthcare**: Treatment protocols, billing rules
- **Workflow**: Approval processes, routing, escalation

Pattern matching excels for rule engines because:
- Rules map naturally to enum variants
- Exhaustiveness ensures all cases handled
- Guards enable complex conditions
- Refactoring rules is safer (compiler catches missing cases)
- Rule composition through nested enums

### Use Cases

1. **E-commerce Promotions**: "Buy 2 get 1 free", "10% off orders over $100"
2. **Credit Approval**: Multi-factor decision trees for loan approval
3. **Insurance Pricing**: Risk-based premium calculation
4. **Fraud Detection**: Score transactions based on patterns
5. **Workflow Routing**: Route tasks based on properties
6. **Tax Calculation**: Complex tax rules with jurisdictions
7. **Access Control**: Role-based permission systems

### Solution Outline

**Core Rule Types:**
```rust
#[derive(Debug, Clone)]
pub enum Rule {
    // Simple conditions
    Equals { field: String, value: Value },
    GreaterThan { field: String, value: Value },
    LessThan { field: String, value: Value },
    In { field: String, values: Vec<Value> },
    Between { field: String, min: Value, max: Value },

    // String matching
    StartsWith { field: String, prefix: String },
    EndsWith { field: String, suffix: String },
    Contains { field: String, substring: String },
    Matches { field: String, pattern: String },

    // Logical operations
    And(Vec<Rule>),
    Or(Vec<Rule>),
    Not(Box<Rule>),

    // Temporal rules
    TimeRange { start: Time, end: Time },
    DateRange { start: Date, end: Date },
    DayOfWeek(Vec<DayOfWeek>),

    // Complex rules
    Custom { name: String, evaluator: fn(&Context) -> bool },
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(Vec<Value>),
}

#[derive(Debug, Clone)]
pub struct RuleSet {
    pub name: String,
    pub rules: Vec<(Rule, Action)>,
    pub default_action: Action,
}

#[derive(Debug, Clone)]
pub enum Action {
    ApplyDiscount(f64),
    SetPrice(f64),
    Approve,
    Reject { reason: String },
    Flag { severity: Severity },
    Route { destination: String },
    Multiple(Vec<Action>),
}
```

**Pattern Matching for Evaluation:**
```rust
impl Rule {
    pub fn evaluate(&self, context: &Context) -> bool {
        match self {
            // Simple comparisons with pattern guards
            Rule::Equals { field, value } => {
                match (context.get(field), value) {
                    (Some(ctx_val), expected) if ctx_val == expected => true,
                    _ => false,
                }
            }

            // Range patterns
            Rule::Between { field, min, max } => {
                match context.get(field) {
                    Some(Value::Int(n)) if matches!(min, Value::Int(min_val))
                        && matches!(max, Value::Int(max_val))
                        && (*min_val..=*max_val).contains(n) => true,
                    _ => false,
                }
            }

            // Logical operations with exhaustive matching
            Rule::And(rules) => rules.iter().all(|r| r.evaluate(context)),
            Rule::Or(rules) => rules.iter().any(|r| r.evaluate(context)),
            Rule::Not(rule) => !rule.evaluate(context),

            // ... other cases
        }
    }
}
```

**Testing Hints:**
```rust
#[test]
fn test_discount_rule() {
    let rule = Rule::And(vec![
        Rule::GreaterThan {
            field: "total".into(),
            value: Value::Float(100.0)
        },
        Rule::Equals {
            field: "customer_tier".into(),
            value: Value::String("gold".into())
        },
    ]);

    let mut context = Context::new();
    context.set("total", Value::Float(150.0));
    context.set("customer_tier", Value::String("gold".into()));

    assert!(rule.evaluate(&context));
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Basic Rule Types and Simple Evaluation

**Goal:** Implement fundamental rule types with basic pattern matching.

**What to implement:**
```rust
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

impl Value {
    fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Rule {
    Equals { field: String, value: Value },
    GreaterThan { field: String, value: Value },
    LessThan { field: String, value: Value },
    And(Vec<Rule>),
    Or(Vec<Rule>),
}

#[derive(Debug)]
pub struct Context {
    values: HashMap<String, Value>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            values: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.values.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }
}

impl Rule {
    pub fn evaluate(&self, context: &Context) -> bool {
        match self {
            Rule::Equals { field, value } => {
                match context.get(field) {
                    Some(ctx_value) if ctx_value == value => true,
                    _ => false,
                }
            }

            Rule::GreaterThan { field, value } => {
                match (context.get(field), value) {
                    // Integer comparison
                    (Some(Value::Int(ctx)), Value::Int(expected)) => ctx > expected,

                    // Float comparison
                    (Some(ctx_val), expected_val) => {
                        match (ctx_val.as_float(), expected_val.as_float()) {
                            (Some(ctx_f), Some(exp_f)) => ctx_f > exp_f,
                            _ => false,
                        }
                    }
                }
            }

            Rule::LessThan { field, value } => {
                match (context.get(field), value) {
                    (Some(Value::Int(ctx)), Value::Int(expected)) => ctx < expected,

                    (Some(ctx_val), expected_val) => {
                        match (ctx_val.as_float(), expected_val.as_float()) {
                            (Some(ctx_f), Some(exp_f)) => ctx_f < exp_f,
                            _ => false,
                        }
                    }
                }
            }

            Rule::And(rules) => {
                rules.iter().all(|rule| rule.evaluate(context))
            }

            Rule::Or(rules) => {
                rules.iter().any(|rule| rule.evaluate(context))
            }
        }
    }
}

// Simple action system
#[derive(Debug, Clone)]
pub enum Action {
    Approve,
    Reject,
    ApplyDiscount(f64),
}

#[derive(Debug)]
pub struct RuleEngine {
    rules: Vec<(Rule, Action)>,
    default_action: Action,
}

impl RuleEngine {
    pub fn new(default_action: Action) -> Self {
        RuleEngine {
            rules: Vec::new(),
            default_action,
        }
    }

    pub fn add_rule(&mut self, rule: Rule, action: Action) {
        self.rules.push((rule, action));
    }

    pub fn evaluate(&self, context: &Context) -> Action {
        for (rule, action) in &self.rules {
            if rule.evaluate(context) {
                return action.clone();
            }
        }

        self.default_action.clone()
    }
}
```

**Check/Test:**
- Test simple equality checks
- Test numeric comparisons (greater than, less than)
- Test AND/OR logic
- Test rule evaluation order
- Test missing fields return false

**Why this isn't enough:**
Only supports basic comparisons. Real business rules need range checks, string matching, temporal logic, priorities, and complex nested conditions. The pattern matching is straightforward—we're not showcasing guards, range patterns, or deep destructuring. We need more rule types and sophisticated matching.

---

### Step 2: Add Range Patterns, String Matching, and Negation

**Goal:** Expand rule types and demonstrate range patterns and guards.

**What to improve:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(Vec<Value>),
}

#[derive(Debug, Clone)]
pub enum Rule {
    Equals { field: String, value: Value },
    GreaterThan { field: String, value: Value },
    LessThan { field: String, value: Value },
    Between { field: String, min: Value, max: Value },
    In { field: String, values: Vec<Value> },

    // String matching
    StartsWith { field: String, prefix: String },
    EndsWith { field: String, suffix: String },
    Contains { field: String, substring: String },

    // Logical
    And(Vec<Rule>),
    Or(Vec<Rule>),
    Not(Box<Rule>),
}

impl Rule {
    pub fn evaluate(&self, context: &Context) -> bool {
        match self {
            // Existing cases...

            // Range patterns with guards
            Rule::Between { field, min, max } => {
                let Some(value) = context.get(field) else {
                    return false;
                };

                match (value, min, max) {
                    // Integer ranges
                    (Value::Int(n), Value::Int(min_val), Value::Int(max_val))
                        if (*min_val..=*max_val).contains(n) => true,

                    // Float ranges
                    (Value::Float(n), Value::Float(min_val), Value::Float(max_val))
                        if n >= min_val && n <= max_val => true,

                    // Mixed numeric types
                    (val, min_val, max_val) => {
                        match (val.as_float(), min_val.as_float(), max_val.as_float()) {
                            (Some(n), Some(min_f), Some(max_f)) if n >= min_f && n <= max_f => true,
                            _ => false,
                        }
                    }
                }
            }

            // In-list checking with or-patterns
            Rule::In { field, values } => {
                match context.get(field) {
                    Some(ctx_value) => values.contains(ctx_value),
                    None => false,
                }
            }

            // String matching patterns
            Rule::StartsWith { field, prefix } => {
                match context.get(field) {
                    Some(Value::String(s)) if s.starts_with(prefix) => true,
                    _ => false,
                }
            }

            Rule::EndsWith { field, suffix } => {
                match context.get(field) {
                    Some(Value::String(s)) if s.ends_with(suffix) => true,
                    _ => false,
                }
            }

            Rule::Contains { field, substring } => {
                match context.get(field) {
                    Some(Value::String(s)) if s.contains(substring) => true,
                    _ => false,
                }
            }

            // Negation
            Rule::Not(rule) => !rule.evaluate(context),

            // Logical operations (existing)
            Rule::And(rules) => rules.iter().all(|r| r.evaluate(context)),
            Rule::Or(rules) => rules.iter().any(|r| r.evaluate(context)),

            _ => false,
        }
    }
}

// Rule builder for convenience
impl Rule {
    pub fn field_equals(field: &str, value: Value) -> Self {
        Rule::Equals {
            field: field.into(),
            value,
        }
    }

    pub fn field_between(field: &str, min: Value, max: Value) -> Self {
        Rule::Between {
            field: field.into(),
            min,
            max,
        }
    }

    pub fn field_in(field: &str, values: Vec<Value>) -> Self {
        Rule::In {
            field: field.into(),
            values,
        }
    }

    pub fn and(rules: Vec<Rule>) -> Self {
        Rule::And(rules)
    }

    pub fn or(rules: Vec<Rule>) -> Self {
        Rule::Or(rules)
    }

    pub fn not(rule: Rule) -> Self {
        Rule::Not(Box::new(rule))
    }
}
```

**Pattern matching for rule analysis:**
```rust
// Analyze rule complexity
pub fn count_conditions(rule: &Rule) -> usize {
    match rule {
        // Leaf conditions
        Rule::Equals { .. }
        | Rule::GreaterThan { .. }
        | Rule::LessThan { .. }
        | Rule::Between { .. }
        | Rule::In { .. }
        | Rule::StartsWith { .. }
        | Rule::EndsWith { .. }
        | Rule::Contains { .. } => 1,

        // Compound conditions
        Rule::And(rules) | Rule::Or(rules) => {
            rules.iter().map(count_conditions).sum()
        }

        Rule::Not(rule) => count_conditions(rule),
    }
}

// Extract field dependencies
pub fn get_required_fields(rule: &Rule) -> Vec<String> {
    match rule {
        Rule::Equals { field, .. }
        | Rule::GreaterThan { field, .. }
        | Rule::LessThan { field, .. }
        | Rule::Between { field, .. }
        | Rule::In { field, .. }
        | Rule::StartsWith { field, .. }
        | Rule::EndsWith { field, .. }
        | Rule::Contains { field, .. } => vec![field.clone()],

        Rule::And(rules) | Rule::Or(rules) => {
            rules.iter()
                .flat_map(get_required_fields)
                .collect()
        }

        Rule::Not(rule) => get_required_fields(rule),
    }
}
```

**Check/Test:**
- Test range checking with Between
- Test In with multiple values
- Test string prefix/suffix/contains
- Test NOT negation
- Test nested AND/OR combinations
- Test rule analysis functions

**Why this isn't enough:**
Rules work but no priority system. What if multiple rules match? Real engines need priority, conflict resolution, and rule ordering. We also don't have temporal rules (date/time-based), custom predicates, or rule validation. The actions are too simple—no parameterized actions or action chaining.

---

### Step 3: Add Rule Priorities, Actions, and Conflict Resolution

**Goal:** Implement rule priorities and rich action types with pattern matching.

**What to improve:**
```rust
#[derive(Debug, Clone)]
pub enum Action {
    Approve,
    Reject { reason: String },
    ApplyDiscount { percent: f64 },
    SetPrice { amount: f64 },
    AddBonus { points: i64 },
    Flag { severity: Severity, message: String },
    Route { destination: String },
    Log { level: LogLevel, message: String },
    Multiple(Vec<Action>),
    Conditional { condition: Rule, then_action: Box<Action>, else_action: Box<Action> },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct RuleWithPriority {
    pub rule: Rule,
    pub action: Action,
    pub priority: i32,  // Higher = more important
    pub name: String,
}

#[derive(Debug)]
pub struct RuleEngine {
    rules: Vec<RuleWithPriority>,
    default_action: Action,
    conflict_strategy: ConflictStrategy,
}

#[derive(Debug, Clone)]
pub enum ConflictStrategy {
    FirstMatch,      // Return first matching rule
    HighestPriority, // Return highest priority match
    AllMatches,      // Execute all matching rules
    MostSpecific,    // Return most specific (most conditions)
}

impl RuleEngine {
    pub fn new(default_action: Action, strategy: ConflictStrategy) -> Self {
        RuleEngine {
            rules: Vec::new(),
            default_action,
            conflict_strategy: strategy,
        }
    }

    pub fn add_rule(&mut self, rule: Rule, action: Action, priority: i32, name: String) {
        self.rules.push(RuleWithPriority {
            rule,
            action,
            priority,
            name,
        });

        // Sort by priority (descending)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn evaluate(&self, context: &Context) -> EvaluationResult {
        let matches: Vec<&RuleWithPriority> = self.rules
            .iter()
            .filter(|r| r.rule.evaluate(context))
            .collect();

        if matches.is_empty() {
            return EvaluationResult {
                action: self.default_action.clone(),
                matched_rules: vec![],
            };
        }

        // Apply conflict resolution strategy
        let selected_rules = match self.conflict_strategy {
            ConflictStrategy::FirstMatch => {
                vec![matches[0]]
            }

            ConflictStrategy::HighestPriority => {
                vec![matches[0]]  // Already sorted by priority
            }

            ConflictStrategy::AllMatches => {
                matches
            }

            ConflictStrategy::MostSpecific => {
                // Find rule with most conditions
                let max_conditions = matches
                    .iter()
                    .map(|r| count_conditions(&r.rule))
                    .max()
                    .unwrap_or(0);

                matches
                    .into_iter()
                    .filter(|r| count_conditions(&r.rule) == max_conditions)
                    .take(1)
                    .collect()
            }
        };

        // Combine actions
        let action = self.combine_actions(
            selected_rules.iter().map(|r| &r.action).collect()
        );

        EvaluationResult {
            action,
            matched_rules: selected_rules.iter().map(|r| r.name.clone()).collect(),
        }
    }

    fn combine_actions(&self, actions: Vec<&Action>) -> Action {
        if actions.is_empty() {
            return self.default_action.clone();
        }

        if actions.len() == 1 {
            return actions[0].clone();
        }

        Action::Multiple(actions.iter().map(|a| (*a).clone()).collect())
    }
}

#[derive(Debug)]
pub struct EvaluationResult {
    pub action: Action,
    pub matched_rules: Vec<String>,
}

// Execute actions with pattern matching
pub fn execute_action(action: &Action, context: &mut Context) -> ActionResult {
    match action {
        Action::Approve => {
            context.set("status", Value::String("approved".into()));
            ActionResult::Success
        }

        Action::Reject { reason } => {
            context.set("status", Value::String("rejected".into()));
            context.set("reason", Value::String(reason.clone()));
            ActionResult::Success
        }

        Action::ApplyDiscount { percent } if *percent > 0.0 && *percent <= 100.0 => {
            if let Some(Value::Float(price)) = context.get("price") {
                let discounted = price * (1.0 - percent / 100.0);
                context.set("final_price", Value::Float(discounted));
                ActionResult::Success
            } else {
                ActionResult::Error("Price field not found".into())
            }
        }

        Action::SetPrice { amount } if *amount >= 0.0 => {
            context.set("final_price", Value::Float(*amount));
            ActionResult::Success
        }

        Action::Flag { severity, message } => {
            context.set("flagged", Value::Bool(true));
            context.set("flag_severity", Value::String(format!("{:?}", severity)));
            context.set("flag_message", Value::String(message.clone()));
            ActionResult::Success
        }

        Action::Multiple(actions) => {
            for action in actions {
                if let ActionResult::Error(e) = execute_action(action, context) {
                    return ActionResult::Error(e);
                }
            }
            ActionResult::Success
        }

        Action::Conditional { condition, then_action, else_action } => {
            if condition.evaluate(context) {
                execute_action(then_action, context)
            } else {
                execute_action(else_action, context)
            }
        }

        _ => ActionResult::Error("Invalid action parameters".into()),
    }
}

#[derive(Debug)]
pub enum ActionResult {
    Success,
    Error(String),
}
```

**Pattern matching for action analysis:**
```rust
// Classify actions by type
pub fn classify_action(action: &Action) -> ActionType {
    match action {
        Action::Approve | Action::Reject { .. } => ActionType::Decision,

        Action::ApplyDiscount { .. } | Action::SetPrice { .. } => ActionType::Pricing,

        Action::Flag { severity, .. } => match severity {
            Severity::Critical | Severity::High => ActionType::Alert,
            _ => ActionType::Warning,
        },

        Action::Route { .. } => ActionType::Routing,

        Action::Log { level, .. } => match level {
            LogLevel::Error => ActionType::Alert,
            _ => ActionType::Informational,
        },

        Action::Multiple(actions) => {
            // Classify by highest severity action
            actions
                .iter()
                .map(classify_action)
                .max_by_key(|t| action_type_priority(t))
                .unwrap_or(ActionType::Other)
        }

        _ => ActionType::Other,
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ActionType {
    Decision,
    Pricing,
    Alert,
    Warning,
    Routing,
    Informational,
    Other,
}

fn action_type_priority(action_type: &ActionType) -> u8 {
    match action_type {
        ActionType::Decision => 5,
        ActionType::Alert => 4,
        ActionType::Pricing => 3,
        ActionType::Warning => 2,
        ActionType::Routing => 2,
        ActionType::Informational => 1,
        ActionType::Other => 0,
    }
}
```

**Check/Test:**
- Test priority ordering
- Test conflict resolution strategies
- Test action execution
- Test multiple action combination
- Test conditional actions
- Test action classification

**Why this isn't enough:**
Rules and actions work but no temporal logic. Business rules often depend on time/date (weekends, holidays, time ranges). We also don't validate rules before execution or provide debugging/explanation. Real engines need rule validation, temporal support, and explainability.

---
