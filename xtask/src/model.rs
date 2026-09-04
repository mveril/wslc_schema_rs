#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: CppType,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CppType {
    String,
    Bool,
    I32,
    I64,
    Vector(Box<Self>),
    Map(Box<Self>),
    Optional(Box<Self>),
    Named(String),
}
