#[derive(Debug, Clone, PartialEq)]
pub enum Schema {
    Null,
    Bool,
    Number {
        min: Option<f64>,
        max: Option<f64>,
        integer_only: bool,
    },
    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
        pattern: Option<String>,
    },
    Array {
        items: Box<Schema>,
        min_items: Option<usize>,
        max_items: Option<usize>,
    },
    Object {
        properties: HashMap<String, PropertySchema>,
        required: Vec<String>,
        additional_properties: bool,
    },
    Any,
    OneOf(Vec<Schema>),
    Const(Value),
}
