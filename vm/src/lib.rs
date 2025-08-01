//! This crate contains most of the python logic.
//!
//! - Interpreter
//! - Import mechanics
//! - Base objects
//!
//! Some stdlib modules are implemented here, but most of them are in the `rustpython-stdlib` module. The

// to allow `mod foo {}` in foo.rs; clippy thinks this is a mistake/misunderstanding of
// how `mod` works, but we want this sometimes for pymodule declarations
#![allow(clippy::module_inception)]
// we want to mirror python naming conventions when defining python structs, so that does mean
// uppercase acronyms, e.g. TextIOWrapper instead of TextIoWrapper
#![allow(clippy::upper_case_acronyms)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/RustPython/RustPython/main/logo.png")]
#![doc(html_root_url = "https://docs.rs/rustpython-vm/")]

#[cfg(feature = "flame-it")]
#[macro_use]
extern crate flamer;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
// extern crate env_logger;

#[macro_use]
extern crate rustpython_derive;

extern crate self as rustpython_vm;

pub use rustpython_derive::*;

//extern crate eval; use eval::eval::*;
// use py_code_object::{Function, NativeType, PyCodeObject};

// This is above everything else so that the defined macros are available everywhere
#[macro_use]
pub(crate) mod macros;

mod anystr;
pub mod buffer;
pub mod builtins;
pub mod byte;
mod bytes_inner;
pub mod cformat;
pub mod class;
mod codecs;
pub mod compiler;
pub mod convert;
mod coroutine;
mod dict_inner;

#[cfg(feature = "rustpython-compiler")]
pub mod eval;

pub mod exceptions;
pub mod format;
pub mod frame;
pub mod function;
pub mod import;
mod intern;
pub mod iter;
pub mod object;

#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
pub mod ospath;

pub mod prelude;
pub mod protocol;
pub mod py_io;

#[cfg(feature = "serde")]
pub mod py_serde;

pub mod readline;
pub mod recursion;
pub mod scope;
pub mod sequence;
pub mod signal;
pub mod sliceable;
pub mod stdlib;
pub mod suggestion;
pub mod types;
pub mod utils;
pub mod version;
pub mod vm;
pub mod warn;

#[cfg(windows)]
pub mod windows;

pub use self::convert::{TryFromBorrowedObject, TryFromObject};
pub use self::object::{
    AsObject, Py, PyAtomicRef, PyExact, PyObject, PyObjectRef, PyPayload, PyRef, PyRefExact,
    PyResult, PyWeakRef,
};
pub use self::vm::{Context, Interpreter, Settings, VirtualMachine};

pub use rustpython_common as common;
pub use rustpython_compiler_core::{bytecode, frozen};
pub use rustpython_literal as literal;

#[doc(hidden)]
pub mod __exports {
    pub use paste;
}
