use quote::{self};
use syn::{parse_macro_input,DeriveInput,Data,Fields,Type, FieldsNamed, spanned::Spanned, FieldsUnnamed, DataEnum, token::{Token, Comma}, punctuated::Punctuated, Variant};

use proc_macro2::{TokenStream, Ident};

#[proc_macro_derive(MTGLoggable)]
pub fn derive_mtg_log(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast = parse_macro_input!(input as DeriveInput);
    let pass_ast=ast.clone();
    let name=quote::format_ident!("Log{}", ast.ident);
    let vis=ast.vis;
    let generics: syn::Generics=ast.generics;
    // Build the impl
    let mut res = match ast.data.clone(){
        Data::Struct(data) => {
            let needs_semicolon=match &data.fields {
                Fields::Named(_)=>false,
                _ => true
            };
            let fields = modify_fields(data.fields);
            if needs_semicolon{
                quote::quote!{
                    #vis struct #name #generics #fields ;
                }
            } else {
                quote::quote!{
                    #vis struct #name #generics #fields
                }
            }
        },
        Data::Enum(data) => {
            let fields=modify_fields_enum(data);
            quote::quote!{
                #vis enum #name #generics #fields
            }
        }
        _=> todo!("This data type isn't implemented yet")
    };
    let implement=impl_mtglog(&name, &pass_ast);
    res.extend(implement.into_iter());
    res.into()
}

fn impl_mtglog(name: &Ident, ast:&DeriveInput) -> TokenStream{
    let generics: &syn::Generics=&ast.generics;
    let orig_name=&ast.ident;
    let body=match ast.data.clone(){
        Data::Struct(data) => {
            impl_for_fields(data.fields)
        },
        Data::Enum(data)=>{
            impl_for_enum(data.variants)
        }
        _=>unimplemented!()
    };
    quote::quote!{
        impl #generics MTGLog for #orig_name #generics{
            type LogType = #name;
            fn log(&self, game_context: &GameContext) -> Self::LogType{
                #body
            }
        }
    }
}
fn impl_for_enum(variants:Punctuated<Variant, Comma>) -> TokenStream{
    let mut new_code=Vec::new();
    for variant in variants{
        let ident=variant.ident;
        let inner=impl_for_fields_inner(variant.fields);
        new_code.push(quote::quote!(
            Self::#ident => {
                Self::LogType:: #ident #inner
            }
        ))
    }
    quote::quote!(
        match self{
            #(
                #new_code , 
            )*
        }
    )
}
fn impl_for_fields(fields:Fields) -> TokenStream{
    let inner=impl_for_fields_inner(fields);
    quote::quote!(Self::LogType #inner)
}
fn impl_for_fields_inner(fields:Fields) -> TokenStream{
    match fields{
        Fields::Unit => {
            quote::quote!( () )
        },
        Fields::Unnamed(fields) => {
            impl_for_fields_unnamed(fields)
        },
        Fields::Named(fields) => {
            impl_for_fields_named(fields)
        }
        _=>unimplemented!()
    }
}
fn impl_for_fields_named(fields: FieldsNamed) -> TokenStream{
    let mut new_code=Vec::new();
    for field in fields.named{
        let ty=field.ty.clone();
        let name = field.ident.unwrap();
        new_code.push(quote::quote_spanned!{ field.ty.span()=>
            #name : <#ty as MTGLog>::log(game_context)
        })
    }
    quote::quote!(
        {
            #( #new_code , )*
        }
    )
}
fn impl_for_fields_unnamed(fields: FieldsUnnamed) -> TokenStream{
    let mut new_code=Vec::new();
    for field in fields.unnamed{
        let ty=field.ty.clone();
        new_code.push(quote::quote_spanned!{ field.ty.span()=>
            <#ty as MTGLog>::log(game_context)
        })
    }
    quote::quote!(
        (
            #( #new_code , )*
        )
    )
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
    quote::quote!( #fields )
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