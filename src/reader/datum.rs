use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Datum {
    Boolean(bool),
    Number(i64),
    Character(char),
    String(String),
    Symbol(String),
    Pair(Box<Datum>, Box<Datum>),
    Vector(Vec<Datum>),
    ByteVector(Vec<u8>),
    EmptyList,
}

impl Datum {
    pub fn symbol(name: impl Into<String>) -> Self {
        Self::Symbol(name.into())
    }

    pub fn pair(car: Datum, cdr: Datum) -> Self {
        Self::Pair(Box::new(car), Box::new(cdr))
    }

    pub fn list(items: Vec<Datum>) -> Self {
        Self::list_with_tail(items, Self::EmptyList)
    }

    pub fn list_with_tail(items: Vec<Datum>, tail: Datum) -> Self {
        items
            .into_iter()
            .rev()
            .fold(tail, |cdr, car| Self::pair(car, cdr))
    }

    pub fn as_symbol(&self) -> Option<&str> {
        match self {
            Self::Symbol(name) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn collect_proper_list(&self) -> Option<Vec<&Datum>> {
        let mut items = Vec::new();
        let mut current = self;
        loop {
            match current {
                Self::EmptyList => return Some(items),
                Self::Pair(car, cdr) => {
                    items.push(car.as_ref());
                    current = cdr.as_ref();
                }
                _ => return None,
            }
        }
    }
}

impl fmt::Display for Datum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(value) => write!(f, "{}", if *value { "#t" } else { "#f" }),
            Self::Number(value) => write!(f, "{value}"),
            Self::Character(value) => write!(f, "{}", fmt_character(*value)),
            Self::String(value) => write!(f, "\"{}\"", value),
            Self::Symbol(value) => write!(f, "{value}"),
            Self::Vector(values) => {
                write!(f, "#(")?;
                fmt_sequence(values, f)?;
                write!(f, ")")
            }
            Self::ByteVector(values) => {
                write!(f, "#u8(")?;
                fmt_bytevector(values, f)?;
                write!(f, ")")
            }
            Self::EmptyList => write!(f, "()"),
            Self::Pair(_, _) => {
                write!(f, "(")?;
                fmt_pair(self, f)?;
                write!(f, ")")
            }
        }
    }
}

fn fmt_sequence(values: &[Datum], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            write!(f, " ")?;
        }
        write!(f, "{value}")?;
    }
    Ok(())
}

fn fmt_bytevector(values: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            write!(f, " ")?;
        }
        write!(f, "{value}")?;
    }
    Ok(())
}

pub(crate) fn fmt_character(value: char) -> String {
    match value {
        ' ' => "#\\space".to_string(),
        '\n' => "#\\newline".to_string(),
        ch => format!("#\\{ch}"),
    }
}

fn fmt_pair(datum: &Datum, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut first = true;
    let mut current = datum;

    loop {
        match current {
            Datum::Pair(car, cdr) => {
                if !first {
                    write!(f, " ")?;
                }
                write!(f, "{car}")?;
                current = cdr;
                first = false;
            }
            Datum::EmptyList => return Ok(()),
            other => {
                write!(f, " . {other}")?;
                return Ok(());
            }
        }
    }
}
