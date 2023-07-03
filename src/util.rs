// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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

pub fn is_alphaspace<T: AsRef<str>>(value: T) -> bool {
    value
        .as_ref()
        .to_ascii_lowercase()
        .bytes()
        .all(|ch| b'a' <= ch && ch <= b'z' || ch == b' ')
}
