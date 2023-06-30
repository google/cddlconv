use convert_case::{Case, Casing};

fn split_identifier<T: ToString>(value: T) -> Vec<String> {
    return value.to_string().split('.').map(String::from).collect();
}

pub fn to_pascalcase<T: ToString>(value: T) -> String {
    value.to_string().to_case(Case::Pascal)
}

pub fn to_namespaced<T: ToString>(value: T) -> String {
    split_identifier(&value.to_string())
        .into_iter()
        .map(to_pascalcase)
        .collect::<Vec<String>>()
        .join(".")
}

pub fn split_namespaced<T: ToString>(value: T) -> (Vec<String>, String) {
    let mut parts = split_identifier(value.to_string())
        .into_iter()
        .map(to_pascalcase)
        .collect::<Vec<String>>();
    let value = parts.pop().unwrap();
    (parts, value)
}

pub fn is_alpha<T: AsRef<str>>(value: T) -> bool {
    value
        .as_ref()
        .to_ascii_lowercase()
        .bytes()
        .all(|ch| b'a' <= ch && ch <= b'z')
}
