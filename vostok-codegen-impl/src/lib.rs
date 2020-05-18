extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use quote::{quote, quote_spanned};
use syn::{
    export::TokenStream2, parse::Parser, parse_macro_input, punctuated::Punctuated,
    spanned::Spanned, token::Comma, AttributeArgs, ItemFn, NestedMeta, Path,
};

const COMMAND_PREFIX: &'static str = "__vostok_request_handler_";
const ROUTE_ID_PREFIX: &'static str = "__vostok_route_id_";

#[proc_macro_attribute]
pub fn request(macro_args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(macro_args as AttributeArgs);

    let route = match args.get(0) {
        Some(NestedMeta::Lit(lit)) => quote! { #lit },
        _ => panic!(),
    };

    let item = parse_macro_input!(item as ItemFn);
    let fn_name = &item.sig.ident;
    let name = syn::Ident::new(
        &format!("{}{}", COMMAND_PREFIX, &item.sig.ident),
        item.sig.ident.span(),
    );

    let route_id = syn::Ident::new(
        &format!("{}{}", ROUTE_ID_PREFIX, &item.sig.ident),
        item.sig.ident.span(),
    );

    let args: Vec<_> = item
        .sig
        .inputs
        .iter()
        .map(|input| {
            let span = input.span();
            quote_spanned!(span=> vostok_core::request::FromBundle::from_bundle(request)?)
        })
        .collect();

    let (span, ty) = match &item.sig.output {
        syn::ReturnType::Default => (item.sig.output.span(), quote! {()}),
        syn::ReturnType::Type(_, ty) => (ty.span(), quote! {#ty}),
    };

    let function_call = if let Some(_) = item.sig.asyncness {
        quote_spanned! {span=> {
            let fut = #fn_name(#(#args),*);
            async {
                let val = fut.await;
                val
            }
        }}
    } else {
        quote_spanned! {span => {
            let res = <#ty as Into<Res>>::into(#fn_name(#(#args),*));
            async move {res}
        }}
    };

    let result = quote! {
        #[allow(non_upper_case_globals)]
        pub const #route_id: &'static str = #route;

        #[allow(non_camel_case_types)]
        pub struct #name;

        impl #name {
            fn __route_id() -> &'static str {
                #route
            }

            fn __base_handle<'a, 'b, Req, Res: From<#ty> + Send + 'static>(
                request: &'a vostok_core::request::Bundle<'b, Req>,
            ) -> std::pin::Pin<Box<std::future::Future<Output = Res> + Send + 'a>> {
                use std::pin::Pin;

                let fut = #function_call;
                Box::pin(fut)
            }
        }

        #item
    };

    result.into()
}

fn prefix_last_segment(prefix: &'static str, path: &mut Path) {
    let mut last_seg = path.segments.last_mut().expect("syn::Path has segments");
    last_seg.ident = syn::Ident::new(
        &format!("{}{}", prefix, &last_seg.ident),
        last_seg.ident.span(),
    )
}

fn _prefixed_vec(input: TokenStream) -> Result<TokenStream2, syn::Error> {
    // Parse a comma-separated list of paths.
    let mut paths = <Punctuated<Path, Comma>>::parse_terminated.parse(input)?;
    let mut routes = paths.clone();

    // Prefix the last segment in each path with `prefix`.
    paths
        .iter_mut()
        .for_each(|p| prefix_last_segment(COMMAND_PREFIX, p));

    routes
        .iter_mut()
        .for_each(|p| prefix_last_segment(ROUTE_ID_PREFIX, p));

    // Return a `vec!` of the prefixed, mapped paths.
    let prefixed_mapped_paths = paths.iter().zip(routes).map(
        |(path, route)| quote_spanned!(path.span().into() => &(#route, &#path::__base_handle)),
    );

    Ok(quote!(vec![#(#prefixed_mapped_paths),*]))
}

fn prefixed_vec(input: TokenStream) -> TokenStream {
    let vec = match _prefixed_vec(input) {
        Ok(vec) => vec,
        Err(err) => return err.to_compile_error().into(),
    };

    quote!({
        let __vector: Vec<&'static dyn vostok_core::handler::RequestHandler<_, _>> = #vec;
        __vector
    })
    .into()
}

#[proc_macro_hack]
pub fn routes(input: TokenStream) -> TokenStream {
    prefixed_vec(input)
}
