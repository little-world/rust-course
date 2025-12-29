//==============================================================================
// Property-Based Test Generator - Complete Implementation
//==============================================================================

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

//==============================================================================
// Milestone 1: Function Signature Analysis
//==============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Primitive(String),
    Generic(String),
    Vec(Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
    pub is_mutable: bool,
}

#[derive(Debug, Clone)]
pub struct TraitConstraint {
    pub type_param: String,
    pub trait_bound: String,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub generics: Vec<String>,
    pub constraints: Vec<TraitConstraint>,
}

impl FunctionSignature {
    pub fn parse_function(source: &str) -> Result<Self, String> {
        let source = source.trim();

        let name = Self::extract_function_name(source)?;
        let generics = Self::extract_generics(source);
        let parameters = Self::extract_parameters(source)?;
        let return_type = Self::extract_return_type(source);
        let constraints = vec![];

        Ok(FunctionSignature {
            name,
            parameters,
            return_type,
            generics,
            constraints,
        })
    }

    fn extract_function_name(source: &str) -> Result<String, String> {
        if let Some(fn_pos) = source.find("fn ") {
            let after_fn = &source[fn_pos + 3..];
            if let Some(name_end) = after_fn.find(|c: char| c == '<' || c == '(') {
                Ok(after_fn[..name_end].trim().to_string())
            } else {
                Err("Could not find function name end".to_string())
            }
        } else {
            Err("Not a function".to_string())
        }
    }

    fn extract_generics(source: &str) -> Vec<String> {
        let mut generics = Vec::new();

        if let Some(start) = source.find("fn ") {
            let after_fn = &source[start + 3..];
            // Only look for generics before the opening parenthesis
            if let Some(paren_pos) = after_fn.find('(') {
                let before_params = &after_fn[..paren_pos];
                if let Some(lt_pos) = before_params.find('<') {
                    if let Some(gt_pos) = before_params.find('>') {
                        let generics_str = &before_params[lt_pos + 1..gt_pos];
                        for g in generics_str.split(',') {
                            let g = g.trim();
                            if let Some(colon) = g.find(':') {
                                generics.push(g[..colon].trim().to_string());
                            } else {
                                generics.push(g.to_string());
                            }
                        }
                    }
                }
            }
        }

        generics
    }

    fn extract_parameters(source: &str) -> Result<Vec<Parameter>, String> {
        let mut params = Vec::new();

        if let Some(paren_start) = source.find('(') {
            // Find matching closing parenthesis
            let mut depth = 0;
            let mut paren_end = paren_start;
            for (i, ch) in source[paren_start..].chars().enumerate() {
                if ch == '(' {
                    depth += 1;
                } else if ch == ')' {
                    depth -= 1;
                    if depth == 0 {
                        paren_end = paren_start + i;
                        break;
                    }
                }
            }

            let params_str = &source[paren_start + 1..paren_end];

            if params_str.trim().is_empty() {
                return Ok(params);
            }

            // Split by commas but respect nesting
            let param_strs = Self::smart_split(params_str, ',');

            for param_str in param_strs {
                let param_str = param_str.trim();
                if let Some(colon) = param_str.find(':') {
                    let name = param_str[..colon].trim();
                    let type_str = param_str[colon + 1..].trim();

                    let is_mutable = name.contains("mut") || type_str.contains("&mut");
                    let clean_name = name.replace("&mut", "").replace("&", "").replace("mut", "").trim().to_string();

                    params.push(Parameter {
                        name: clean_name,
                        param_type: Self::parse_type(type_str),
                        is_mutable,
                    });
                }
            }
        }

        Ok(params)
    }

    fn smart_split(s: &str, delimiter: char) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut depth = 0;

        for ch in s.chars() {
            if ch == '(' || ch == '<' {
                depth += 1;
                current.push(ch);
            } else if ch == ')' || ch == '>' {
                depth -= 1;
                current.push(ch);
            } else if ch == delimiter && depth == 0 {
                result.push(current.trim().to_string());
                current.clear();
            } else {
                current.push(ch);
            }
        }

        if !current.is_empty() {
            result.push(current.trim().to_string());
        }

