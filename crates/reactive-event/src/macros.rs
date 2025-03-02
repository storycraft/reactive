#[macro_export]
macro_rules! EventTarget {
    (for<$($lt:lifetime),*> $($arg:ty)?) => {
        $crate::EventTarget<
            $crate::__private::ForLt!(
                for<'__fn> dyn for<$($lt),*> ::core::ops::FnMut($($arg)?) -> bool + '__fn
            )
        >
    };

    ($($arg:ty)?) => {
        $crate::EventTarget!(for<> $($arg)?)
    };
}
