use windows_metadata::{self, File as MetadataFile, Item};
use reqwest;
use std::io::Write;
use std::fs::File;
use serde::Serialize;
// https://github.com/microsoft/windows-rs/raw/master/crates/libs/bindgen/default/Windows.Win32.winmd

// TODO: Make it prettier goddamnit
fn download_metadata() -> Vec<u8> {
    // let cache_path = Path::new("./windows.winmd");
    // if !cache_path.exists() {
        let response = reqwest::blocking::get("https://github.com/microsoft/windows-rs/raw/master/crates/libs/bindgen/default/Windows.Win32.winmd").unwrap();
        let response = response.bytes().unwrap();
        // let mut file = File::create(cache_path).unwrap();
        // let _ = file.write_all(&response);
        return response.into();
    // } else {
    //     let mut file = File::open(cache_path).unwrap();
    //     let mut response: Vec<u8> = vec![];
    //     file.read_to_end(&mut response).unwrap();
    //     return response.into();
    // }
}

fn parse_type(type_name: &windows_metadata::Type) -> String {
    match type_name {
        windows_metadata::Type::Name(type_name) => {
            let mut r = String::new();
            r += type_name.1;
            r += " ";
            r
        },
        windows_metadata::Type::MutPtr(type_name_inner, ..) => {
            format!("*{}", parse_type(type_name_inner))
        },
        windows_metadata::Type::TypeDef(type_def, ..) => {
            format!("{} ", type_def.name())
        }
        _ => format!("{:?} ", type_name),
    }
}

#[derive(Serialize)]
struct Module {
    module_name:    String,
    functions:      Vec<Function>,
}

#[derive(Serialize)]
struct Function {
    function_name:  String,
    ret_type:       String,
    params:     Vec<String>,
}

fn main() {
    let metadata = MetadataFile::new(download_metadata()).unwrap();
    let reader = windows_metadata::Reader::new(vec![metadata]);
    let mut result: Vec<Module> = vec![];
    for data in reader.items() {
        match data {
            Item::Const(_) => {
                // print!("{:?} ", field.name());
                // if field.name().to_lowercase() == "pszusername" {
                //     println!("Field: {:?}", field.name());
                // }
            },
            Item::Fn(method_def, _) => {
                // print!("\n");
                // let method = reader.get_type_def(namespace, name)
                let func_name   = method_def.name().to_string();
                let module_name = method_def.module_name().to_string();
                
                let mut result_index = result
                    .iter()
                    .position(|module| module.module_name == module_name);

                if result_index.is_none() {
                    result.push(Module {
                        module_name,
                        functions: Vec::new(),
                    });
                    result_index = Some(result.len() - 1);
                }
                
                let func_signature = method_def.signature(&[]);
                let return_type = parse_type(&func_signature.return_type).trim().to_string();

                let params = {
                    let mut res: Vec<String> = Vec::new();
                    let param_names = method_def.params().map(|x| x.name()).filter(|&x| x != "").collect::<Vec<&str>>();
                    for i in 0..param_names.len() {
                        let t = func_signature.params.get(i).unwrap();
                        let el = format!("{}{}", parse_type(t), param_names.get(i).unwrap());
                        // el.concat(param_names.get(i).unwrap());
                        res.push(el);
                    }
                    res
                };
                // println!("Function: {} : {}({:?})", module_name, func_name, params); 
                // println!("Module: {} | Function: {} {}({})", module_name, return_type, func_name, params.join(", "));
                let module = result.get_mut(result_index.unwrap()).unwrap();
                module.functions.push(Function {
                    function_name: func_name,
                    ret_type: return_type, 
                    params,
                });
                // println!("{} : {} {}()", module_name, return_type, func_name);
            },
            Item::Type(_) => {                
                // if type_def.name().to_lowercase() == "pszusername" {
                // print!("{:?} ", type_def.name());
                // }
            },
        };
    }
    
    let res = serde_json::to_string_pretty(&result).unwrap();
    
    let mut handler = File::create("./output.json").unwrap();
    
    handler.write_all(res.as_bytes()).unwrap();
}