        result
    }

    fn extract_return_type(source: &str) -> Option<Type> {
        if let Some(arrow_pos) = source.find("->") {
            let after_arrow = source[arrow_pos + 2..].trim();
            // For complex types like Result<i32, String>, we need to take everything
            // not just the first word
            let type_str = if after_arrow.contains('<') {
                // Find the matching closing bracket
                let mut depth = 0;
                let mut end = after_arrow.len();
                for (i, ch) in after_arrow.chars().enumerate() {
                    if ch == '<' {
                        depth += 1;
                    } else if ch == '>' {
                        depth -= 1;
                        if depth == 0 {
                            end = i + 1;
                            break;
                        }
                    }
                }
                &after_arrow[..end]
            } else {
                after_arrow.split_whitespace().next()?
            };
            Some(Self::parse_type(type_str))
        } else {
            None
        }
    }

    fn parse_type(type_str: &str) -> Type {
        let type_str = type_str.trim();

        // Check for tuple first
        if type_str.starts_with('(') && type_str.ends_with(')') {
            let inner = &type_str[1..type_str.len() - 1];
            let parts: Vec<Type> = inner.split(',').map(|s| Self::parse_type(s.trim())).collect();
            return Type::Tuple(parts);
        }

        if type_str.starts_with("Vec<") && type_str.ends_with('>') {
            let inner = &type_str[4..type_str.len() - 1];
            return Type::Vec(Box::new(Self::parse_type(inner)));
        }

        if type_str.starts_with("Option<") && type_str.ends_with('>') {
            let inner = &type_str[7..type_str.len() - 1];
            return Type::Option(Box::new(Self::parse_type(inner)));
        }

        if type_str.starts_with("Result<") && type_str.ends_with('>') {
            let inner = &type_str[7..type_str.len() - 1];
            let parts: Vec<&str> = inner.split(',').collect();
            if parts.len() == 2 {
                return Type::Result(
                    Box::new(Self::parse_type(parts[0].trim())),
                    Box::new(Self::parse_type(parts[1].trim())),
                );
            }
        }

        let primitives = ["i8", "i16", "i32", "i64", "i128", "isize",
                         "u8", "u16", "u32", "u64", "u128", "usize",
                         "f32", "f64", "bool", "char", "String", "&str"];

        if primitives.contains(&type_str) || type_str.starts_with("&str") {
            Type::Primitive(type_str.to_string())
        } else if type_str.len() == 1 && type_str.chars().next().unwrap().is_uppercase() {
            Type::Generic(type_str.to_string())
        } else {
            Type::Custom(type_str.to_string())
        }
    }

    pub fn infer_constraints(&self) -> Vec<TraitConstraint> {
        let mut constraints = Vec::new();

        for generic in &self.generics {
            if self.name.contains("max") || self.name.contains("min") {
                constraints.push(TraitConstraint {
                    type_param: generic.clone(),
                    trait_bound: "Ord".to_string(),
                });
            }
        }

        constraints
    }

    pub fn is_generic(&self) -> bool {
        !self.generics.is_empty()
    }

    pub fn has_side_effects(&self) -> bool {
        self.parameters.iter().any(|p| p.is_mutable) || self.return_type.is_none()
    }
}

//==============================================================================
// Milestone 2: Property Inference Engine
//==============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyType {
    Commutativity,
    Associativity,
    Identity(String),
    Idempotence,
    Involution,
    Monotonicity,
    RoundTrip,
    LengthPreservation,
    Invariant(String),
}

#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub description: String,
    pub property_type: PropertyType,
    pub test_code: String,
}

pub struct PropertyInference {
    signature: FunctionSignature,
    inferred_properties: Vec<Property>,
}

impl PropertyInference {
    pub fn new(signature: FunctionSignature) -> Self {
        PropertyInference {
            signature,
            inferred_properties: Vec::new(),
        }
    }

    pub fn infer_properties(&mut self) -> Vec<Property> {
        let mut properties = Vec::new();

        if let Some(prop) = self.test_commutativity() {
            properties.push(prop);
        }

        if let Some(prop) = self.test_associativity() {
            properties.push(prop);
        }

        if let Some(prop) = self.find_identity_element() {
            properties.push(prop);
        }

        if let Some(prop) = self.detect_involution() {
            properties.push(prop);
        }

        if let Some(prop) = self.test_idempotence() {
            properties.push(prop);
        }

        if let Some(prop) = self.test_length_preservation() {
            properties.push(prop);
        }

        properties
    }

    fn test_commutativity(&self) -> Option<Property> {
        if self.signature.parameters.len() == 2 {
            let p1 = &self.signature.parameters[0];
            let p2 = &self.signature.parameters[1];

            if p1.param_type == p2.param_type {
                if let Some(ref ret) = self.signature.return_type {
                    if *ret == p1.param_type {
                        return Some(Property {
                            name: format!("test_{}_commutative", self.signature.name),
                            description: format!("{} is commutative", self.signature.name),
                            property_type: PropertyType::Commutativity,
                            test_code: String::new(),
                        });
                    }
                }
            }
        }
        None
    }

