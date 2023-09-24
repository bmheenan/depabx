use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemTrait};

#[proc_macro_attribute]
pub fn wrap(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemTrait);
    let trait_ident = &input.ident;
    let struct_ident: Ident = parse_macro_input!(attr as Ident);
    let trait_items = &input.items;
    let mut trait_impls = Vec::new();
    let generics = &input.generics;

    for trait_item in trait_items {
        if let syn::TraitItem::Method(method) = trait_item {
            let method_name = &method.sig.ident;
            let method_str = method_name.to_string();
            if !method_str.starts_with("abx_") || method_str.len() <= 4 {
                panic!("Every method name must begin with \"abx_\"");
            }
            let method_name_short = Ident::new(&method_str[4..], Span::call_site());
            let method_inputs = &method.sig.inputs;
            let return_type = &method.sig.output;

            let mut param_names = Vec::new();
            let mut param_types = Vec::new();
            for param in method_inputs {
                if let syn::FnArg::Typed(arg) = param {
                    if let syn::Pat::Ident(ident) = &*arg.pat {
                        let param_name = &ident.ident;
                        let param_type = &arg.ty;
                        param_names.push(quote! { #param_name });
                        param_types.push(quote! { #param_type });
                    }
                }
            }

            let mutability = match &method.sig.receiver() {
                Some(syn::FnArg::Receiver(syn::Receiver { mutability, .. })) => {
                    if mutability.is_some() {
                        quote! { &mut self }
                    } else {
                        quote! { &self }
                    }
                }
                _ => panic!("All functions must be methods with receivers"),
            };

            let generics = &method.sig.generics;

            let trait_impl = quote! {
                fn #method_name #generics(#mutability, #(#param_names: #param_types),*) #return_type {
                    #struct_ident::#method_name_short(self, #(#param_names),*)
                }
            };
            trait_impls.push(trait_impl);
        }
    }

    let gen = quote! {
        trait #trait_ident #generics {
            #(#trait_items)*
        }
        impl #generics #trait_ident for #struct_ident #generics {
            #(#trait_impls)*
        }
    };

    gen.into()
}
