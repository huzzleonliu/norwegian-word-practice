use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VerbForms {
    // 动词：原型（一般式），过去式，祈使式
    pub base_form: String,
    pub past_tense: Option<String>,
    pub imperative: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NounForms {
    // 名词：原型（单数），复数，单数特指，复数特指
    pub singular: String,
    pub plural: Option<String>,
    pub singular_definite: Option<String>,
    pub plural_definite: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdjectiveForms {
    // 形容词：原型（阴/阳性），中性，复数，比较级，最高级泛指，最高级特指
    pub common_gender: String,
    pub neuter: Option<String>,
    pub plural: Option<String>,
    pub comparative: Option<String>,
    pub superlative_indefinite: Option<String>,
    pub superlative_definite: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdverbForms {
    // 副词：原级，比较级，最高级
    pub base_form: String,
    pub comparative: Option<String>,
    pub superlative: Option<String>,
}