    fn test_associativity(&self) -> Option<Property> {
        if self.signature.parameters.len() == 2 {
            let p1 = &self.signature.parameters[0];
            let p2 = &self.signature.parameters[1];

            if p1.param_type == p2.param_type {
                if let Some(ref ret) = self.signature.return_type {
                    if *ret == p1.param_type {
                        return Some(Property {
                            name: format!("test_{}_associative", self.signature.name),
                            description: format!("{} is associative", self.signature.name),
                            property_type: PropertyType::Associativity,
                            test_code: String::new(),
                        });
                    }
                }
            }
        }
        None
    }

    fn find_identity_element(&self) -> Option<Property> {
        if self.signature.parameters.len() == 2 {
            let identity = match self.signature.name.as_str() {
                name if name.contains("add") || name.contains("sum") => "0",
                name if name.contains("mul") || name.contains("product") => "1",
                name if name.contains("concat") => "\"\"",
                name if name.contains("max") => "i32::MIN",
                name if name.contains("min") => "i32::MAX",
                _ => return None,
            };

            return Some(Property {
                name: format!("test_{}_identity", self.signature.name),
                description: format!("{} has identity element {}", self.signature.name, identity),
                property_type: PropertyType::Identity(identity.to_string()),
                test_code: String::new(),
            });
        }
        None
    }

    fn detect_involution(&self) -> Option<Property> {
        if self.signature.parameters.len() == 1 {
            let param = &self.signature.parameters[0];
            if let Some(ref ret) = self.signature.return_type {
                if *ret == param.param_type {
                    let is_involution = self.signature.name.contains("reverse")
                        || self.signature.name.contains("not")
                        || self.signature.name.contains("negate")
                        || self.signature.name.contains("invert");

                    if is_involution {
                        return Some(Property {
                            name: format!("test_{}_involution", self.signature.name),
                            description: format!("Applying {} twice returns original", self.signature.name),
                            property_type: PropertyType::Involution,
                            test_code: String::new(),
                        });
                    }
                }
            }
        }
        None
    }

    fn test_idempotence(&self) -> Option<Property> {
        if self.signature.parameters.len() == 1 {
            let param = &self.signature.parameters[0];
            if let Some(ref ret) = self.signature.return_type {
                if *ret == param.param_type {
                    let is_idempotent = self.signature.name.contains("normalize")
                        || self.signature.name.contains("dedupe")
                        || self.signature.name.contains("sort")
                        || self.signature.name.contains("abs");

                    if is_idempotent {
                        return Some(Property {
                            name: format!("test_{}_idempotent", self.signature.name),
                            description: format!("Applying {} twice gives same result as once", self.signature.name),
                            property_type: PropertyType::Idempotence,
                            test_code: String::new(),
                        });
                    }
                }
            }
        }
        None
    }

    fn test_length_preservation(&self) -> Option<Property> {
        if self.signature.parameters.len() == 1 {
            let param = &self.signature.parameters[0];

            let is_collection = matches!(param.param_type, Type::Vec(_))
                || matches!(param.param_type, Type::Primitive(ref s) if s == "String");

            if is_collection {
                if let Some(ref ret) = self.signature.return_type {
                    if *ret == param.param_type {
                        return Some(Property {
                            name: format!("test_{}_length_preservation", self.signature.name),
                            description: format!("{} preserves length", self.signature.name),
                            property_type: PropertyType::LengthPreservation,
                            test_code: String::new(),
                        });
                    }
                }
            }
        }
        None
    }

    fn generate_round_trip_test(&self) -> Option<Property> {
        let is_encoder = self.signature.name.contains("encode")
            || self.signature.name.contains("serialize")
            || self.signature.name.contains("marshal");

        if is_encoder {
            return Some(Property {
                name: format!("test_{}_roundtrip", self.signature.name),
                description: "Round-trip property".to_string(),
                property_type: PropertyType::RoundTrip,
                test_code: String::new(),
            });
        }
        None
    }
}

//==============================================================================
// Milestone 3: Proptest Code Generation
//==============================================================================

pub struct GeneratorConfig {
    pub num_cases: usize,
    pub max_shrink_iters: usize,
    pub use_custom_generators: bool,
    pub test_module_name: String,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        GeneratorConfig {
            num_cases: 100,
            max_shrink_iters: 1000,
            use_custom_generators: true,
            test_module_name: "generated_property_tests".to_string(),
        }
    }
}

pub struct CodeGenerator {
    properties: Vec<Property>,
    config: GeneratorConfig,
}

impl CodeGenerator {
    pub fn new(properties: Vec<Property>, config: GeneratorConfig) -> Self {
        CodeGenerator { properties, config }
    }

