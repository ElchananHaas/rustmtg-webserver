use quote;
use syn::{parse_macro_input,DeriveInput,Data,Fields,Type, FieldsNamed, spanned::Spanned, FieldsUnnamed, DataEnum, DataStruct, Field};

use proc_macro2::TokenStream;

#[proc_macro_derive(MTGLoggable)]
pub fn derive_mtg_log(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast = parse_macro_input!(input as DeriveInput);
    let name=quote::format_ident!("Log{}", ast.ident);
    let generics: syn::Generics=ast.generics;
    // Build the impl
    let new_struct_contents = match ast.data.clone(){
        Data::Struct(data) => {
            let fields = modify_fields(data.fields);
            quote::quote!{
                struct #name #generics #fields ;
            }
        },
        Data::Enum(data) => {
            let fields=modify_fields_enum(data);
            quote::quote!{
                enum #name #generics #fields ;
            }
        }
        _=> todo!("This data type isn't implemented yet")
    };
    //panic!("{:?}",res);
    new_struct_contents.into()
}

fn modify_fields(fields:Fields) -> TokenStream{
    match fields {
        Fields::Named(fields)=>{
            modify_fields_named(fields)
        }
        Fields::Unnamed(fields)=>{
            modify_fields_unnamed(fields)
        }
        Fields::Unit => { 
            let field=Fields::Unit;
            quote::quote!( #field )
        }
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
    quote::quote!( #fields )
}
fn modify_fields_enum(fields: DataEnum) -> TokenStream{
    let mut variants = Vec::new();
    for field in fields.variants{
        let ident=field.ident;
        let fields=modify_fields(field.fields);
        variants.push(quote::quote!(
            #ident #fields
        ))
    }
    let res=quote::quote!(  { #( #variants , )* } );
    res
}