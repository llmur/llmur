use async_trait::async_trait;

// Apply `#[async_trait]` to the `AsyncInto` trait
#[async_trait]
pub trait AsyncInto<T>: Sized {
    async fn into_async(self) -> T;
}

#[async_trait]
pub trait AsyncFrom<T>: Sized {
    async fn from_async(value: T) -> Self;
}

// Default implementation for `AsyncInto` using `AsyncFrom`
#[async_trait]
impl<T, U> AsyncInto<U> for T
where
    U: AsyncFrom<T> + Send, // Ensure `U` is `Send` if required by `async_trait`
    T: Send,                // Ensure `T` is `Send` if required by `async_trait`
{
    async fn into_async(self) -> U {
        U::from_async(self).await
    }
}