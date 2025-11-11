#[macro_export]
macro_rules! boxed_async {
    ($body:expr) => {
        Box::pin($body) as Pin<Box<dyn Future<Output = _> + Send>>
    };
}
