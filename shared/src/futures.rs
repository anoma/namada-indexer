use std::future::Future;

pub trait AwaitOption<F> {
    type Output;

    #[allow(async_fn_in_trait)]
    async fn future(self) -> Option<Self::Output>;
}

impl<F, T> AwaitOption<F> for Option<F>
where
    F: Future<Output = T>,
{
    type Output = T;

    #[inline]
    async fn future(self) -> Option<T> {
        if let Some(fut) = self {
            Some(fut.await)
        } else {
            None
        }
    }
}
