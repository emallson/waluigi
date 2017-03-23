use errors::*;

use std::collections::HashMap;
use std::string::ToString;
use regex::Regex;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Str,
    Path,
    #[serde(rename = "uint")]
    UInt,
    Float,
    Bool,
}

impl FieldType {
    pub fn matches(&self, data: &FieldData) -> bool {
        match data {
            &FieldData::Str(_) => self == &FieldType::Str || self == &FieldType::Path,
            &FieldData::UInt(_) => self == &FieldType::UInt,
            &FieldData::Float(f) => {
                self == &FieldType::Float || (self == &FieldType::UInt && f.trunc() == f)
            }
            &FieldData::Bool(_) => self == &FieldType::Bool,
            &FieldData::Future => self == &FieldType::Str,
        }
    }

    pub fn matches_setting(&self, data: &FieldSetting) -> bool {
        match data {
            &FieldSetting::Range { ref from, ref to, ref step } => {
                (self == &FieldType::UInt || self == &FieldType::Float) && self.matches(from) &&
                self.matches(to) && self.matches(step)
            }
            &FieldSetting::List(ref data) => data.iter().all(|datum| self.matches(datum)),
            &FieldSetting::Value(ref v) => self.matches(v),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
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

impl Field {
    pub fn matches(&self, datum: &FieldData) -> bool {
        self.dtype.matches(&datum)
    }

    pub fn fill_with(&self, datum: &FieldData) -> Result<String> {
        if self.matches(datum) {
            if let Some(ref opt) = self.option {
                match datum {
                    &FieldData::Bool(false) => Ok("".to_string()),
                    &FieldData::Bool(true) => Ok(opt.clone()),
                    _ => {
                        let rep: &str = &datum.to_string();
                        Ok(Regex::new(r"<.+?>")
                            .unwrap()
                            .replace(&opt, rep)
                            .to_string())
                    }
                }
            } else {
                Ok(datum.to_string())
            }
        } else {
            Err(ErrorKind::FieldMismatch(self.dtype, datum.clone()).into())
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
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

impl Program {
    pub fn cmd(&self, params: &HashMap<String, FieldData>) -> Result<String> {
        let mut fmt = format!("{} {}", self.bin, self.format);
        for (field, datum) in params {
            if self.fields.contains_key(field) && self.fields[field].matches(&datum) {
                if self.fields[field].option.is_none() {
                    let fname = format!("<{}>", field);
                    fmt = fmt.replace(&fname, &self.fields[field].fill_with(datum)?);
                } else {
                    fmt.push_str(&self.fields[field].fill_with(datum)?);
                }
            }
        }
        return Ok(fmt);
    }

    pub fn validate_parameters(&self, params: &HashMap<String, FieldSetting>) -> Result<()> {
        // every field must either be filled or be optional (as indicated by the option: foo field
        // on the field object)
        for (field, details) in &self.fields {
            if !params.contains_key(field) && details.option.is_none() {
                return Err(ErrorKind::MissingParameter(field.clone(), self.name.clone()).into());
            }

            if !params.contains_key(field) {
                continue;
            }

            let ref param = params[field];
            if !details.dtype.matches_setting(param) {
                return Err(ErrorKind::InvalidParameterSetting(field.clone(),
                                                              param.clone(),
                                                              details.dtype)
                    .into());
            }
        }

        Ok(())
    }

    pub fn validate_parameter_data(&self, params: &HashMap<String, FieldData>) -> Result<()> {
        // every field must either be filled or be optional (as indicated by the option: foo field
        // on the field object)
        for (field, details) in &self.fields {
            if !params.contains_key(field) && details.option.is_none() {
                return Err(ErrorKind::MissingParameter(field.clone(), self.name.clone()).into());
            }

            if !params.contains_key(field) {
                continue;
            }

            let ref param = params[field];
            if !details.dtype.matches(param) {
                return Err(ErrorKind::InvalidParameterData(field.clone(),
                                                           param.clone(),
                                                           details.dtype)
                    .into());
            }
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum FieldData {
    Future,
    Str(String),
    Float(f64),
    UInt(usize),
    Bool(bool),
}

impl FieldData {
    pub fn unwrap_usize(&self) -> usize {
        match self {
            &FieldData::UInt(u) => u,
            _ => panic!(),
        }
    }

    pub fn unwrap_float(&self) -> f64 {
        match self {
            &FieldData::Float(u) => u,
            _ => panic!(),
        }
    }
}

impl ToString for FieldData {
    fn to_string(&self) -> String {
        match self {
            &FieldData::Str(ref s) => s.clone(),
            &FieldData::UInt(v) => v.to_string(),
            &FieldData::Float(v) => v.to_string(),
            &FieldData::Bool(v) => v.to_string(),
            &FieldData::Future => "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

impl FieldSetting {
    pub fn vectorize(&self) -> Vec<FieldData> {
        match self {
            &FieldSetting::Range { ref from, ref to, ref step } => {
                let mut range = Vec::new();
                match from {
                    &FieldData::UInt(start) => {
                        let end = to.unwrap_usize();
                        let step = step.unwrap_usize();
                        let mut cur = start;

                        while cur <= end {
                            range.push(FieldData::UInt(cur));
                            cur += step;
                        }
                    }
                    &FieldData::Float(start) => {
                        let end = to.unwrap_float();
                        let step = step.unwrap_float();
                        let mut cur = start;

                        while cur <= end {
                            range.push(FieldData::Float(cur));
                            cur += step;
                        }
                    }
                    _ => unreachable!(),
                }
                range
            }
            &FieldSetting::List(ref v) => v.clone(),
            &FieldSetting::Value(ref v) => vec![v.clone()],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Job {
    run: String,
    parameters: HashMap<String, FieldSetting>,
    repetitions: Option<usize>,
    on_each: Option<Vec<String>>,
}

impl Job {
    pub fn has_depends(&self) -> bool {
        self.on_each.is_some()
    }

    pub fn batch(&self) -> Result<Vec<HashMap<String, FieldData>>> {
        let mut param_sets = HashMap::new();

        for (field, param) in &self.parameters {
            param_sets.insert(field.clone(), param.vectorize());
        }

        fn prod(params: HashMap<String, Vec<FieldData>>) -> Vec<HashMap<String, FieldData>> {
            let key = params.keys().next();
            if let Some(key) = key {
                let mut cl = params.clone();
                cl.remove(key);
                let subproducts = prod(cl);

                subproducts.iter()
                    .flat_map(|prod| {
                        params[key]
                            .iter()
                            .map(|datum| {
                                let mut p = prod.clone();
                                p.insert(key.clone(), datum.clone());
                                p
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect()
            } else {
                vec![HashMap::new()]
            }
        };

        let res = prod(param_sets);
        let rl = res.len();
        Ok(res.into_iter()
            .cycle()
            .enumerate()
            .map(|(i, mut params)| {
                params.insert(format!("repetition-{}", self.run), FieldData::UInt(i / rl));
                params
            })
            .take(self.repetitions.unwrap_or(1) * rl)
            .collect())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Experiment {
    jobs: Vec<Job>,
}

impl Experiment {
    /// Converts a sequence of Job specs into a sequence of job instances ready to be sent to the
    /// broker.
    pub fn plan(&self,
                threads: usize,
                programs: &HashMap<String, Program>)
                -> Result<Vec<JobInstance>> {
        let mut id = 0;
        let mut jobify = |prog: &Program, params, deps| {
            let inst = JobInstance {
                id: Some(id),
                command: try!(prog.cmd(&params)),
                params: params,
                log: "".to_string(),
                threads: threads,
                depends: deps,
            };

            id += 1;
            Ok(inst)
        };
        let mut jobmap: HashMap<String, Vec<JobInstance>> = HashMap::new();
        for job in &self.jobs {
            if !programs.contains_key(&job.run) {
                return Err(ErrorKind::InvalidProgram(job.run.clone(),
                                                     programs.keys().cloned().collect())
                    .into());
            }

            if !job.has_depends() {
                programs[&job.run].validate_parameters(&job.parameters)?;
                jobmap.insert(job.run.clone(),
                              job.batch()?
                                  .into_iter()
                                  .map(|params| jobify(&programs[&job.run], params, vec![]))
                                  .collect::<Result<_>>()?); // no dependencies, all params are local
            } else if let Some(ref deps) = job.on_each {
                let mut batch =
                    job.batch()?.into_iter().map(|params| (params, vec![])).collect::<Vec<_>>();
                for dep in deps {
                    if !jobmap.contains_key(dep) {
                        return Err(ErrorKind::UnknownDependency(job.run.clone(), dep.clone())
                            .into());
                    }

                    batch = batch.into_iter()
                        .flat_map(|(params, par_deps)| {
                            jobmap[dep]
                                .iter()
                                .map(|dep_params| {
                                    let mut p = params.clone();
                                    let mut pd = par_deps.clone();
                                    p.extend(dep_params.params.clone().into_iter());
                                    p.extend(programs[dep]
                                        .outputs
                                        .clone()
                                        .into_iter()
                                        .map(|(k, _)| (k, FieldData::Future)));
                                    pd.push(dep_params.id.unwrap());
                                    (p, pd)
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect();
                }

                jobmap.insert(job.run.clone(),
                              batch.into_iter()
                                  .map(|(params, deps)| {
                                      programs[&job.run]
                                          .validate_parameter_data(&params)
                                          .and_then(|_| jobify(&programs[&job.run], params, deps))
                                  })
                                  .collect::<Result<_>>()?);
            }
        }

        Ok(jobmap.into_iter()
            .flat_map(|(_, x)| x)
            .collect::<Vec<JobInstance>>())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct JobInstance {
    id: Option<usize>,
    command: String,
    params: HashMap<String, FieldData>,
    log: String,
    depends: Vec<usize>,
    threads: usize,
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_yaml;
    use std::fs::File;


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
        let _prob: Program = serde_yaml::from_reader(File::open("spec/curv.yaml").unwrap())
            .unwrap();
    }

    #[test]
    fn deser_problem_interdict() {
        let _prob: Program = serde_yaml::from_reader(File::open("spec/interdict.yaml").unwrap())
            .unwrap();
    }

    #[test]
    fn deser_problem_interdict_validate() {
        let _prob: Program =
            serde_yaml::from_reader(File::open("spec/interdict-validate.yaml").unwrap()).unwrap();
    }

    #[test]
    fn deser_exp_curv() {
        let _prob: Experiment = serde_yaml::from_reader(File::open("spec/exp-curv.yaml").unwrap())
            .unwrap();
    }

    #[test]
    fn deser_exp_interdict() {
        let _prob: Experiment =
            serde_yaml::from_reader(File::open("spec/exp-interdict.yaml").unwrap()).unwrap();
    }

    #[test]
    fn fill_no_option() {
        let field = Field {
            dtype: FieldType::UInt,
            aka: vec![],
            batch: BatchType::None,
            option: None,
        };

        assert!(field.fill_with(&FieldData::UInt(27)).unwrap() == "27".to_string());
    }

    #[test]
    fn fill_option_bool() {
        let field = Field {
            dtype: FieldType::Bool,
            aka: vec![],
            batch: BatchType::None,
            option: Some("--flag".to_string()),
        };

        assert!(field.fill_with(&FieldData::Bool(true)).unwrap() == "--flag".to_string());
        assert!(field.fill_with(&FieldData::Bool(false)).unwrap() == "".to_string());
    }

    #[test]
    fn fill_option_value() {
        let field = Field {
            dtype: FieldType::Float,
            aka: vec![],
            batch: BatchType::None,
            option: Some("--float <foo>".to_string()),
        };

        println!("{}", field.fill_with(&FieldData::Float(0.27)).unwrap());
        assert!(field.fill_with(&FieldData::Float(0.27)).unwrap() == "--float 0.27".to_string());
    }

    #[test]
    fn job_validate_curv() {
        let prog: Program = serde_yaml::from_reader(File::open("spec/curv.yaml").unwrap()).unwrap();
        let exp: Experiment = serde_yaml::from_reader(File::open("spec/exp-curv.yaml").unwrap())
            .unwrap();

        for job in &exp.jobs {
            prog.validate_parameters(&job.parameters).unwrap();
        }
    }

    #[test]
    #[should_panic]
    fn job_invalid_curv() {
        let prog: Program = serde_yaml::from_reader(File::open("spec/curv.yaml").unwrap()).unwrap();
        let exp: Experiment =
            serde_yaml::from_reader(File::open("spec/exp-interdict.yaml").unwrap()).unwrap();

        for job in &exp.jobs {
            prog.validate_parameters(&job.parameters).unwrap();
        }
    }

    #[test]
    fn job_batch_curv() {
        let exp: Experiment = serde_yaml::from_reader(File::open("spec/exp-curv.yaml").unwrap())
            .unwrap();

        for job in &exp.jobs {
            let batch = job.batch().unwrap();
            assert!(batch.len() == 2310)
        }
    }

    #[test]
    fn job_batch_interdict() {
        let exp: Experiment =
            serde_yaml::from_reader(File::open("spec/exp-interdict.yaml").unwrap()).unwrap();

        for (job, &size) in exp.jobs.iter().zip(&vec![330, 1]) {
            let batch = job.batch().unwrap();
            println!("{} {}", batch.len(), size);
            assert!(batch.len() == size)
        }
    }

    #[test]
    fn plan_curv() {
        let prog: Program = serde_yaml::from_reader(File::open("spec/curv.yaml").unwrap()).unwrap();
        let exp: Experiment = serde_yaml::from_reader(File::open("spec/exp-curv.yaml").unwrap())
            .unwrap();

        let map = hashmap!{
            "curv".to_string() => prog,
        };

        assert!(exp.plan(6, &map).unwrap().len() == 2310);
    }

    #[test]
    fn plan_interdict() {
        let prog: Program = serde_yaml::from_reader(File::open("spec/interdict.yaml").unwrap())
            .unwrap();
        let validate: Program =
            serde_yaml::from_reader(File::open("spec/interdict-validate.yaml").unwrap()).unwrap();
        let exp: Experiment =
            serde_yaml::from_reader(File::open("spec/exp-interdict.yaml").unwrap()).unwrap();

        let map = hashmap!{
            "interdict".to_string() => prog,
            "interdict-validate".to_string() => validate,
        };

        assert!(exp.plan(6, &map).unwrap().len() == 660);
    }
}
