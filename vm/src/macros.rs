#[macro_export]
macro_rules! extend_module {
    ( $vm:expr, $module:expr, { $($name:expr => $value:expr),* $(,)? }) => {{
        $(
            $vm.__module_set_attr($module, $vm.ctx.intern_str($name), $value).unwrap();
        )*
    }};
}

#[macro_export]
macro_rules! py_class {
    ( $ctx:expr, $class_name:expr, $class_base:expr, { $($name:tt => $value:expr),* $(,)* }) => {
        py_class!($ctx, $class_name, $class_base, $crate::types::PyTypeFlags::BASETYPE, { $($name => $value),* })
    };
    ( $ctx:expr, $class_name:expr, $class_base:expr, $flags:expr, { $($name:tt => $value:expr),* $(,)* }) => {
        {
            #[allow(unused_mut)]
            let mut slots = $crate::types::PyTypeSlots::heap_default();
            slots.flags = $flags;
            $($crate::py_class!(@extract_slots($ctx, &mut slots, $name, $value));)*
            let py_class = $ctx.new_class(None, $class_name, $class_base, slots);
            $($crate::py_class!(@extract_attrs($ctx, &py_class, $name, $value));)*
            py_class
        }
    };
    (@extract_slots($ctx:expr, $slots:expr, (slot $slot_name:ident), $value:expr)) => {
        $slots.$slot_name.store(Some($value));
    };
    (@extract_slots($ctx:expr, $class:expr, $name:expr, $value:expr)) => {};
    (@extract_attrs($ctx:expr, $slots:expr, (slot $slot_name:ident), $value:expr)) => {};
    (@extract_attrs($ctx:expr, $class:expr, $name:expr, $value:expr)) => {
        $class.set_attr($name, $value);
    };
}

#[macro_export]
macro_rules! extend_class {
    ( $ctx:expr, $class:expr, { $($name:expr => $value:expr),* $(,)* }) => {
        $(
            $class.set_attr($ctx.intern_str($name), $value.into());
        )*
    };
}

#[macro_export]
macro_rules! py_namespace {
    ( $vm:expr, { $($name:expr => $value:expr),* $(,)* }) => {
        {
            let namespace = $crate::object::PyPayload::into_ref($crate::builtins::PyNamespace {}, &$vm.ctx);
            let obj = $crate::object::AsObject::as_object(&namespace);
            $(
                obj.generic_setattr($vm.ctx.intern_str($name), $crate::function::PySetterValue::Assign($value.into()), $vm).unwrap();
            )*
            namespace
        }
    }
}

/// Macro to match on the built-in class of a Python object.
///
/// Like `match`, `match_class!` must be exhaustive, so a default arm without
/// casting is required.
///
/// # Examples
///
/// ```
/// use malachite_bigint::ToBigInt;
/// use num_traits::Zero;
///
/// use rustpython_vm::match_class;
/// use rustpython_vm::builtins::{PyFloat, PyInt};
/// use rustpython_vm::{PyPayload};
///
/// # rustpython_vm::Interpreter::without_stdlib(Default::default()).enter(|vm| {
/// let obj = PyInt::from(0).into_pyobject(vm);
/// assert_eq!(
///     "int",
///     match_class!(match obj {
///         PyInt => "int",
///         PyFloat => "float",
///         _ => "neither",
///     })
/// );
/// # });
///
/// ```
///
/// With a binding to the downcasted type:
///
/// ```
/// use malachite_bigint::ToBigInt;
/// use num_traits::Zero;
///
/// use rustpython_vm::match_class;
/// use rustpython_vm::builtins::{PyFloat, PyInt};
/// use rustpython_vm::{ PyPayload};
///
/// # rustpython_vm::Interpreter::without_stdlib(Default::default()).enter(|vm| {
/// let obj = PyInt::from(0).into_pyobject(vm);
///
/// let int_value = match_class!(match obj {
///     i @ PyInt => i.as_bigint().clone(),
///     f @ PyFloat => f.to_f64().to_bigint().unwrap(),
///     obj => panic!("non-numeric object {:?}", obj),
/// });
///
/// assert!(int_value.is_zero());
/// # });
/// ```
#[macro_export]
macro_rules! match_class {
    // The default arm.
    (match ($obj:expr) { _ => $default:expr $(,)? }) => {
        $default
    };

    // The default arm, binding the original object to the specified identifier.
    (match ($obj:expr) { $binding:ident => $default:expr $(,)? }) => {{
        #[allow(clippy::redundant_locals)]
        let $binding = $obj;
        $default
    }};
    (match ($obj:expr) { ref $binding:ident => $default:expr $(,)? }) => {{
        #[allow(clippy::redundant_locals)]
        let $binding = &$obj;
        $default
    }};

    // An arm taken when the object is an instance of the specified built-in
    // class and binding the downcasted object to the specified identifier and
    // the target expression is a block.
    (match ($obj:expr) { $binding:ident @ $class:ty => $expr:block $($rest:tt)* }) => {
        $crate::match_class!(match ($obj) { $binding @ $class => ($expr), $($rest)* })
    };
    (match ($obj:expr) { ref $binding:ident @ $class:ty => $expr:block $($rest:tt)* }) => {
        $crate::match_class!(match ($obj) { ref $binding @ $class => ($expr), $($rest)* })
    };

    // An arm taken when the object is an instance of the specified built-in
    // class and binding the downcasted object to the specified identifier.
    (match ($obj:expr) { $binding:ident @ $class:ty => $expr:expr, $($rest:tt)* }) => {
        match $obj.downcast::<$class>() {
            Ok($binding) => $expr,
            Err(_obj) => $crate::match_class!(match (_obj) { $($rest)* }),
        }
    };
    (match ($obj:expr) { ref $binding:ident @ $class:ty => $expr:expr, $($rest:tt)* }) => {
        match $obj.downcast_ref::<$class>() {
            ::std::option::Option::Some($binding) => $expr,
            ::std::option::Option::None => $crate::match_class!(match ($obj) { $($rest)* }),
        }
    };

    // An arm taken when the object is an instance of the specified built-in
    // class and the target expression is a block.
    (match ($obj:expr) { $class:ty => $expr:block $($rest:tt)* }) => {
        $crate::match_class!(match ($obj) { $class => ($expr), $($rest)* })
    };

    // An arm taken when the object is an instance of the specified built-in
    // class.
    (match ($obj:expr) { $class:ty => $expr:expr, $($rest:tt)* }) => {
        if $obj.downcastable::<$class>() {
            $expr
        } else {
            $crate::match_class!(match ($obj) { $($rest)* })
        }
    };

    // To allow match expressions without parens around the match target
    (match $($rest:tt)*) => {
        $crate::match_class!(@parse_match () ($($rest)*))
    };
    (@parse_match ($($target:tt)*) ({ $($inner:tt)* })) => {
        $crate::match_class!(match ($($target)*) { $($inner)* })
    };
    (@parse_match ($($target:tt)*) ($next:tt $($rest:tt)*)) => {
        $crate::match_class!(@parse_match ($($target)* $next) ($($rest)*))
    };
}

