#[macro_export]
macro_rules! EventTarget {
    (for<$($lt:lifetime),*> $($arg:ty)?) => {
        $crate::EventTarget<
            $crate::__private::ForLt!(
                for<'hkt> dyn for<$($lt),*> ::core::ops::FnMut($($arg)?) -> bool + 'hkt
            )
        >
    };

    ($($arg:ty)?) => {
        $crate::EventTarget!(for<> $($arg)?)
    };
}
