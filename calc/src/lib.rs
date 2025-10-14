#[macro_export]
macro_rules! described_struct {
    (
        struct $name:ident {
            $(
                $(#[$meta:meta])* $field:ident : $type:ty => $desc:expr
            ),* $(,)?
        }
    ) => {
        pub struct $name {
            $( $(#[$meta])* pub $field: $type ),*
        }
        impl $name {
            pub const DESCRIPTIONS: &'static [(&'static str, &'static str)] = &[
                $( (stringify!($field), $desc) ),*
            ];
        }
    };
}
