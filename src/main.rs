use windows_metadata::{self, File as MetadataFile, Item, Reader, Type, TypeKind};
use reqwest;
use std::any::Any;
use std::io::{Read, Write};
use std::fs::File;
use std::ops::Deref;
use std::path::Path;
use serde::Serialize;
// https://github.com/microsoft/windows-rs/raw/master/crates/libs/bindgen/default/Windows.Win32.winmd

const USE_CACHE: bool = true;

// TODO: Make it prettier goddamnit
fn download_metadata(url: &str) -> Vec<u8> {
    if USE_CACHE == true {
        let file_name = String::from(url);
        let file_name = file_name.split("/").last().unwrap();
        let file_name = format!("winmd_cache/{}", file_name);
        let cache_path = Path::new(file_name.as_str());
        if !cache_path.exists() {
            let response = reqwest::blocking::get(url).unwrap();
            let response = response.bytes().unwrap();
            let mut file = File::create(cache_path).unwrap();
            let _ = file.write_all(&response);
            return response.into();
        } else {
            let mut file = File::open(cache_path).unwrap();
            let mut response: Vec<u8> = vec![];
            file.read_to_end(&mut response).unwrap();
            return response.into();
        }
    } else {
        let response = reqwest::blocking::get(url).unwrap();
        let response = response.bytes().unwrap();
        return response.into();
    }   
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
    params:         Vec<String>,
}

#[derive(Serialize)]
struct Enum {
    name:           String,
    members:        Vec<String>,
}

#[derive(Serialize)]
struct Struct {
    name:           String,
    members:        Vec<String>,
}

struct Result {
    function_def:   Vec<Module>,
    struct_def:     Vec<Struct>,
    enum_def:       Vec<Enum>,
}

fn populate_result(result: &mut Result, data: &Reader) {
    for data in data.items() {
        match data {
            Item::Const(_) => {
                // println!("Item::Const {:?} ", field.name());
                // if field.name().to_lowercase() == "pszusername" {
                //     println!("Field: {:?}", field.name());
                // }
            },
            Item::Fn(method_def, _) => {
                // print!("\n");
                // let method = reader.get_type_def(namespace, name)
                let func_name   = method_def.name().to_string();
                let module_name = method_def.module_name().to_string();
                
                let mut result_index = result.function_def
                    .iter()
                    .position(|module| module.module_name == module_name);

                if result_index.is_none() {
                    result.function_def.push(Module {
                        module_name,
                        functions: Vec::new(),
                    });
                    result_index = Some(result.function_def.len() - 1);
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
                let module = result.function_def.get_mut(result_index.unwrap()).unwrap();
                module.functions.push(Function {
                    function_name: func_name,
                    ret_type: return_type, 
                    params,
                });
                // println!("{} : {} {}()", module_name, return_type, func_name);
            },
            Item::Type(type_def) => {
                let type_def_type = type_def.kind();

                let fields = type_def.fields()
                    .map(|field| { 
                        let field_type = field.ty(Some(type_def));
                        let field_type = parse_type(&field_type);
                        
                        let res = if type_def_type == TypeKind::Enum {
                            format!("{}", field.name())
                        } else {
                            format!("{}{}", field_type, field.name())
                        };
                        
                        res
                    })
                    .filter(|field| { !field.contains("value__") })
                    .collect::<Vec<String>>();

                match type_def_type {
                    TypeKind::Struct => {
                        let res_struct = Struct {
                            name: String::from(type_def.name()),
                            members: fields,
                        };
                        result.struct_def.push(res_struct);
                    },
                    TypeKind::Enum => {
                        let res_enum = Enum {
                            name: String::from(type_def.name()),
                            members: fields,
                        };
                        result.enum_def.push(res_enum);
                    },
                    _ => {}
                }
                // if type_def.name().to_lowercase() == "pszusername" {
                
                // let 
                // println!("{:?} {:?} {:?}", type_def_type, type_def.name(), fields);
                // }
            },
        };
    }
}

fn save_output(path: &str, data: (impl Serialize + Sized)) {
    let res = serde_json::to_string_pretty(&data).unwrap();
    let mut handler = File::create(path).unwrap();
    
    handler.write_all(res.as_bytes()).unwrap();
}

fn main() {
    let mut result = Result {
        enum_def: Vec::new(),
        function_def: Vec::new(),
        struct_def: Vec::new(),
    };
    // let mut class_def: Vec<Class>        = vec![];
    for (i, metadata_url) in vec![
        "https://github.com/microsoft/windows-rs/raw/master/crates/libs/bindgen/default/Windows.Win32.winmd",
        "https://github.com/microsoft/windows-rs/raw/master/crates/libs/bindgen/default/Windows.Wdk.winmd",
        "https://github.com/microsoft/windows-rs/raw/master/crates/libs/bindgen/default/Windows.winmd"
    ].iter().enumerate() {
        let metadata = MetadataFile::new(download_metadata(metadata_url)).unwrap();
        let reader = windows_metadata::Reader::new(vec![metadata]);
        
        populate_result(&mut result, &reader);
    }

    save_output("./output.json", &result.function_def);
    save_output("./enums.json", &result.enum_def);
    save_output("./structs.json", &result.struct_def);
}