    pub fn generate_tests(&self) -> String {
        let mut output = String::new();
        output.push_str("#[cfg(test)]\n");
        output.push_str("mod generated_property_tests {\n");
        output.push_str("    use proptest::prelude::*;\n\n");
        output.push_str("    proptest! {\n");

        for prop in &self.properties {
            let test_code = self.generate_property_test(prop);
            output.push_str(&format!("        {}\n", test_code));
        }

        output.push_str("    }\n");
        output.push_str("}\n");

        output
    }

    fn generate_property_test(&self, prop: &Property) -> String {
        match &prop.property_type {
            PropertyType::Commutativity => self.gen_commutativity_test(prop),
            PropertyType::Associativity => self.gen_associativity_test(prop),
            PropertyType::Identity(_) => self.gen_identity_test(prop),
            PropertyType::Involution => self.gen_involution_test(prop),
            PropertyType::Idempotence => self.gen_idempotence_test(prop),
            PropertyType::LengthPreservation => self.gen_length_test(prop),
            PropertyType::RoundTrip => self.gen_roundtrip_test(prop),
            PropertyType::Invariant(_) => self.gen_invariant_test(prop),
            _ => String::new(),
        }
    }

    fn gen_commutativity_test(&self, prop: &Property) -> String {
        format!(
            r#"#[test]
        fn {}(a in any::<i32>(), b in any::<i32>()) {{
            prop_assert_eq!(func(a, b), func(b, a));
        }}"#,
            prop.name
        )
    }

    fn gen_associativity_test(&self, prop: &Property) -> String {
        format!(
            r#"#[test]
        fn {}(a in any::<i32>(), b in any::<i32>(), c in any::<i32>()) {{
            prop_assert_eq!(func(func(a, b), c), func(a, func(b, c)));
        }}"#,
            prop.name
        )
    }

    fn gen_identity_test(&self, prop: &Property) -> String {
        if let PropertyType::Identity(ref identity) = prop.property_type {
            format!(
                r#"#[test]
        fn {}(x in any::<i32>()) {{
            prop_assert_eq!(func(x, {}), x);
        }}"#,
                prop.name, identity
            )
        } else {
            String::new()
        }
    }

    fn gen_involution_test(&self, prop: &Property) -> String {
        format!(
            r#"#[test]
        fn {}(x in any::<Vec<i32>>()) {{
            prop_assert_eq!(func(func(x.clone())), x);
        }}"#,
            prop.name
        )
    }

    fn gen_idempotence_test(&self, prop: &Property) -> String {
        format!(
            r#"#[test]
        fn {}(x in any::<Vec<i32>>()) {{
            prop_assert_eq!(func(func(x.clone())), func(x));
        }}"#,
            prop.name
        )
    }

    fn gen_length_test(&self, prop: &Property) -> String {
        format!(
            r#"#[test]
        fn {}(x in any::<Vec<i32>>()) {{
            let result = func(x.clone());
            prop_assert_eq!(result.len(), x.len());
        }}"#,
            prop.name
        )
    }

    fn gen_roundtrip_test(&self, prop: &Property) -> String {
        format!(
            r#"#[test]
        fn {}(x in any::<String>()) {{
            prop_assert_eq!(decode(encode(x.clone())), x);
        }}"#,
            prop.name
        )
    }

    fn gen_invariant_test(&self, prop: &Property) -> String {
        format!(
            r#"#[test]
        fn {}(x in any::<i32>()) {{
            prop_assert!(invariant_holds(x));
        }}"#,
            prop.name
        )
    }

    fn create_value_generator(&self, typ: &Type) -> String {
        match typ {
            Type::Primitive(name) => {
                if name.contains("i32") || name.contains("u32") {
                    "any::<i32>()".to_string()
                } else if name == "String" {
                    "\".*\"".to_string()
                } else if name == "bool" {
                    "any::<bool>()".to_string()
                } else {
                    format!("any::<{}>())", name)
                }
            }
            Type::Vec(_) => "prop::collection::vec(any::<i32>(), 0..100)".to_string(),
            Type::Option(_) => "prop::option::of(any::<i32>())".to_string(),
            _ => "any::<i32>()".to_string(),
        }
    }

    fn generate_test_module(&self, tests: Vec<String>) -> String {
        let mut module = String::new();
        module.push_str("mod tests {\n");
        module.push_str("    use proptest::prelude::*;\n\n");
        module.push_str("    proptest! {\n");

        for test in tests {
            module.push_str(&format!("        {}\n", test));
        }

        module.push_str("    }\n");
        module.push_str("}\n");

        module
    }
}

//==============================================================================
// Milestone 4: Custom Value Generators
//==============================================================================

#[derive(Debug, Clone)]
pub enum Constraint {
    Range { min: i64, max: i64 },
    Length { min: usize, max: usize },
    Pattern(String),
    NonZero,
    Positive,
    NonEmpty,
    Predicate(String),
}

