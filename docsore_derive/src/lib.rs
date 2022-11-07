use darling::FromDeriveInput;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput, Ident};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(docsore), supports(struct_named))]
struct Opts {
    collection: Option<String>,
    #[darling(default)]
    index: Option<String>,
}

#[proc_macro_derive(Document, attributes(docsore))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput { ident, .. } = input;
    let collection = match opts.collection {
        Some(x) => quote! {
            fn collection() -> &'static str {
                #x
            }
        },
        None => quote! {
            fn collection() -> &'static str {
                stringify!(#ident)
            }
        },
    };

    let indexes = {
        // let mut indexes = vec![];
        // for idx in opts.indexes {
        //     indexes.push(quote! {#idx.as_bytes().to_vec()});
        // }
        dbg!(&opts.index);
        let idxs = opts
            .index
            .as_ref()
            .map(|s| s.clone())
            .map(|s| {
                s.split(",")
                    .into_iter()
                    .map(|s| s.to_owned())
                    .collect::<Vec<String>>()
            })
            .unwrap_or(vec![])
            .into_iter()
            .map(|s| syn::Ident::new(&s.trim().to_owned(), Span::call_site()))
            .collect::<Vec<Ident>>();
        quote! {
            fn indexes(&self) -> Vec<adapter::HashIndex> {
                vec![#( self.#idxs.as_bytes().to_vec(), )*]
            }
        }
    };
    let gen = quote! {
        impl Document for #ident {
            #collection

            fn id(&self) -> Vec<u8> {
                self.id.as_bytes().to_vec()
            }

            #indexes
        }
    };
    gen.into()
}
