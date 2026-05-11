# Upstream Meilisearch WASI Feasibility Blockers

- Date: 2026-05-11T16:04:10Z
- Requested upstream tag: `v1.43.0`
- Git exact-match tag: latest
- Upstream commit: `475ed56e5612df0dbb826748add5f93e0e7d5500`
- Checked target: `wasm32-wasip2`
- Checked packages: `flatten-serde-json filter-parser milli meilisearch-types routes meilisearch`
- Overall status: `1`

## Summary

| Package | Result | Exit | Log |
|---|---:|---:|---|
| `flatten-serde-json` | PASS | `0` | `docs/upstream-wasi-check-flatten-serde-json.log` |
| `filter-parser` | PASS | `0` | `docs/upstream-wasi-check-filter-parser.log` |
| `milli` | FAIL | `101` | `docs/upstream-wasi-check-milli.log` |
| `meilisearch-types` | FAIL | `101` | `docs/upstream-wasi-check-meilisearch-types.log` |
| `routes` | FAIL | `101` | `docs/upstream-wasi-check-routes.log` |
| `meilisearch` | FAIL | `101` | `docs/upstream-wasi-check-meilisearch.log` |

## Interpretation

This layered check identifies the highest upstream crates that compile for Spin's `wasm32-wasip2` target before native-runtime dependencies fail. Failures are expected for the full Meilisearch-on-Spin path because upstream Meilisearch depends on native HTTP/runtime crates, crypto/C dependencies, LMDB/heed, memory-mapped storage, filesystem behavior, and background task scheduling.


## `flatten-serde-json` last 80 log lines

```text
   Compiling serde_core v1.0.228
   Compiling serde_json v1.0.145
    Checking memchr v2.7.6
    Checking flatten-serde-json v1.43.0 (/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/crates/flatten-serde-json)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.42s
```

## `filter-parser` last 80 log lines

```text
   Compiling fst v0.4.7
    Checking minimal-lexical v0.2.1
    Checking memchr v2.7.6
    Checking bytecount v0.6.9
   Compiling syn v2.0.111
    Checking nom v7.1.3
    Checking levenshtein_automata v0.2.1
    Checking nom_locate v4.2.0
   Compiling thiserror-impl v2.0.17
    Checking thiserror v2.0.17
    Checking unescaper v0.1.6
    Checking filter-parser v1.43.0 (/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/crates/filter-parser)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.95s
```

## `milli` last 80 log lines

