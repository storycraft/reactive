#[macro_export]
macro_rules! pin_ref {
    ($name:ident) => {
        let $name = ::core::pin::pin!($name);
        let $name = ::core::pin::Pin::into_ref($name);
    };
}