#[macro_export]
macro_rules! identifier(
    ($as_ctx:expr, $name:ident) => {
        $as_ctx.as_ref().names.$name
    };
);

#[macro_export]
macro_rules! identifier_utf8(
    ($as_ctx:expr, $name:ident) => {
        // Safety: All known identifiers are ascii strings.
        #[allow(clippy::macro_metavars_in_unsafe)]
        unsafe { $as_ctx.as_ref().names.$name.as_object().downcast_unchecked_ref::<$crate::builtins::PyUtf8Str>() }
    };
);

/// Super detailed logging. Might soon overflow your log buffers
/// Default, this logging is discarded, except when a the `vm-tracing-logging`
/// build feature is enabled.
macro_rules! vm_trace {
    ($($arg:tt)+) => {
        #[cfg(feature = "vm-tracing-logging")]
        trace!($($arg)+);
    }
}

macro_rules! flame_guard {
    ($name:expr) => {
        #[cfg(feature = "flame-it")]
        let _guard = ::flame::start_guard($name);
    };
}

#[macro_export]
macro_rules! class_or_notimplemented {
    ($t:ty, $obj:expr) => {{
        let a: &$crate::PyObject = &*$obj;
        match $crate::PyObject::downcast_ref::<$t>(&a) {
            Some(pyref) => pyref,
            None => return Ok($crate::function::PyArithmeticValue::NotImplemented),
        }
    }};
}

#[macro_export]
macro_rules! named_function {
    ($ctx:expr, $module:ident, $func:ident) => {{
        #[allow(unused_variables)] // weird lint, something to do with paste probably
        let ctx: &$crate::Context = &$ctx;
        $crate::__exports::paste::expr! {
            ctx.new_method_def(
                stringify!($func),
                [<$module _ $func>],
                ::rustpython_vm::function::PyMethodFlags::empty(),
            )
            .to_function()
            .with_module(ctx.intern_str(stringify!($module)).into())
            .into_ref(ctx)
        }
    }};
}

// can't use PyThreadingConstraint for stuff like this since it's not an auto trait, and
// therefore we can't add it ad-hoc to a trait object
cfg_if::cfg_if! {
    if #[cfg(feature = "threading")] {
        macro_rules! py_dyn_fn {
            (dyn Fn($($arg:ty),*$(,)*) -> $ret:ty) => {
                dyn Fn($($arg),*) -> $ret + Send + Sync + 'static
            };
        }
    } else {
        macro_rules! py_dyn_fn {
            (dyn Fn($($arg:ty),*$(,)*) -> $ret:ty) => {
                dyn Fn($($arg),*) -> $ret + 'static
            };
        }
    }
}