```text
   Compiling serde_derive v1.0.228
   Compiling zerocopy-derive v0.8.31
   Compiling bytemuck_derive v1.10.2
   Compiling zerofrom-derive v0.1.6
   Compiling displaydoc v0.2.5
   Compiling yoke-derive v0.8.1
   Compiling zerovec-derive v0.11.2
   Compiling phf_macros v0.11.3
   Compiling toml_parser v1.0.4
   Compiling ring v0.17.14
warning: ring@0.17.14: error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-wasip2"'
warning: ring@0.17.14: 1 error generated.
error: failed to run custom build command for `ring v0.17.14`

Caused by:
  process didn't exit successfully: `/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/debug/build/ring-f95cb4875cd5e874/build-script-build` (exit status: 1)
  --- stdout
  cargo:rerun-if-env-changed=CARGO_MANIFEST_DIR
  cargo:rerun-if-env-changed=CARGO_PKG_NAME
  cargo:rerun-if-env-changed=CARGO_PKG_VERSION_MAJOR
  cargo:rerun-if-env-changed=CARGO_PKG_VERSION_MINOR
  cargo:rerun-if-env-changed=CARGO_PKG_VERSION_PATCH
  cargo:rerun-if-env-changed=CARGO_PKG_VERSION_PRE
  cargo:rerun-if-env-changed=CARGO_MANIFEST_LINKS
  cargo:rerun-if-env-changed=RING_PREGENERATE_ASM
  cargo:rerun-if-env-changed=OUT_DIR
  cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH
  cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS
  cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ENV
  cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ENDIAN
  OPT_LEVEL = Some(0)
  OUT_DIR = Some(/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/wasm32-wasip2/debug/build/ring-a50e5148d1e3508e/out)
  TARGET = Some(wasm32-wasip2)
  CARGO_ENCODED_RUSTFLAGS = Some()
  HOST = Some(aarch64-apple-darwin)
  cargo:rerun-if-env-changed=CC_wasm32-wasip2
  CC_wasm32-wasip2 = None
  cargo:rerun-if-env-changed=CC_wasm32_wasip2
  CC_wasm32_wasip2 = None
  cargo:rerun-if-env-changed=TARGET_CC
  TARGET_CC = None
  cargo:rerun-if-env-changed=CC
  CC = None
  cargo:rerun-if-env-changed=WASI_SDK_PATH
  WASI_SDK_PATH = None
  cargo:rerun-if-env-changed=CC_ENABLE_DEBUG_OUTPUT
  RUSTC_WRAPPER = None
  cargo:rerun-if-env-changed=CRATE_CC_NO_DEFAULTS
  CRATE_CC_NO_DEFAULTS = None
  cargo:rerun-if-env-changed=WASI_SYSROOT
  WASI_SYSROOT = None
  DEBUG = Some(true)
  cargo:rerun-if-env-changed=CFLAGS
  CFLAGS = None
  cargo:rerun-if-env-changed=TARGET_CFLAGS
  TARGET_CFLAGS = None
  cargo:rerun-if-env-changed=CFLAGS_wasm32_wasip2
  CFLAGS_wasm32_wasip2 = None
  cargo:rerun-if-env-changed=CFLAGS_wasm32-wasip2
  CFLAGS_wasm32-wasip2 = None
  cargo:warning=error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-wasip2"'
  cargo:warning=1 error generated.

  --- stderr


  error occurred in cc-rs: command did not execute successfully (status code exit status: 1): LC_ALL="C" "clang" "-O0" "-ffunction-sections" "-fdata-sections" "-fno-exceptions" "-g" "-fno-omit-frame-pointer" "--target=wasm32-wasip2" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/include" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/pregenerated" "-Wall" "-Wextra" "-fvisibility=hidden" "-std=c1x" "-Wall" "-Wbad-function-cast" "-Wcast-align" "-Wcast-qual" "-Wconversion" "-Wmissing-field-initializers" "-Wmissing-include-dirs" "-Wnested-externs" "-Wredundant-decls" "-Wshadow" "-Wsign-compare" "-Wsign-conversion" "-Wstrict-prototypes" "-Wundef" "-Wuninitialized" "-g3" "-nostdlibinc" "-DNDEBUG" "-DRING_CORE_NOSTDLIBINC=1" "-o" "/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/wasm32-wasip2/debug/build/ring-a50e5148d1e3508e/out/25ac62e5b3c53843-curve25519.o" "-c" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/crypto/curve25519/curve25519.c"


warning: build failed, waiting for other jobs to finish...
```

## `meilisearch-types` last 80 log lines

