#[macro_export]
/// Take ownership of the variable and pin reference locally.
/// Returns pinned immutable reference.
macro_rules! pin_ref {
    ($name:ident) => {
        let $name = ::core::pin::pin!($name);
        let $name = ::core::pin::Pin::into_ref($name);
    };
}