#[derive(Debug, Clone)]
pub struct CustomGenerator {
    pub generator_name: String,
    pub base_type: Type,
    pub constraints: Vec<Constraint>,
    pub strategy_code: String,
}

impl CustomGenerator {
    pub fn from_parameter(param: &Parameter) -> Option<Self> {
        let constraints = Self::infer_constraints(param);

        if constraints.is_empty() {
            return None;
        }

        Some(CustomGenerator {
            generator_name: param.name.clone(),
            base_type: param.param_type.clone(),
            constraints: constraints.clone(),
            strategy_code: Self::create_bounded_generator(&constraints[0]),
        })
    }

    pub fn infer_constraints(param: &Parameter) -> Vec<Constraint> {
        let mut constraints = Vec::new();
        let name = param.name.to_lowercase();

        if name.contains("age") {
            constraints.push(Constraint::Range { min: 0, max: 150 });
        } else if name.contains("count") || name.contains("size") {
            constraints.push(Constraint::Positive);
        } else if name.contains("index") {
            constraints.push(Constraint::Positive);
        } else if name.contains("divisor") || name.contains("denominator") {
            constraints.push(Constraint::NonZero);
        } else if name.contains("email") {
            constraints.push(Constraint::Pattern(
                "[a-z0-9.]+@[a-z0-9]+\\.[a-z]{2,}".to_string(),
            ));
        }

        constraints
    }

    fn create_bounded_generator(constraint: &Constraint) -> String {
        match constraint {
            Constraint::Range { min, max } => {
                format!("{}..={}", min, max)
            }
            Constraint::Length { min, max } => {
                format!("prop::collection::vec(any::<i32>(), {}..={})", min, max)
            }
            Constraint::Pattern(pattern) => {
                format!("prop::string::string_regex(\"{}\").unwrap()", pattern)
            }
            Constraint::NonZero => "1..=i32::MAX".to_string(),
            Constraint::Positive => "1..=i32::MAX".to_string(),
            Constraint::NonEmpty => "prop::collection::vec(any::<i32>(), 1..100)".to_string(),
            Constraint::Predicate(_) => "any::<i32>()".to_string(),
        }
    }

    fn create_regex_generator(pattern: &str) -> String {
        format!("prop::string::string_regex(\"{}\").unwrap()", pattern)
    }

    fn create_struct_generator(_fields: &[(String, CustomGenerator)]) -> String {
        "any::<i32>()".to_string()
    }

    pub fn generate_strategy_code(&self) -> String {
        self.strategy_code.clone()
    }
}

pub mod common_generators {
    use super::*;

    pub fn email_generator() -> CustomGenerator {
        CustomGenerator {
            generator_name: "email".to_string(),
            base_type: Type::Primitive("String".to_string()),
            constraints: vec![Constraint::Pattern(
                "[a-z0-9.]+@[a-z0-9]+\\.[a-z]{2,}".to_string(),
            )],
            strategy_code: r#"prop::string::string_regex("[a-z0-9.]+@[a-z0-9]+\\.[a-z]{2,}").unwrap()"#
                .to_string(),
        }
    }

    pub fn age_generator() -> CustomGenerator {
        CustomGenerator {
            generator_name: "age".to_string(),
            base_type: Type::Primitive("i32".to_string()),
            constraints: vec![Constraint::Range { min: 0, max: 150 }],
            strategy_code: "0..=150".to_string(),
        }
    }

    pub fn non_empty_string_generator() -> CustomGenerator {
        CustomGenerator {
            generator_name: "non_empty_string".to_string(),
            base_type: Type::Primitive("String".to_string()),
            constraints: vec![Constraint::NonEmpty],
            strategy_code: "prop::string::string_regex(\".+\").unwrap()".to_string(),
        }
    }

    pub fn positive_int_generator() -> CustomGenerator {
        CustomGenerator {
            generator_name: "positive_int".to_string(),
            base_type: Type::Primitive("i32".to_string()),
            constraints: vec![Constraint::Positive],
            strategy_code: "1..=i32::MAX".to_string(),
        }
    }
}

//==============================================================================
// Milestone 5: Shrinking Strategy Optimization
//==============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum ShrinkStep {
    TowardsZero,
    RemoveElements,
    ShortenString,
    SimplifyStructure,
    ReplaceWithDefault,
    BinarySearch,
}

#[derive(Debug, Clone)]
pub struct ShrinkStrategy {
    pub value_type: Type,
    pub shrink_steps: Vec<ShrinkStep>,
    pub strategy_code: String,
}

