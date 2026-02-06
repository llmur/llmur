#[macro_export]
macro_rules! impl_from_vec_result {
    ($type:ty, $container:ty, $vec_field:ident) => {
        impl From<Vec<$type>> for $container {
            fn from(vec: Vec<$type>) -> Self {
                Self {
                    total: vec.len(),
                    $vec_field: vec,
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_from_vec_into_result {
    ($type:ty, $container:ty, $vec_field:ident) => {
        impl From<Vec<$type>> for $container {
            fn from(vec: Vec<$type>) -> Self {
                Self {
                    total: vec.len(),
                    $vec_field: vec.into_iter().map(Into::into).collect(),
                }
            }
        }
    };
}
