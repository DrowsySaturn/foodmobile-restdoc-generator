extern crate regex;
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::io::{Error, ErrorKind};

static FUNCTION_PATTERN : &str = r#"@(Post|Put|Get)Mapping\(path\s*=\s*"(.+?)".+?\)\s+(public|private|protected)\s+(\S+)\s+([^(]+)\((.+?)\)"#;
static PARAM_PATTERN : &str = r#"@RequestParam\s+(.+?)\s+([A-Za-z]+)"#;
static NESTED_TYPE_PATTERN : &str = r#"DataModelResponse<(.+?)>"#;

struct FunctionMatch {
    method: Box<String>,
    path: Box<String>,
    access: Box<String>,
    result_type: Box<String>,
    func_name: Box<String>,
    params_str: Box<String>
}

fn extract_function_groups(file_name : &str) -> std::io::Result<Vec<FunctionMatch>> {
    if !Path::new(file_name).exists() {
        return Err(Error::new(ErrorKind::Other, "File does not exist"));
    }
    let mut file = File::open(file_name)?;
    let mut file_data = String::new();
    file.read_to_string(&mut file_data)?;
    let re = Regex::new(FUNCTION_PATTERN).unwrap();
    let mut result : Vec<FunctionMatch> = vec!();
    for cap in re.captures_iter(&mut file_data) {
        let func = FunctionMatch {
            method: Box::new(String::from(&cap[1])),
            path: Box::new(String::from(&cap[2])),
            access: Box::new(String::from(&cap[3])),
            result_type: Box::new(String::from(&cap[4])),
            func_name: Box::new(String::from(&cap[5])),
            params_str: Box::new(String::from(&cap[6]))
        };
        result.push(func);
    }
    Ok(result)
}

fn generate_doc_param(param_str : &String) -> String {
    let re = Regex::new(PARAM_PATTERN).unwrap();
    let mut response = String::from("");
    for cap in re.captures_iter(param_str) {
        let type_name = &cap[1];
        let param_name = &cap[2];
        response = String::from(&format!("{}- {} : {}\n", response, param_name, type_name));
    }
    response
}

fn pull_nested_type(ret_type : &String) -> String {
    let re = Regex::new(NESTED_TYPE_PATTERN).unwrap();
    let caps = re.captures(ret_type).unwrap();
    return String::from(&caps[1]);
}

fn deserialize_return_type(ret_type : &String) -> String {
    if ret_type == "SimpleStatusResponse" {
        return String::from(&format!("[{}](../com/foodmobile/server/datamodels/{}.html)", ret_type, ret_type));
    } else if ret_type == "LoginResponse" {
        return String::from(&format!("[{}](../com/foodmobile/server/datamodels/{}.html)", ret_type, ret_type));
    } else if ret_type.starts_with("DataModelResponse") {
        let nested_type = pull_nested_type(ret_type);
        return String::from(&format!("[{}](../com/foodmobile/server/datamodels/{}.html) [{}](../com/foodmobile/server/datamodels/{}.html)", "DataModelResponse", "DataModelResponse", nested_type, nested_type));
    } else if ret_type.starts_with("MultiDataModelResponse") {
        let nested_type = pull_nested_type(ret_type);
        return String::from(&format!("[{}](../com/foodmobile/server/datamodels/{}.html) [{}](../com/foodmobile/server/datamodels/{}.html)", "MultiDataModelResponse", "MultiDataModelResponse", nested_type, nested_type));
    }
    return String::from("");
}

fn generate_doc_str(func_match : &FunctionMatch) -> String {
    let param_str = generate_doc_param(&*func_match.params_str);
    let link = deserialize_return_type(&*func_match.result_type);
    format!("#### {} {}\n---\n{}\n---\n{}\n", func_match.method, func_match.path, param_str, link)
}

fn main() -> std::io::Result<()> {
    let args : Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: controller_matcher controller.java");
        return Err(Error::new(ErrorKind::Other, "Usage error"));
    }
    let file_name = &args[1];
    let match_result = extract_function_groups(&file_name);
    let doc_path = format!("{}{}", file_name, ".md");
    let mut doc_messages : Vec<String> = vec!();
    if let Ok(matches) = &match_result {
        for mat in matches {
            let doc = generate_doc_str(&mat);
            doc_messages.push(String::from(doc));
        }
    } else if let Err(err) = &match_result {
        println!("Failed: {:?}", err);
        return Err(Error::new(ErrorKind::Other, "Error"));
    }
    let mut output_file = File::create(doc_path)?;
    for doc_msg in doc_messages {
        output_file.write_all(&doc_msg.into_bytes())?;
    }
    output_file.sync_all()?;
    Ok(())
}