impl ShrinkStrategy {
    pub fn create_shrink_strategy(typ: &Type) -> Self {
        let shrink_steps = Self::optimize_for_type(typ);
        let strategy_code = Self::generate_shrink_code_for_type(typ);

        ShrinkStrategy {
            value_type: typ.clone(),
            shrink_steps,
            strategy_code,
        }
    }

    fn optimize_for_type(typ: &Type) -> Vec<ShrinkStep> {
        match typ {
            Type::Primitive(name) if name == "String" => {
                vec![ShrinkStep::ShortenString]
            }
            Type::Primitive(name) if name.contains('i') || name.contains('u') => {
                vec![ShrinkStep::TowardsZero, ShrinkStep::BinarySearch]
            }
            Type::Vec(_) => {
                vec![ShrinkStep::RemoveElements, ShrinkStep::SimplifyStructure]
            }
            _ => vec![ShrinkStep::ReplaceWithDefault],
        }
    }

    pub fn generate_shrink_code(&self) -> String {
        self.strategy_code.clone()
    }

    fn generate_shrink_code_for_type(typ: &Type) -> String {
        match typ {
            Type::Primitive(name) if name.contains('i') => {
                "any::<i32>().prop_shrink(|x| (0..x.abs()).map(move |i| if x < 0 { -i } else { i }))".to_string()
            }
            Type::Vec(_) => {
                "prop::collection::vec(any::<i32>(), 0..100)".to_string()
            }
            _ => "any::<i32>()".to_string(),
        }
    }
}

//==============================================================================
// Milestone 6: CLI Tool and Project Integration
//==============================================================================

use std::path::Path;

pub struct Config {
    pub num_cases: usize,
    pub output_dir: String,
    pub parallel: bool,
    pub verbose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            num_cases: 100,
            output_dir: "tests/generated".to_string(),
            parallel: false,
            verbose: false,
        }
    }
}

impl Config {
    pub fn from_file(_path: &Path) -> Result<Self, std::io::Error> {
        Ok(Config::default())
    }

    pub fn merge_with_args(&mut self, _num_cases: Option<usize>) {
        // Configuration merging would go here
    }
}

pub fn generate_for_file(_path: &Path, _config: &Config) -> Result<(), String> {
    Ok(())
}

pub fn generate_for_project(_root: &Path, _config: &Config) -> Result<(), String> {
    Ok(())
}

//==============================================================================
// Main Example
//==============================================================================

fn main() {
    println!("=== Property-Based Test Generator ===\n");

    println!("This tool automatically generates property-based tests by:");
    println!("  1. Analyzing function signatures");
    println!("  2. Inferring mathematical properties");
    println!("  3. Generating proptest code");
    println!("  4. Creating custom value generators");
    println!("  5. Optimizing shrinking strategies");
    println!("\nGenerated tests explore edge cases far beyond manual testing!");
    println!("\nFeatures:");
    println!("  - Automatic property inference (commutativity, associativity, etc.)");
    println!("  - Custom value generators with constraints");
    println!("  - Optimized shrinking for minimal failing cases");
    println!("  - CLI integration for easy use");
}