```text
  --- stdout
  cargo:rerun-if-env-changed=CARGO_MANIFEST_DIR
  cargo:rerun-if-env-changed=CARGO_PKG_NAME
  cargo:rerun-if-env-changed=CARGO_PKG_VERSION_MAJOR
  cargo:rerun-if-env-changed=CARGO_PKG_VERSION_MINOR
  cargo:rerun-if-env-changed=CARGO_PKG_VERSION_PATCH
  cargo:rerun-if-env-changed=CARGO_PKG_VERSION_PRE
  cargo:rerun-if-env-changed=CARGO_MANIFEST_LINKS
  cargo:rerun-if-env-changed=RING_PREGENERATE_ASM
  cargo:rerun-if-env-changed=OUT_DIR
  cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH
  cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS
  cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ENV
  cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ENDIAN
  OPT_LEVEL = Some(0)
  OUT_DIR = Some(/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/wasm32-wasip2/debug/build/ring-a50e5148d1e3508e/out)
  TARGET = Some(wasm32-wasip2)
  CARGO_ENCODED_RUSTFLAGS = Some()
  HOST = Some(aarch64-apple-darwin)
  cargo:rerun-if-env-changed=CC_wasm32-wasip2
  CC_wasm32-wasip2 = None
  cargo:rerun-if-env-changed=CC_wasm32_wasip2
  CC_wasm32_wasip2 = None
  cargo:rerun-if-env-changed=TARGET_CC
  TARGET_CC = None
  cargo:rerun-if-env-changed=CC
  CC = None
  cargo:rerun-if-env-changed=WASI_SDK_PATH
  WASI_SDK_PATH = None
  cargo:rerun-if-env-changed=CC_ENABLE_DEBUG_OUTPUT
  RUSTC_WRAPPER = None
  cargo:rerun-if-env-changed=CRATE_CC_NO_DEFAULTS
  CRATE_CC_NO_DEFAULTS = None
  cargo:rerun-if-env-changed=WASI_SYSROOT
  WASI_SYSROOT = None
  DEBUG = Some(true)
  cargo:rerun-if-env-changed=CFLAGS
  CFLAGS = None
  cargo:rerun-if-env-changed=TARGET_CFLAGS
  TARGET_CFLAGS = None
  cargo:rerun-if-env-changed=CFLAGS_wasm32_wasip2
  CFLAGS_wasm32_wasip2 = None
  cargo:rerun-if-env-changed=CFLAGS_wasm32-wasip2
  CFLAGS_wasm32-wasip2 = None
  cargo:warning=error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-wasip2"'
  cargo:warning=1 error generated.

  --- stderr


  error occurred in cc-rs: command did not execute successfully (status code exit status: 1): LC_ALL="C" "clang" "-O0" "-ffunction-sections" "-fdata-sections" "-fno-exceptions" "-g" "-fno-omit-frame-pointer" "--target=wasm32-wasip2" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/include" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/pregenerated" "-Wall" "-Wextra" "-fvisibility=hidden" "-std=c1x" "-Wall" "-Wbad-function-cast" "-Wcast-align" "-Wcast-qual" "-Wconversion" "-Wmissing-field-initializers" "-Wmissing-include-dirs" "-Wnested-externs" "-Wredundant-decls" "-Wshadow" "-Wsign-compare" "-Wsign-conversion" "-Wstrict-prototypes" "-Wundef" "-Wuninitialized" "-g3" "-nostdlibinc" "-DNDEBUG" "-DRING_CORE_NOSTDLIBINC=1" "-o" "/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/wasm32-wasip2/debug/build/ring-a50e5148d1e3508e/out/25ac62e5b3c53843-curve25519.o" "-c" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/crypto/curve25519/curve25519.c"


warning: build failed, waiting for other jobs to finish...
error[E0658]: use of unstable library feature `wasip2`
   --> /Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/net/tcp/listener.rs:295:37
    |
295 |             use std::os::wasi::io::{FromRawFd, IntoRawFd};
    |                                     ^^^^^^^^^

error[E0658]: use of unstable library feature `wasip2`
   --> /Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/net/tcp/listener.rs:295:48
    |
295 |             use std::os::wasi::io::{FromRawFd, IntoRawFd};
    |                                                ^^^^^^^^^

error[E0658]: use of unstable library feature `wasip2`
   --> /Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/net/tcp/stream.rs:275:37
    |
275 |             use std::os::wasi::io::{FromRawFd, IntoRawFd};
    |                                     ^^^^^^^^^

error[E0658]: use of unstable library feature `wasip2`
   --> /Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/net/tcp/stream.rs:275:48
    |
275 |             use std::os::wasi::io::{FromRawFd, IntoRawFd};
    |                                                ^^^^^^^^^

For more information about this error, try `rustc --explain E0658`.
error: could not compile `tokio` (lib) due to 5 previous errors
```

