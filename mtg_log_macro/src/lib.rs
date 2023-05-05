use quote;
use syn::{parse_macro_input,DeriveInput,Data,Fields,Type, FieldsNamed, spanned::Spanned, FieldsUnnamed};

use proc_macro2::TokenStream;

#[proc_macro_derive(MTGLoggable)]
pub fn derive_mtg_log(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast = parse_macro_input!(input as DeriveInput);
    let name=quote::format_ident!("Log{}", ast.ident);
    // Build the impl
    let new_struct_contents = log_struct(&ast.data);
    let generics: syn::Generics=ast.generics;
    let res=quote::quote!{
        struct #name #generics #new_struct_contents 
    };
    //panic!("{:?}",res);
    res.into()
}

fn log_struct(data: &Data) -> TokenStream{
    let data=data.clone();
    match data{
        Data::Struct(data) => {
            match data.fields {
                Fields::Named(fields)=>{
                    modify_fields_named(fields)
                }
                Fields::Unnamed(fields)=>{
                    modify_fields_unnamed(fields)
                }
                Fields::Unit => { 
                    let field=Fields::Unit;
                    quote::quote!( #field ;)
                }
            }
        }
        _=> todo!("This data type isn't implemented yet")
    }
}
fn modify_fields_named(mut fields: FieldsNamed) -> TokenStream{
    for field in &mut fields.named{
        let ty=field.ty.clone();
        field.ty=Type::Verbatim(quote::quote_spanned!{ field.ty.span()=>
            <#ty as MTGLog>::LogType
        });
    }
    quote::quote!( #fields)
}
fn modify_fields_unnamed(mut fields: FieldsUnnamed) -> TokenStream{
    for field in &mut fields.unnamed{
        let ty=field.ty.clone();
        field.ty=Type::Verbatim(quote::quote_spanned!{ field.ty.span()=>
            <#ty as MTGLog>::LogType
        });
    }
    quote::quote!( #fields ;)
}