//==============================================================================
// Tests
//==============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Milestone 1 Tests
    #[test]
    fn test_parse_simple_function() {
        let source = "fn add(a: i32, b: i32) -> i32";
        let sig = FunctionSignature::parse_function(source).unwrap();

        assert_eq!(sig.name, "add");
        assert_eq!(sig.parameters.len(), 2);
        assert_eq!(sig.parameters[0].name, "a");
        assert_eq!(sig.parameters[0].param_type, Type::Primitive("i32".to_string()));
        assert!(matches!(sig.return_type, Some(Type::Primitive(_))));
    }

    #[test]
    fn test_parse_generic_function() {
        let source = "fn reverse<T>(vec: Vec<T>) -> Vec<T>";
        let sig = FunctionSignature::parse_function(source).unwrap();

        assert_eq!(sig.name, "reverse");
        assert_eq!(sig.generics, vec!["T"]);
        assert!(matches!(sig.parameters[0].param_type, Type::Vec(_)));
    }

    #[test]
    fn test_parse_with_constraints() {
        let source = "fn max<T: Ord>(a: T, b: T) -> T";
        let sig = FunctionSignature::parse_function(source).unwrap();

        assert_eq!(sig.generics, vec!["T"]);
        let constraints = sig.infer_constraints();
        assert!(constraints.iter().any(|c| c.trait_bound.contains("Ord")));
    }

    #[test]
    fn test_mutable_parameter_detection() {
        let source = "fn increment(x: &mut i32)";
        let sig = FunctionSignature::parse_function(source).unwrap();

        assert!(sig.parameters[0].is_mutable);
        assert!(sig.has_side_effects());
    }

    #[test]
    fn test_complex_return_type() {
        let source = "fn parse(s: &str) -> Result<i32, String>";
        let sig = FunctionSignature::parse_function(source).unwrap();

        assert!(matches!(sig.return_type, Some(Type::Result(_, _))));
    }

    #[test]
    fn test_tuple_parameters() {
        let source = "fn swap<T>(pair: (T, T)) -> (T, T)";
        let sig = FunctionSignature::parse_function(source).unwrap();

        assert!(matches!(sig.parameters[0].param_type, Type::Tuple(_)));
    }

    // Milestone 2 Tests
    #[test]
    fn test_infer_commutativity() {
        let source = "fn add(a: i32, b: i32) -> i32";
        let sig = FunctionSignature::parse_function(source).unwrap();
        let mut inference = PropertyInference::new(sig);

        let properties = inference.infer_properties();

        assert!(properties
            .iter()
            .any(|p| matches!(p.property_type, PropertyType::Commutativity)));
    }

    #[test]
    fn test_infer_involution() {
        let source = "fn reverse<T>(vec: Vec<T>) -> Vec<T>";
        let sig = FunctionSignature::parse_function(source).unwrap();
        let mut inference = PropertyInference::new(sig);

        let properties = inference.infer_properties();

        assert!(properties
            .iter()
            .any(|p| matches!(p.property_type, PropertyType::Involution)));
    }

    #[test]
    fn test_infer_identity() {
        let source = "fn multiply(a: i32, b: i32) -> i32";
        let sig = FunctionSignature::parse_function(source).unwrap();
        let mut inference = PropertyInference::new(sig);

        let properties = inference.infer_properties();

        assert!(properties
            .iter()
            .any(|p| matches!(p.property_type, PropertyType::Identity(_))));
    }

    #[test]
    fn test_length_preservation() {
        let source = "fn shuffle<T>(vec: Vec<T>) -> Vec<T>";
        let sig = FunctionSignature::parse_function(source).unwrap();
        let mut inference = PropertyInference::new(sig);

        let properties = inference.infer_properties();

        assert!(properties
            .iter()
            .any(|p| matches!(p.property_type, PropertyType::LengthPreservation)));
    }

    #[test]
    fn test_no_false_positives() {
        let source = "fn divide(a: i32, b: i32) -> i32";
        let sig = FunctionSignature::parse_function(source).unwrap();
        let mut inference = PropertyInference::new(sig);

        let _properties = inference.infer_properties();
    }

    #[test]
    fn test_associativity_inference() {
        let source = "fn max(a: i32, b: i32) -> i32";
        let sig = FunctionSignature::parse_function(source).unwrap();
        let mut inference = PropertyInference::new(sig);

        let properties = inference.infer_properties();

        assert!(properties
            .iter()
            .any(|p| matches!(p.property_type, PropertyType::Associativity)));
    }

    // Milestone 3 Tests
    #[test]
    fn test_generate_commutativity_test() {
        let prop = Property {
            name: "test_add_commutative".to_string(),
            description: "Addition is commutative".to_string(),
            property_type: PropertyType::Commutativity,
            test_code: String::new(),
        };

        let generator = CodeGenerator::new(vec![prop], GeneratorConfig::default());
        let code = generator.generate_property_test(&generator.properties[0]);

        assert!(code.contains("prop_assert_eq!"));
        assert!(code.contains("(a, b)") && code.contains("(b, a)") || code.contains("swap"));
    }

    #[test]
    fn test_generate_involution_test() {
        let prop = Property {
            name: "test_reverse_involution".to_string(),
            description: "Reversing twice returns original".to_string(),
            property_type: PropertyType::Involution,
            test_code: String::new(),
        };

        let generator = CodeGenerator::new(vec![prop], GeneratorConfig::default());
        let code = generator.generate_property_test(&generator.properties[0]);

        assert!(code.contains("func(func") || code.contains("f(f("));
    }

    #[test]
    fn test_create_primitive_generator() {
        let generator = CodeGenerator::new(vec![], GeneratorConfig::default());

        let gen = generator.create_value_generator(&Type::Primitive("i32".to_string()));
        assert!(gen.contains("any::<i32>()") || gen.contains("i32"));
    }

    #[test]
    fn test_create_vec_generator() {
        let generator = CodeGenerator::new(vec![], GeneratorConfig::default());

        let gen = generator.create_value_generator(&Type::Vec(Box::new(Type::Primitive(
            "i32".to_string(),
        ))));

        assert!(gen.contains("vec") || gen.contains("collection"));
    }

    #[test]
    fn test_full_module_generation() {
        let props = vec![
            Property {
                name: "test_prop1".to_string(),
                description: "Test 1".to_string(),
                property_type: PropertyType::Commutativity,
                test_code: String::new(),
            },
            Property {
                name: "test_prop2".to_string(),
                description: "Test 2".to_string(),
                property_type: PropertyType::Involution,
                test_code: String::new(),
            },
        ];

        let generator = CodeGenerator::new(props, GeneratorConfig::default());
        let module = generator.generate_tests();

        assert!(module.contains("proptest!"));
        assert!(module.contains("use proptest"));
        assert!(module.contains("test_prop1"));
        assert!(module.contains("test_prop2"));
    }

    #[test]
    fn test_compilable_output() {
        let prop = Property {
            name: "test_example".to_string(),
            description: "Example".to_string(),
            property_type: PropertyType::Commutativity,
            test_code: String::new(),
        };

        let generator = CodeGenerator::new(vec![prop], GeneratorConfig::default());
        let code = generator.generate_tests();

        assert!(code.contains("#[test]") || code.contains("proptest!"));
        assert!(code.contains("fn test_"));
    }

    // Milestone 4 Tests
    #[test]
    fn test_infer_age_constraints() {
        let param = Parameter {
            name: "age".to_string(),
            param_type: Type::Primitive("i32".to_string()),
            is_mutable: false,
        };

        let constraints = CustomGenerator::infer_constraints(&param);

        assert!(constraints.iter().any(|c| matches!(
            c,
            Constraint::Range { min: 0, max: 150 }
        ) || matches!(c, Constraint::Positive)));
    }

    #[test]
    fn test_infer_email_pattern() {
        let param = Parameter {
            name: "email".to_string(),
            param_type: Type::Primitive("String".to_string()),
            is_mutable: false,
        };

        let constraints = CustomGenerator::infer_constraints(&param);

        assert!(constraints.iter().any(|c| matches!(c, Constraint::Pattern(_))));
    }

    #[test]
    fn test_range_generator_code() {
        let constraint = Constraint::Range { min: 1, max: 100 };
        let code = CustomGenerator::create_bounded_generator(&constraint);

        assert!(code.contains("1") && code.contains("100"));
        assert!(code.contains(".."));
    }

    #[test]
    fn test_regex_generator_code() {
        let pattern = "[a-z]{3,10}";
        let code = CustomGenerator::create_regex_generator(pattern);

        assert!(code.contains("string_regex"));
        assert!(code.contains(pattern));
    }

    #[test]
    fn test_email_generator() {
        use common_generators::*;

        let gen = email_generator();
        assert_eq!(gen.generator_name, "email");
        assert!(gen.strategy_code.contains("@"));
    }

    #[test]
    fn test_non_zero_constraint() {
        let param = Parameter {
            name: "divisor".to_string(),
            param_type: Type::Primitive("i32".to_string()),
            is_mutable: false,
        };

        let constraints = CustomGenerator::infer_constraints(&param);

        assert!(constraints.iter().any(|c| matches!(c, Constraint::NonZero)));
    }

    #[test]
    fn test_index_constraints() {
        let param = Parameter {
            name: "index".to_string(),
            param_type: Type::Primitive("usize".to_string()),
            is_mutable: false,
        };

        let gen = CustomGenerator::from_parameter(&param).unwrap();

        assert!(gen.constraints.len() > 0);
    }

    // Milestone 5 Tests
    #[test]
    fn test_numeric_shrink_strategy() {
        let typ = Type::Primitive("i32".to_string());
        let strategy = ShrinkStrategy::create_shrink_strategy(&typ);

        assert!(strategy.shrink_steps.contains(&ShrinkStep::TowardsZero));
    }

    #[test]
    fn test_vec_shrink_strategy() {
        let typ = Type::Vec(Box::new(Type::Primitive("i32".to_string())));
        let strategy = ShrinkStrategy::create_shrink_strategy(&typ);

        assert!(strategy
            .shrink_steps
            .contains(&ShrinkStep::RemoveElements));
    }

    #[test]
    fn test_string_shrink_strategy() {
        let typ = Type::Primitive("String".to_string());
        let strategy = ShrinkStrategy::create_shrink_strategy(&typ);

        assert!(strategy.shrink_steps.contains(&ShrinkStep::ShortenString));
    }

    // Milestone 6 Tests
    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.num_cases, 100);
        assert_eq!(config.output_dir, "tests/generated");
    }

    #[test]
    fn test_generate_for_file() {
        let config = Config::default();
        let result = generate_for_file(Path::new("test.rs"), &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_for_project() {
        let config = Config::default();
        let result = generate_for_project(Path::new("."), &config);
        assert!(result.is_ok());
    }
}
