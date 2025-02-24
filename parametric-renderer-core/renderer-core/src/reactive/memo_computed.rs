use reactive_graph::computed::Memo;

pub trait MemoComputed<T>
where
    T: Send + Sync + 'static,
{
    fn new_computed(fun: impl Fn(Option<&T>) -> T + Send + Sync + 'static) -> Memo<T>;
}

impl<T> MemoComputed<T> for Memo<T>
where
    T: Send + Sync + 'static,
{
    fn new_computed(fun: impl Fn(Option<&T>) -> T + Send + Sync + 'static) -> Memo<T> {
        Memo::new_with_compare(fun, |_, _| true)
    }
}
