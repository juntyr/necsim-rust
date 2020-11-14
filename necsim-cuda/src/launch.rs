#[macro_export]
macro_rules! type_checked_launch {
    ($module:ident . $function:ident <<<$grid:expr, $block:expr, $shared:expr, $stream:ident>>>(
        $($param:ident: $ty:ty = $arg:expr),*
    )) => {
        {
            $(
            let $param: $ty = $arg;
            )*

            launch!($module.$function<<<$grid, $block, $shared, $stream>>>($($param),*))
        }
    };
    ($function:ident <<<$grid:expr, $block:expr, $shared:expr, $stream:ident>>>(
        $($param:ident: $ty:ty = $arg:expr),*
    )) => {
        {
            $(
            let $param: $ty = $arg;
            )*

            launch!($function<<<$grid, $block, $shared, $stream>>>($($param),*))
        }
    };
}
