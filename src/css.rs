pub struct Stylesheet {
    rules: Vec<Rule>,
}

impl Stylesheet {
    pub fn new() -> Stylesheet {
        Stylesheet { rules: vec![] }
    }
}

pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

impl Rule {
    pub fn new() -> Rule {
        Rule { selectors: vec![], declarations: vec![] }
    }
}

pub enum Selector {
    Simple(SimpleSelector),
}

pub type Specificity = (usize, usize, usize);
impl Selector {
    pub fn specificity(&self) -> Specificity {
        let Selector::Simple(ref simple) = *self;
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag_name.iter().count();

        (a, b, c)
    }
}

pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}

pub struct Declaration {
    pub name: String,
    pub value: Value,
}

pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    Color(Color),
    // insert more values here
}

pub enum Unit {
    Px,
}

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

