#[macro_export]
macro_rules! EventTarget {
    (for<$($lt:lifetime),*> $arg:ty) => {
        $crate::EventTarget<
            $crate::hkt::Wrapper<
                dyn for<'hkt> $crate::hkt::Hkt<
                    'hkt,
                    T = dyn for<$($lt),*> ::core::ops::FnMut($arg) -> bool + 'hkt
                >
            >
        >
    };

    ($arg:ty) => {
        $crate::EventTarget!(for<> $arg)
    };
}
