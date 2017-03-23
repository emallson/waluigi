#![allow(dead_code)]
use structs::*;

error_chain! {
    types {
        Error, ErrorKind, ChainErr, Result;
    }

    links {

    }
    
    foreign_links {
        IO(::std::io::Error);
        Yaml(::serde_yaml::Error);
    }

    errors {
        FieldMismatch(dtype: FieldType, datum: FieldData) {
            description("field did not match datum used to fill it")
            display("field of type {:?} did not match datum {:?} used to fill it", dtype, datum)
        }

        InvalidProgram(name: String, options: Vec<String>) {
            description("unknown program found in experiment spec")
            display("unknown program {} found in spec. available: {}", name, options.join(", "))
        }

        MissingParameter(name: String, program: String) {
            description("parameter missing for program specification")
            display("parameter {} missing for {}", name, program)
        }

        InvalidParameterSetting(name: String, setting: FieldSetting, dtype: FieldType) {
            description("invalid parameter setting for field")
            display("invalid parameter setting {:?} for field {} of type {:?}", setting, name, dtype)
        }

        InvalidParameterData(name: String, data: FieldData, dtype: FieldType) {
            description("invalid parameter data for field")
            display("invalid parameter data {:?} for field {} of type {:?}", data, name, dtype)
        }

        UnknownDependency(job: String, dependency: String) {
            description("job has unknown dependency")
            display("job {} has {} listed as a dependency, but no previous job provides {}", job, dependency, dependency)
        }
    }
}
