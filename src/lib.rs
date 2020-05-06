// Copyright 2020 nytopop (Eric Izoita)
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//! Runtime-agnostic attribute macros to use quickcheck with async tests.
#![warn(rust_2018_idioms, missing_docs)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, AttributeArgs, Error, FnArg, ItemFn,
    NestedMeta, Pat, Type,
};

struct Arguments {
    ids: Punctuated<Pat, Comma>,
    tys: Punctuated<Type, Comma>,
}

fn parse_args(fn_item: &ItemFn) -> Result<Arguments, TokenStream> {
    let mut args = Arguments {
        ids: Punctuated::new(),
        tys: Punctuated::new(),
    };

    for pt in fn_item.sig.inputs.iter() {
        match pt {
            FnArg::Receiver(_) => {
                return Err(
                    Error::new_spanned(&fn_item, "test fn cannot take a receiver")
                        .to_compile_error()
                        .into(),
                )
            }

            FnArg::Typed(pt) => {
                args.ids.push(*pt.pat.clone());
                args.tys.push(*pt.ty.clone());
            }
        }
    }

    Ok(args)
}

/// Mark an async function to be fuzz-tested using [quickcheck][qc], within a tokio
/// executor.
///
/// # Usage
///
/// ```
/// #[quickcheck_async::tokio]
/// async fn fuzz_me(fuzz_arg: String) -> bool {
///     fuzz_arg != "fuzzed".to_owned()
/// }
/// ```
///
/// # Attribute arguments
///
/// Arguments to this attribute are passed through to [tokio::test][tt].
///
/// ```
/// #[quickcheck_async::tokio(core_threads = 3)]
/// async fn fuzz_me(fuzz_arg: String) -> bool {
///     fuzz_arg != "fuzzed".to_owned()
/// }
/// ```
/// [qc]: https://docs.rs/quickcheck/latest/quickcheck/fn.quickcheck.html
/// [tt]: https://docs.rs/tokio/latest/tokio/attr.test.html
#[proc_macro_attribute]
pub fn tokio(args: TokenStream, item: TokenStream) -> TokenStream {
    let fn_item = parse_macro_input!(item as ItemFn);

    for attr in &fn_item.attrs {
        if attr.path.is_ident("test") {
            return Error::new_spanned(&fn_item, "multiple #[test] attributes were supplied")
                .to_compile_error()
                .into();
        }
    }

    if fn_item.sig.asyncness.is_none() {
        return Error::new_spanned(&fn_item, "test fn must be async")
            .to_compile_error()
            .into();
    }

    let p_args = parse_macro_input!(args as AttributeArgs);
    let attrib: Punctuated<NestedMeta, Comma> = p_args.into_iter().collect();

    let call_by = format_ident!("{}", fn_item.sig.ident);

    let Arguments { ids, tys } = match parse_args(&fn_item) {
        Err(e) => return e,
        Ok(ts) => ts,
    };

    let ret = &fn_item.sig.output;

    quote! (
        #[::tokio::test(#attrib)]
        async fn #call_by() {
            #fn_item

            let test_fn: fn(#tys) #ret = |#ids| {
                ::futures::executor::block_on(#call_by(#ids))
            };

            ::tokio::task::spawn_blocking(move || {
                ::quickcheck::quickcheck(test_fn)
            })
            .await
            .unwrap()
        }
    )
    .into()
}

/// Mark an async function to be fuzz-tested using [quickcheck][qc], within an async_std
/// executor.
///
/// # Usage
///
/// ```
/// #[quickcheck_async::async_std]
/// async fn fuzz_me(fuzz_arg: String) -> bool {
///     fuzz_arg != "fuzzed".to_owned()
/// }
/// ```
/// [qc]: https://docs.rs/quickcheck/latest/quickcheck/fn.quickcheck.html
#[proc_macro_attribute]
pub fn async_std(args: TokenStream, item: TokenStream) -> TokenStream {
    let fn_item = parse_macro_input!(item as ItemFn);

    for attr in &fn_item.attrs {
        if attr.path.is_ident("test") {
            return Error::new_spanned(&fn_item, "multiple #[test] attributes were supplied")
                .to_compile_error()
                .into();
        }
    }

    if fn_item.sig.asyncness.is_none() {
        return Error::new_spanned(&fn_item, "test fn must be async")
            .to_compile_error()
            .into();
    }

    let p_args = parse_macro_input!(args as AttributeArgs);
    let attrib: Punctuated<NestedMeta, Comma> = p_args.into_iter().collect();

    let call_by = format_ident!("{}", fn_item.sig.ident);

    let Arguments { ids, tys } = match parse_args(&fn_item) {
        Err(e) => return e,
        Ok(ts) => ts,
    };

    let ret = &fn_item.sig.output;

    quote! (
        #[::async_std::test(#attrib)]
        async fn #call_by() {
            #fn_item

            let test_fn: fn(#tys) #ret = |#ids| {
                ::futures::executor::block_on(#call_by(#ids))
            };

            ::quickcheck::quickcheck(test_fn);
        }
    )
    .into()
}
