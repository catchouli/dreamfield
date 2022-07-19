extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

struct UniformField<'a> {
    ident: &'a Ident,
    ty: &'a syn::Type
}

#[proc_macro_derive(UniformSetters)]
pub fn uniform_setters_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

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

    // Store name for later
    let name = input.ident;
    println!("Generating setters for {name}");

    // Generate token stream
    let mut setters_impl = quote! { impl dreamfield_traits::UniformSetters for #name { } };

    // Calculate field offsets
    let mut cur_offset: usize = 0;
    let field_offsets: Vec<usize> = fields.iter().map(|field| {
        let field_size = get_field_size_std140(field);
        let field_alignment = get_field_alignment_std140(field);

        let field_offset = align(cur_offset, field_alignment);

        cur_offset = field_offset + field_size;

        field_offset
    }).collect();

    // Calculate field ends, include padding up to next field so we can combine uploads.
    // We can do that just by taking the next element from field_offsets, and adding the final
    // offset to the end.
    let mut field_ends: Vec<usize> = field_offsets.iter().skip(1).cloned().collect();
    field_ends.push(cur_offset);

    // Get first field alignment and calculate element alignment
    let first_field_alignment = get_field_alignment_std140(fields.first().unwrap());
    let element_alignment = align(cur_offset, first_field_alignment);

    // Generate setter for each field
    for (i, field) in fields.iter().enumerate() {
        let field_offset = field_offsets[i];
        let field_end = field_ends[i];

        println!("  {} {} {}", field.ident, field_offset, field_end);

        let field_name = field.ident;
        let field_type = field.ty;

        let setter_name_str = format!("set_{}", &field.ident);
        let setter_name = Ident::new(&setter_name_str, field.ident.span());

        let multi_setter_name_str = format!("set_{}_n", &field.ident);
        let multi_setter_name = Ident::new(&multi_setter_name_str, field.ident.span());

        setters_impl.extend(quote! {
            impl UniformBuffer<#name> {
                pub fn #setter_name<T: ToStd140<#field_type>>(&mut self, val: &T) {
                    self.data[0].#field_name = val.to_std140();
                    self.modified_ranges.insert(#field_offset..#field_end);
                }

                pub fn #multi_setter_name<T: ToStd140<#field_type>>(&mut self, idx: usize, val: &T) {
                    let offset = idx * #element_alignment;
                    let end = offset - #field_offset + #field_end;
                    self.data[idx].#field_name = val.to_std140();
                    self.modified_ranges.insert(offset..end);
                }
            }
        });
    }

    TokenStream::from(setters_impl)
}

/// Get size for an std140 field
fn get_field_size_std140(field: &UniformField) -> usize {
    match &field.ty {
        syn::Type::Path(path) => {
            let type_name = &path.path.segments.last().unwrap().ident;
            get_type_size_std140(type_name)
        },
        _ => { panic!("Field {} did not have type Path", &field.ident); }
    }
}

/// Get size for an std140 type, including std140 alignment
fn get_type_size_std140(ident: &Ident) -> usize {
    println!("size of type {}", ident);
    match ident.to_string().as_str() {
        "float"   => 4,
        "vec2"    => 8,
        "vec3"    => 12,
        "vec4"    => 16,
        "int"     => 4,
        "ivec2"   => 8,
        "ivec3"   => 12,
        "ivec4"   => 16,
        "uint"    => 4,
        "uvec2"   => 8,
        "uvec3"   => 12,
        "uvec4"   => 16,
        "boolean" => 4,
        "bvec2"   => 8,
        "bvec3"   => 12,
        "bvec4"   => 16,
        "mat2x2"  => 2 * 8,
        "mat2x3"  => 3 * 12,
        "mat2x4"  => 2 * 16,
        "mat3x2"  => 3 * 8,
        "mat3x3"  => 3 * 12,
        "mat3x4"  => 3 * 16,
        "mat4x2"  => 4 * 8,
        "mat4x3"  => 4 * 12,
        "mat4x4"  => 4 * 16,
        _ => panic!("Attempted to get type size for unknown type {}", ident)
    }
}

/// Get alignment for an std140 field
fn get_field_alignment_std140(field: &UniformField) -> usize {
    match &field.ty {
        syn::Type::Path(path) => {
            let type_name = &path.path.segments.last().unwrap().ident;
            get_type_alignment_std140(type_name)
        },
        _ => { panic!("Field {} did not have type Path", &field.ident); }
    }
}

/// Get alignment for an std140 type, including std140 alignment
fn get_type_alignment_std140(ident: &Ident) -> usize {
    match ident.to_string().as_str() {
        "float"   => 4,
        "vec2"    => 8,
        "vec3"    => 16,
        "vec4"    => 16,
        "int"     => 4,
        "ivec2"   => 8,
        "ivec3"   => 16,
        "ivec4"   => 16,
        "uint"    => 4,
        "uvec2"   => 8,
        "uvec3"   => 16,
        "uvec4"   => 16,
        "boolean" => 4,
        "bvec2"   => 8,
        "bvec3"   => 16,
        "bvec4"   => 16,
        "mat2x2"  => 8,
        "mat2x3"  => 16,
        "mat2x4"  => 16,
        "mat3x2"  => 8,
        "mat3x3"  => 16,
        "mat3x4"  => 16,
        "mat4x2"  => 8,
        "mat4x3"  => 16,
        "mat4x4"  => 16,
        _ => panic!("Attempted to get type alignment for unknown type {}", ident)
    }
}

/// Align to byte alignment
fn align(offset: usize, alignment: usize) -> usize {
    if offset % alignment == 0 {
        offset
    }
    else {
        offset + alignment - (offset % alignment)
    }
}
