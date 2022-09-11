#![feature(proc_macro_expand, proc_macro_span)]

extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Ident, LitByteStr};

/// A field in a uniform block struct with an ident and type
struct UniformField<'a> {
    ident: &'a Ident,
    ty: &'a syn::Type
}

/// A macro that loads and preprocesses a shader at compile time
#[proc_macro]
pub fn preprocess_shader_vf(args: TokenStream) -> TokenStream {
    // Expand macro invocation
    let args_expanded = args.expand_expr().expect("Failed to expand expression");

    // Get literal byte string (should be shader source)
    let args_byte_str: LitByteStr = syn::parse(args_expanded).expect("Expected byte string");

    // Get the shader source as a utf8 string
    let shader_source_bytes = args_byte_str.value().clone();
    let shader_source = std::str::from_utf8(&shader_source_bytes).expect("Failed to parse utf-8 string");

    // Split the shader source into the version directive and the rest
    let (version, rest) = split_version_directive(&shader_source);

    // Preprocess vertex shader
    println!("Preprocessing vertex shader source");
    let vertex_shader = {
        let mut context = gpp::Context::new();
        context.macros.insert("BUILDING_VERTEX_SHADER".to_string(), "1".to_string());
        let processed = gpp::process_str(&rest, &mut context)
            .expect("failed to preprocess vertex shader");
        format!("{}\n{}", version, processed)
    };

    // Preprocess fragment shader
    println!("Preprocessing fragment shader source");
    let fragment_shader = {
        let mut context = gpp::Context::new();
        context.macros.insert("BUILDING_FRAGMENT_SHADER".to_string(), "1".to_string());
        let processed = gpp::process_str(&rest, &mut context)
            .expect("failed to preprocess fragment shader");
        format!("{}\n{}", version, processed)
    };

    TokenStream::from(quote! { 
        dreamfield_renderer::resources::ShaderSource::VertexFragment(#vertex_shader, #fragment_shader)
    })
}

/// A macro that loads and preprocesses a shader at compile time, with tessellation
#[proc_macro]
pub fn preprocess_shader_vtf(args: TokenStream) -> TokenStream {
    // Expand macro invocation
    let args_expanded = args.expand_expr().expect("Failed to expand expression");

    // Get literal byte string (should be shader source)
    let args_byte_str: LitByteStr = syn::parse(args_expanded).expect("Expected byte string");

    // Get the shader source as a utf8 string
    let shader_source_bytes = args_byte_str.value().clone();
    let shader_source = std::str::from_utf8(&shader_source_bytes).expect("Failed to parse utf-8 string");

    // Split the shader source into the version directive and the rest
    let (version, rest) = split_version_directive(&shader_source);

    // Preprocess vertex shader
    println!("Preprocessing vertex shader source");
    let vertex_shader = {
        let mut context = gpp::Context::new();
        context.macros.insert("TESSELLATION_ENABLED".to_string(), "1".to_string());
        context.macros.insert("BUILDING_VERTEX_SHADER".to_string(), "1".to_string());
        let processed = gpp::process_str(&rest, &mut context)
            .expect("failed to preprocess vertex shader");
        format!("{}\n{}", version, processed)
    };

    // Preprocess tessellation control shader
    println!("Preprocessing tessellation control shader source");
    let tess_control_shader = {
        let mut context = gpp::Context::new();
        context.macros.insert("TESSELLATION_ENABLED".to_string(), "1".to_string());
        context.macros.insert("BUILDING_TESS_CONTROL_SHADER".to_string(), "1".to_string());
        let processed = gpp::process_str(&rest, &mut context)
            .expect("failed to preprocess tessellation control shader");
        format!("{}\n{}", version, processed)
    };

    // Preprocess tessellation evaluation shader
    println!("Preprocessing tessellation evaluation shader source");
    let tess_eval_shader = {
        let mut context = gpp::Context::new();
        context.macros.insert("TESSELLATION_ENABLED".to_string(), "1".to_string());
        context.macros.insert("BUILDING_TESS_EVAL_SHADER".to_string(), "1".to_string());
        let processed = gpp::process_str(&rest, &mut context)
            .expect("failed to preprocess tessellation evaluation shader");
        format!("{}\n{}", version, processed)
    };

    // Preprocess fragment shader
    println!("Preprocessing fragment shader source");
    let fragment_shader = {
        let mut context = gpp::Context::new();
        context.macros.insert("TESSELLATION_ENABLED".to_string(), "1".to_string());
        context.macros.insert("BUILDING_FRAGMENT_SHADER".to_string(), "1".to_string());
        let processed = gpp::process_str(&rest, &mut context)
            .expect("failed to preprocess fragment shader");
        format!("{}\n{}", version, processed)
    };


    TokenStream::from(quote! {
        dreamfield_renderer::resources::ShaderSource::VertexTessFragment(
            #vertex_shader, #tess_control_shader, #tess_eval_shader, #fragment_shader)
    })
}

/// Split a shader source at the version directive
fn split_version_directive(source: &str) -> (String, String) {
    let mut version_directive = String::new();
    let mut remainder = String::new();

    let mut version_directive_found = false;
    for line in source.lines() {
        if !version_directive_found {
            version_directive.push_str(line);
            version_directive.push('\n');
        }
        else {
            remainder.push_str(line);
            remainder.push('\n');
        }

        if !version_directive_found && line.contains("#version") {
            version_directive_found = true;
        }
    }

    (version_directive, remainder)
}

/// A macro that adds uniform setters to a struct, along with dirty ranges
#[proc_macro_derive(UniformSetters, attributes(field_size))]
pub fn uniform_setters_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    println!("Generating setters for {name}");

    // Get fields we want to generate setters for
    let mut fields: Vec<UniformField> = Vec::new();
    if let syn::Data::Struct(struct_data) = &input.data {
        for field in &struct_data.fields {
            if let Some(ident) = &field.ident {
                fields.push(UniformField {
                    ident,
                    ty: &field.ty
                });
            }
        }
    }

    // Generate field offsets function
    let mut field_offsets = quote! { };
    for field in fields.iter() {
        let ty_name = field.ty;
        field_offsets.extend(quote! {
            std::mem::size_of::<#ty_name>(),
        });
    }

    // Generate UniformSetters trait impl
    let mut setters_impl = quote! {
        impl dreamfield_traits::UniformSetters for #name {
            fn calculate_field_offsets() -> Vec<usize> {
                vec![#field_offsets]
            }
        }
    };

    // Generate setter for each field
    for (i, field) in fields.iter().enumerate() {
        let field_name = field.ident;
        let field_type = field.ty;

        let setter_name_str = format!("set_{}", &field.ident);
        let setter_name = Ident::new(&setter_name_str, field.ident.span());

        // Check if it's an array type
        let array_type = match field_type {
            syn::Type::Path(path) => {
                let tok = path.to_token_stream().to_string();
                if tok.starts_with("std140 :: array < ") {
                    let truncated_start = tok[18..].to_string();
                    let comma_offset = truncated_start.find(',').unwrap();
                    let inner_type = truncated_start[..comma_offset].to_string();
                    Some(inner_type)
                }
                else {
                    None
                }
            },
            _ => None
        };

        // Generate array setter
        if let Some(array_type) = array_type {
            let array_type_ident = Ident::new(&array_type, field_name.span());
            setters_impl.extend(quote! {
                impl UniformBuffer<#name> {
                    pub fn #setter_name(&mut self, idx: usize, val: &#array_type_ident) {
                        self.data.#field_name.internal[idx] = std140::ArrayElementWrapper { element: *val };

                        // Calculate modified range
                        let (base_offset, _) = self.field_offsets[#i];
                        let field_size = std::mem::size_of::<std140::ArrayElementWrapper<#array_type_ident>>();
                        let field_offset = base_offset + idx * field_size;
                        let field_end = field_offset + field_size;

                        self.modified_ranges.insert(field_offset..field_end);
                    }
                }
            });
        }
        // Generate standard setter
        else {
            setters_impl.extend(quote! {
                impl UniformBuffer<#name> {
                    pub fn #setter_name<T: ToStd140<#field_type>>(&mut self, val: &T) {
                        let (field_offset, field_end) = self.field_offsets[#i];
                        self.data.#field_name = val.to_std140();
                        self.modified_ranges.insert(field_offset..field_end);
                    }
                }
            });
        }
    }

    TokenStream::from(setters_impl)
}
