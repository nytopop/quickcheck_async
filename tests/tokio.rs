// Copyright 2020 nytopop (Eric Izoita)
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
#![warn(rust_2018_idioms)]

#[quickcheck_async::tokio]
async fn bool_test() -> bool {
    true
}

#[quickcheck_async::tokio(max_threads = 4)]
async fn max_threads() {}

#[quickcheck_async::tokio(core_threads = 4)]
async fn core_threads() {}

#[quickcheck_async::tokio(core_threads = 3, max_threads = 5)]
async fn all_args() {}
