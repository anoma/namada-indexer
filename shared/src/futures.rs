use std::future::Future;

pub trait AwaitContainer<F> {
    type Output<T>;

    #[allow(async_fn_in_trait)]
    async fn future<T>(self) -> Self::Output<F::Output>
    where
        F: Future<Output = T>;
}

impl<F> AwaitContainer<F> for Option<F> {
    type Output<X> = Option<X>;

    #[inline]
    async fn future<T>(self) -> Option<T>
    where
        F: Future<Output = T>,
    {
        if let Some(fut) = self {
            Some(fut.await)
        } else {
            None
        }
    }
}

impl<F, E> AwaitContainer<F> for Result<F, E> {
    type Output<X> = Result<X, E>;

    #[inline]
    async fn future<T>(self) -> Result<T, E>
    where
        F: Future<Output = T>,
    {
        match self {
            Ok(fut) => Ok(fut.await),
            Err(err) => Err(err),
        }
    }
}

impl<F> AwaitContainer<F> for Vec<F> {
    type Output<X> = Vec<X>;

    #[inline]
    async fn future<T>(self) -> Vec<T>
    where
        F: Future<Output = T>,
    {
        ::futures::future::join_all(self).await
    }
}
