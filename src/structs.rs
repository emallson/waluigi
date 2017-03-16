use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Str,
    Path,
    #[serde(rename = "uint")]
    UInt,
    Float,
    Bool,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BatchType {
    Max,
    Join(String),
    None,
}

impl Default for BatchType {
    fn default() -> Self {
        BatchType::None
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Field {
    #[serde(rename = "type")]
    dtype: FieldType,
    #[serde(default)]
    aka: Vec<String>,
    option: Option<String>,
    #[serde(default)]
    batch: BatchType,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Output {
    msg: String,
    #[serde(default)]
    aka: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Program {
    name: String,
    bin: String,
    format: String,
    outputs: HashMap<String, Output>,
    fields: HashMap<String, Field>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum FieldData {
    Str(String),
    UInt(usize),
    Float(f64),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum FieldSetting {
    Range {
        from: FieldData,
        to: FieldData,
        step: FieldData,
    },
    List(Vec<FieldData>),
    Value(FieldData),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Job {
    run: String,
    parameters: HashMap<String, FieldSetting>,
    repetitions: Option<usize>,
    on_each: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Experiment {
    jobs: Vec<Job>,
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_yaml;

    #[test]
    fn deser_field_simple() {
        let field_raw = "
            type: str
            ";

        let field: Field = serde_yaml::from_str(&field_raw).unwrap();

        assert!(field.dtype == FieldType::Str);
    }

    #[test]
    fn deser_field_full() {
        let field_raw = "type: float\naka: ['δ']\noption: '--delta <delta>'\nbatch: {join: ','}";
        let field: Field = serde_yaml::from_str(&field_raw).unwrap();

        assert!(field ==
                Field {
            dtype: FieldType::Float,
            aka: vec!["δ".to_string()],
            option: Some("--delta <delta>".to_string()),
            batch: BatchType::Join(",".to_string()),
        });
    }

    #[test]
    fn deser_output_simple() {
        let out_raw = "msg: \"approximation ratio\"";
        let out: Output = serde_yaml::from_str(&out_raw).unwrap();

        assert!(out ==
                Output {
            msg: "approximation ratio".to_string(),
            aka: vec![],
        });
    }

    #[test]
    fn deser_output_full() {
        let out_raw = "msg: approximation ratio\naka: ['ratio']";
        let out: Output = serde_yaml::from_str(&out_raw).unwrap();

        assert!(out ==
                Output {
            msg: "approximation ratio".to_string(),
            aka: vec!["ratio".to_string()],
        })
    }

    #[test]
    fn deser_problem_curv() {
        use std::fs::File;

        let _prob: Program = serde_yaml::from_reader(File::open("spec/curv.yaml").unwrap())
            .unwrap();
    }

    #[test]
    fn deser_problem_interdict() {
        use std::fs::File;

        let _prob: Program = serde_yaml::from_reader(File::open("spec/interdict.yaml").unwrap())
            .unwrap();
    }

    #[test]
    fn deser_problem_interdict_validate() {
        use std::fs::File;

        let _prob: Program =
            serde_yaml::from_reader(File::open("spec/interdict-validate.yaml").unwrap()).unwrap();
    }

    #[test]
    fn deser_exp_curv() {
        use std::fs::File;

        let _prob: Experiment = serde_yaml::from_reader(File::open("spec/exp-curv.yaml").unwrap())
            .unwrap();
    }

    #[test]
    fn deser_exp_interdict() {
        use std::fs::File;

        let _prob: Experiment =
            serde_yaml::from_reader(File::open("spec/exp-interdict.yaml").unwrap()).unwrap();
    }
}