## `routes` last 80 log lines

```text
   Compiling serde_core v1.0.228
    Checking smallvec v1.15.1
    Checking stable_deref_trait v1.2.1
   Compiling serde v1.0.228
    Checking once_cell v1.21.3
   Compiling memchr v2.7.6
    Checking zerofrom v0.1.6
   Compiling regex-syntax v0.8.8
   Compiling rustls v0.23.36
    Checking untrusted v0.9.0
    Checking alloc-no-stdlib v2.0.4
    Checking rustls-webpki v0.103.13
    Checking yoke v0.8.1
    Checking futures-task v0.3.31
    Checking tracing-core v0.1.35
    Checking parking_lot_core v0.9.12
   Compiling unicode-segmentation v1.12.0
    Checking zerovec v0.11.5
    Checking parking_lot v0.12.5
    Checking zerotrie v0.2.3
    Checking tracing v0.1.43
    Checking tokio v1.48.0
error: Only features sync,macros,io-util,rt,time are supported on wasm.
   --> /Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/lib.rs:481:1
    |
481 | compile_error!("Only features sync,macros,io-util,rt,time are supported on wasm.");
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

   Compiling aho-corasick v1.1.4
    Checking subtle v2.6.1
    Checking fnv v1.0.7
    Checking hashbrown v0.16.1
    Checking local-waker v0.1.4
    Checking actix-utils v3.0.1
    Checking tinystr v0.8.2
    Checking potential_utf v0.1.4
    Checking icu_collections v2.1.1
    Checking icu_locale_core v2.1.1
error[E0658]: use of unstable library feature `wasip2`
   --> /Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/net/tcp/listener.rs:295:37
    |
295 |             use std::os::wasi::io::{FromRawFd, IntoRawFd};
    |                                     ^^^^^^^^^

error[E0658]: use of unstable library feature `wasip2`
   --> /Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/net/tcp/listener.rs:295:48
    |
295 |             use std::os::wasi::io::{FromRawFd, IntoRawFd};
    |                                                ^^^^^^^^^

error[E0658]: use of unstable library feature `wasip2`
   --> /Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/net/tcp/stream.rs:275:37
    |
275 |             use std::os::wasi::io::{FromRawFd, IntoRawFd};
    |                                     ^^^^^^^^^

error[E0658]: use of unstable library feature `wasip2`
   --> /Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/net/tcp/stream.rs:275:48
    |
275 |             use std::os::wasi::io::{FromRawFd, IntoRawFd};
    |                                                ^^^^^^^^^

    Checking http v0.2.12
    Checking futures-util v0.3.31
   Compiling convert_case v0.10.0
    Checking icu_provider v2.1.1
    Checking alloc-stdlib v0.2.2
    Checking actix-service v2.0.3
   Compiling pin-project-lite v0.2.16
   Compiling httparse v1.10.1
   Compiling regex-automata v0.4.13
    Checking icu_normalizer v2.1.1
    Checking icu_properties v2.1.1
    Checking powerfmt v0.2.0
    Checking adler2 v2.0.1
   Compiling bytes v1.11.1
For more information about this error, try `rustc --explain E0658`.
error: could not compile `tokio` (lib) due to 5 previous errors
warning: build failed, waiting for other jobs to finish...
```

## `meilisearch` last 80 log lines

