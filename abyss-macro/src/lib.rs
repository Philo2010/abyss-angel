use std::{ffi::OsStr, fs, path::Path};

use proc_macro::{Span, TokenStream};
use quote::ToTokens;
use syn::{
	Attribute, Ident,
	Item::{Enum, Struct, Type, Union},
	ItemEnum, Meta, MetaNameValue, Token, parse_file, parse_macro_input,
	punctuated::Punctuated,
};
use walkdir::WalkDir;

const ENUM_DISPATCH: &'static str = "enum_dispatch";

fn remove_enum_dispatch(mut item: ItemEnum) -> TokenStream {
	let mut enum_dispatch_index = None;

	for (index, attr) in item.attrs.iter().enumerate() {
		match &attr.meta {
			Meta::Path(path) => {
				if path.to_token_stream().to_string() == ENUM_DISPATCH {
					enum_dispatch_index = Some(index);
				}
			}
			Meta::List(list) => {
				if list.path.to_token_stream().to_string() == ENUM_DISPATCH {
					enum_dispatch_index = Some(index);
				}
			}
			_ => {}
		}
	}

	if let Some(index) = enum_dispatch_index {
		item.attrs.remove(index);
	}

	item.to_token_stream().into()
}

fn valid_variant(enum_name: &Ident, attrs: Vec<Attribute>) -> bool {
	for attr in attrs {
		if let Meta::List(list) = attr.meta {
			if list.path.to_token_stream().to_string() != "enum_builder_variant" {
				continue;
			}

			if list.tokens.to_token_stream().to_string() == enum_name.to_string() {
				return true;
			}
		}
	}

	false
}

#[proc_macro_attribute]
pub fn enum_builder(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let parsed_item = parse_macro_input!(item);
    let Enum(item_enum) = parsed_item else {
        return parsed_item.to_token_stream().into();
    };
    let Some(dir) = Span::call_site().local_file() else {
        return remove_enum_dispatch(item_enum);
    };
    let mut dir = dir.parent().unwrap().to_owned();
    
    // Store both the variant name and the type info
    let mut enum_variants: Vec<(String, String, String, String)> = vec![]; // (ident, full_path, generics)
    
    let attrs =
        parse_macro_input!(attrs with Punctuated::<MetaNameValue, Token![,]>::parse_terminated);
    
    for attr in attrs {
        let name = attr.path.to_token_stream().to_string();
        if name != "path" {
            continue;
        }
        dir = dir.join(
            attr.value
                .to_token_stream()
                .to_string()
                .trim_matches('"')
                .to_owned(),
        );
    }
    
    let base_path = dir.clone();
    
    for entry in WalkDir::new(&dir) {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if path.extension() != Some(OsStr::new("rs")) {
            continue;
        };
        
        // Get the file name without extension for disambiguation
        let file_name = path.file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        
        let relative_path = path.strip_prefix(&base_path).unwrap();
        let module_path = path_to_module_path(relative_path);
        
        let src = fs::read_to_string(path)
            .expect(format!("unable to read file {}", path.to_string_lossy()).as_str());
        let syntax = parse_file(&src)
            .expect(format!("unable to parse file {}", path.to_string_lossy()).as_str());
        
        for item in syntax.items {
            let ident;
            let generics;
            match item {
                Struct(item) => {
                    if !valid_variant(&item_enum.ident, item.attrs) {
                        continue;
                    }
                    ident = item.ident;
                    generics = item.generics.to_token_stream().to_string();
                }
                Type(item) => {
                    if !valid_variant(&item_enum.ident, item.attrs) {
                        continue;
                    }
                    ident = item.ident;
                    generics = item.generics.to_token_stream().to_string();
                }
                Enum(item) => {
                    if !valid_variant(&item_enum.ident, item.attrs) {
                        continue;
                    }
                    ident = item.ident;
                    generics = item.generics.to_token_stream().to_string();
                }
                Union(item) => {
                    if !valid_variant(&item_enum.ident, item.attrs) {
                        continue;
                    }
                    ident = item.ident;
                    generics = item.generics.to_token_stream().to_string();
                }
                _ => continue,
            }
            
            let full_path = if module_path.is_empty() {
                ident.to_string()
            } else {
                format!("{}::{}", module_path, ident)
            };
            
            // Store the type name, full path, generics, and file name
            enum_variants.push((
                ident.to_string(),
                full_path,
                generics,
                file_name.clone()
            ));
        }
    }
    
    if enum_variants.is_empty() {
        return remove_enum_dispatch(item_enum);
    }
    
    // Build final variants with disambiguation
    let final_variants: Vec<String> = enum_variants.iter().map(|(ident, full_path, generics, file_name)| {
        let variant_name = if ident == "Model" {
            // Capitalize first letter of file name
            let capitalized_file = capitalize_first(&file_name);
            format!("{}Model", capitalized_file)
        } else {
            ident.clone()
        };
        
        format!("{}({}{})", variant_name, full_path, generics)
    }).collect();
    
    format!(
        "#[enum_dispatch]\npub enum {} {{ {} }}",
        item_enum.ident,
        final_variants.join(",\n")
    )
    .parse()
    .unwrap()
}

// Helper function to capitalize first letter
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

// Helper function to convert file path to module path
fn path_to_module_path(path: &Path) -> String {
    let mut components = vec![];
    
    for component in path.components() {
        let comp_str = component.as_os_str().to_string_lossy();
        
        // Skip "mod.rs" files - they represent the parent module
        if comp_str == "mod.rs" {
            break;
        }
        
        // Remove .rs extension and add to path
        if let Some(module_name) = comp_str.strip_suffix(".rs") {
            components.push(module_name.to_string());
        } else if comp_str != "." && comp_str != ".." {
            components.push(comp_str.to_string());
        }
    }
    
    components.join("::")
}

#[proc_macro_attribute]
pub fn enum_builder_variant(_: TokenStream, item: TokenStream) -> TokenStream {
	item
}