```text
  HOST = Some(aarch64-apple-darwin)
  cargo:rerun-if-env-changed=CC_wasm32-wasip2
  CC_wasm32-wasip2 = None
  cargo:rerun-if-env-changed=CC_wasm32_wasip2
  CC_wasm32_wasip2 = None
  cargo:rerun-if-env-changed=TARGET_CC
  TARGET_CC = None
  cargo:rerun-if-env-changed=CC
  CC = None
  cargo:rerun-if-env-changed=WASI_SDK_PATH
  WASI_SDK_PATH = None
  cargo:rerun-if-env-changed=CC_ENABLE_DEBUG_OUTPUT
  RUSTC_WRAPPER = None
  cargo:rerun-if-env-changed=CRATE_CC_NO_DEFAULTS
  CRATE_CC_NO_DEFAULTS = None
  cargo:rerun-if-env-changed=WASI_SYSROOT
  WASI_SYSROOT = None
  DEBUG = Some(true)
  cargo:rerun-if-env-changed=CFLAGS
  CFLAGS = None
  cargo:rerun-if-env-changed=TARGET_CFLAGS
  TARGET_CFLAGS = None
  cargo:rerun-if-env-changed=CFLAGS_wasm32_wasip2
  CFLAGS_wasm32_wasip2 = None
  cargo:rerun-if-env-changed=CFLAGS_wasm32-wasip2
  CFLAGS_wasm32-wasip2 = None
  cargo:warning=error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-wasip2"'
  cargo:warning=1 error generated.
  exit status: 1
  cargo:warning=ToolExecError: command did not execute successfully (status code exit status: 1): LC_ALL="C" "clang" "-O0" "-ffunction-sections" "-fdata-sections" "-fno-exceptions" "-g" "-fno-omit-frame-pointer" "--target=wasm32-wasip2" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/include" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/pregenerated" "-Wall" "-Wextra" "-fvisibility=hidden" "-std=c1x" "-Wall" "-Wbad-function-cast" "-Wcast-align" "-Wcast-qual" "-Wconversion" "-Wmissing-field-initializers" "-Wmissing-include-dirs" "-Wnested-externs" "-Wredundant-decls" "-Wshadow" "-Wsign-compare" "-Wsign-conversion" "-Wstrict-prototypes" "-Wundef" "-Wuninitialized" "-g3" "-nostdlibinc" "-DNDEBUG" "-DRING_CORE_NOSTDLIBINC=1" "-o" "/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/wasm32-wasip2/debug/build/ring-d0c6c53b92eada7d/out/25ac62e5b3c53843-curve25519.o" "-c" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/crypto/curve25519/curve25519.c"cargo:warning=error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-wasip2"'
  cargo:warning=1 error generated.

  exit status: 1
  cargo:warning=ToolExecError: command did not execute successfully (status code exit status: 1): LC_ALL="C" "clang" "-O0" "-ffunction-sections" "-fdata-sections" "-fno-exceptions" "-g" "-fno-omit-frame-pointer" "--target=wasm32-wasip2" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/include" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/pregenerated" "-Wall" "-Wextra" "-fvisibility=hidden" "-std=c1x" "-Wall" "-Wbad-function-cast" "-Wcast-align" "-Wcast-qual" "-Wconversion" "-Wmissing-field-initializers" "-Wmissing-include-dirs" "-Wnested-externs" "-Wredundant-decls" "-Wshadow" "-Wsign-compare" "-Wsign-conversion" "-Wstrict-prototypes" "-Wundef" "-Wuninitialized" "-g3" "-nostdlibinc" "-DNDEBUG" "-DRING_CORE_NOSTDLIBINC=1" "-o" "/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/wasm32-wasip2/debug/build/ring-d0c6c53b92eada7d/out/0bbbd18bda93c05b-aes_nohw.o" "-c" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/crypto/fipsmodule/aes/aes_nohw.c"cargo:warning=error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-wasip2"'
  cargo:warning=1 error generated.

  exit status: 1
  cargo:warning=ToolExecError: command did not execute successfully (status code exit status: 1): LC_ALL="C" "clang" "-O0" "-ffunction-sections" "-fdata-sections" "-fno-exceptions" "-g" "-fno-omit-frame-pointer" "--target=wasm32-wasip2" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/include" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/pregenerated" "-Wall" "-Wextra" "-fvisibility=hidden" "-std=c1x" "-Wall" "-Wbad-function-cast" "-Wcast-align" "-Wcast-qual" "-Wconversion" "-Wmissing-field-initializers" "-Wmissing-include-dirs" "-Wnested-externs" "-Wredundant-decls" "-Wshadow" "-Wsign-compare" "-Wsign-conversion" "-Wstrict-prototypes" "-Wundef" "-Wuninitialized" "-g3" "-nostdlibinc" "-DNDEBUG" "-DRING_CORE_NOSTDLIBINC=1" "-o" "/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/wasm32-wasip2/debug/build/ring-d0c6c53b92eada7d/out/00c879ee3285a50d-montgomery.o" "-c" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/crypto/fipsmodule/bn/montgomery.c"cargo:warning=error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-wasip2"'
  cargo:warning=1 error generated.

  exit status: 1
  cargo:warning=ToolExecError: command did not execute successfully (status code exit status: 1): LC_ALL="C" "clang" "-O0" "-ffunction-sections" "-fdata-sections" "-fno-exceptions" "-g" "-fno-omit-frame-pointer" "--target=wasm32-wasip2" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/include" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/pregenerated" "-Wall" "-Wextra" "-fvisibility=hidden" "-std=c1x" "-Wall" "-Wbad-function-cast" "-Wcast-align" "-Wcast-qual" "-Wconversion" "-Wmissing-field-initializers" "-Wmissing-include-dirs" "-Wnested-externs" "-Wredundant-decls" "-Wshadow" "-Wsign-compare" "-Wsign-conversion" "-Wstrict-prototypes" "-Wundef" "-Wuninitialized" "-g3" "-nostdlibinc" "-DNDEBUG" "-DRING_CORE_NOSTDLIBINC=1" "-o" "/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/wasm32-wasip2/debug/build/ring-d0c6c53b92eada7d/out/00c879ee3285a50d-montgomery_inv.o" "-c" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/crypto/fipsmodule/bn/montgomery_inv.c"cargo:warning=error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-wasip2"'
  cargo:warning=1 error generated.

  exit status: 1
  cargo:warning=ToolExecError: command did not execute successfully (status code exit status: 1): LC_ALL="C" "clang" "-O0" "-ffunction-sections" "-fdata-sections" "-fno-exceptions" "-g" "-fno-omit-frame-pointer" "--target=wasm32-wasip2" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/include" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/pregenerated" "-Wall" "-Wextra" "-fvisibility=hidden" "-std=c1x" "-Wall" "-Wbad-function-cast" "-Wcast-align" "-Wcast-qual" "-Wconversion" "-Wmissing-field-initializers" "-Wmissing-include-dirs" "-Wnested-externs" "-Wredundant-decls" "-Wshadow" "-Wsign-compare" "-Wsign-conversion" "-Wstrict-prototypes" "-Wundef" "-Wuninitialized" "-g3" "-nostdlibinc" "-DNDEBUG" "-DRING_CORE_NOSTDLIBINC=1" "-o" "/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/wasm32-wasip2/debug/build/ring-d0c6c53b92eada7d/out/a0330e891e733f4e-ecp_nistz.o" "-c" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/crypto/fipsmodule/ec/ecp_nistz.c"cargo:warning=error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-wasip2"'
  cargo:warning=1 error generated.

  exit status: 1
  cargo:warning=ToolExecError: command did not execute successfully (status code exit status: 1): LC_ALL="C" "clang" "-O0" "-ffunction-sections" "-fdata-sections" "-fno-exceptions" "-g" "-fno-omit-frame-pointer" "--target=wasm32-wasip2" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/include" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/pregenerated" "-Wall" "-Wextra" "-fvisibility=hidden" "-std=c1x" "-Wall" "-Wbad-function-cast" "-Wcast-align" "-Wcast-qual" "-Wconversion" "-Wmissing-field-initializers" "-Wmissing-include-dirs" "-Wnested-externs" "-Wredundant-decls" "-Wshadow" "-Wsign-compare" "-Wsign-conversion" "-Wstrict-prototypes" "-Wundef" "-Wuninitialized" "-g3" "-nostdlibinc" "-DNDEBUG" "-DRING_CORE_NOSTDLIBINC=1" "-o" "/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/wasm32-wasip2/debug/build/ring-d0c6c53b92eada7d/out/a0330e891e733f4e-gfp_p256.o" "-c" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/crypto/fipsmodule/ec/gfp_p256.c"cargo:warning=error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-wasip2"'
  cargo:warning=1 error generated.

  exit status: 1
  cargo:warning=ToolExecError: command did not execute successfully (status code exit status: 1): LC_ALL="C" "clang" "-O0" "-ffunction-sections" "-fdata-sections" "-fno-exceptions" "-g" "-fno-omit-frame-pointer" "--target=wasm32-wasip2" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/include" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/pregenerated" "-Wall" "-Wextra" "-fvisibility=hidden" "-std=c1x" "-Wall" "-Wbad-function-cast" "-Wcast-align" "-Wcast-qual" "-Wconversion" "-Wmissing-field-initializers" "-Wmissing-include-dirs" "-Wnested-externs" "-Wredundant-decls" "-Wshadow" "-Wsign-compare" "-Wsign-conversion" "-Wstrict-prototypes" "-Wundef" "-Wuninitialized" "-g3" "-nostdlibinc" "-DNDEBUG" "-DRING_CORE_NOSTDLIBINC=1" "-o" "/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/wasm32-wasip2/debug/build/ring-d0c6c53b92eada7d/out/a0330e891e733f4e-gfp_p384.o" "-c" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/crypto/fipsmodule/ec/gfp_p384.c"cargo:warning=error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-wasip2"'
  cargo:warning=1 error generated.

  exit status: 1
  cargo:warning=ToolExecError: command did not execute successfully (status code exit status: 1): LC_ALL="C" "clang" "-O0" "-ffunction-sections" "-fdata-sections" "-fno-exceptions" "-g" "-fno-omit-frame-pointer" "--target=wasm32-wasip2" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/include" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/pregenerated" "-Wall" "-Wextra" "-fvisibility=hidden" "-std=c1x" "-Wall" "-Wbad-function-cast" "-Wcast-align" "-Wcast-qual" "-Wconversion" "-Wmissing-field-initializers" "-Wmissing-include-dirs" "-Wnested-externs" "-Wredundant-decls" "-Wshadow" "-Wsign-compare" "-Wsign-conversion" "-Wstrict-prototypes" "-Wundef" "-Wuninitialized" "-g3" "-nostdlibinc" "-DNDEBUG" "-DRING_CORE_NOSTDLIBINC=1" "-o" "/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/wasm32-wasip2/debug/build/ring-d0c6c53b92eada7d/out/a0330e891e733f4e-p256.o" "-c" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/crypto/fipsmodule/ec/p256.c"cargo:warning=error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-wasip2"'
  cargo:warning=1 error generated.

  exit status: 1
  cargo:warning=ToolExecError: command did not execute successfully (status code exit status: 1): LC_ALL="C" "clang" "-O0" "-ffunction-sections" "-fdata-sections" "-fno-exceptions" "-g" "-fno-omit-frame-pointer" "--target=wasm32-wasip2" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/include" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/pregenerated" "-Wall" "-Wextra" "-fvisibility=hidden" "-std=c1x" "-Wall" "-Wbad-function-cast" "-Wcast-align" "-Wcast-qual" "-Wconversion" "-Wmissing-field-initializers" "-Wmissing-include-dirs" "-Wnested-externs" "-Wredundant-decls" "-Wshadow" "-Wsign-compare" "-Wsign-conversion" "-Wstrict-prototypes" "-Wundef" "-Wuninitialized" "-g3" "-nostdlibinc" "-DNDEBUG" "-DRING_CORE_NOSTDLIBINC=1" "-o" "/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/wasm32-wasip2/debug/build/ring-d0c6c53b92eada7d/out/aaa1ba3e455ee2e1-limbs.o" "-c" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/crypto/limbs/limbs.c"cargo:warning=error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-wasip2"'
  cargo:warning=1 error generated.

  exit status: 1
  cargo:warning=ToolExecError: command did not execute successfully (status code exit status: 1): LC_ALL="C" "clang" "-O0" "-ffunction-sections" "-fdata-sections" "-fno-exceptions" "-g" "-fno-omit-frame-pointer" "--target=wasm32-wasip2" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/include" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/pregenerated" "-Wall" "-Wextra" "-fvisibility=hidden" "-std=c1x" "-Wall" "-Wbad-function-cast" "-Wcast-align" "-Wcast-qual" "-Wconversion" "-Wmissing-field-initializers" "-Wmissing-include-dirs" "-Wnested-externs" "-Wredundant-decls" "-Wshadow" "-Wsign-compare" "-Wsign-conversion" "-Wstrict-prototypes" "-Wundef" "-Wuninitialized" "-g3" "-nostdlibinc" "-DNDEBUG" "-DRING_CORE_NOSTDLIBINC=1" "-o" "/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/wasm32-wasip2/debug/build/ring-d0c6c53b92eada7d/out/a4019cc0736b0423-mem.o" "-c" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/crypto/mem.c"cargo:warning=error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-wasip2"'
  cargo:warning=1 error generated.

  exit status: 1
  cargo:warning=ToolExecError: command did not execute successfully (status code exit status: 1): LC_ALL="C" "clang" "-O0" "-ffunction-sections" "-fdata-sections" "-fno-exceptions" "-g" "-fno-omit-frame-pointer" "--target=wasm32-wasip2" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/include" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/pregenerated" "-Wall" "-Wextra" "-fvisibility=hidden" "-std=c1x" "-Wall" "-Wbad-function-cast" "-Wcast-align" "-Wcast-qual" "-Wconversion" "-Wmissing-field-initializers" "-Wmissing-include-dirs" "-Wnested-externs" "-Wredundant-decls" "-Wshadow" "-Wsign-compare" "-Wsign-conversion" "-Wstrict-prototypes" "-Wundef" "-Wuninitialized" "-g3" "-nostdlibinc" "-DNDEBUG" "-DRING_CORE_NOSTDLIBINC=1" "-o" "/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/wasm32-wasip2/debug/build/ring-d0c6c53b92eada7d/out/d5a9841f3dc6e253-poly1305.o" "-c" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/crypto/poly1305/poly1305.c"

  --- stderr


  error occurred in cc-rs: command did not execute successfully (status code exit status: 1): LC_ALL="C" "clang" "-O0" "-ffunction-sections" "-fdata-sections" "-fno-exceptions" "-g" "-fno-omit-frame-pointer" "--target=wasm32-wasip2" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/include" "-I" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/pregenerated" "-Wall" "-Wextra" "-fvisibility=hidden" "-std=c1x" "-Wall" "-Wbad-function-cast" "-Wcast-align" "-Wcast-qual" "-Wconversion" "-Wmissing-field-initializers" "-Wmissing-include-dirs" "-Wnested-externs" "-Wredundant-decls" "-Wshadow" "-Wsign-compare" "-Wsign-conversion" "-Wstrict-prototypes" "-Wundef" "-Wuninitialized" "-g3" "-nostdlibinc" "-DNDEBUG" "-DRING_CORE_NOSTDLIBINC=1" "-o" "/Users/milzor/agh/LSC/LSC-wasm-vs-containers-benchmark/vendor/meilisearch/target/wasm32-wasip2/debug/build/ring-d0c6c53b92eada7d/out/d5a9841f3dc6e253-poly1305.o" "-c" "/Users/milzor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ring-0.17.14/crypto/poly1305/poly1305.c"


warning: build failed, waiting for other jobs to finish...
For more information about this error, try `rustc --explain E0658`.
error: could not compile `tokio` (lib) due to 5 previous errors
```
