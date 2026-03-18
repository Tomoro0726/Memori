use std::marker::PhantomData;
pub mod impls;

pub struct ScalingBench<I, F, R> {
    name: String,
    description: Option<String>,
    inputs: Vec<I>, // Runnerではなく引数のリストを持つ
    function: F,    // 関数は1つだけ持つ
    _marker: PhantomData<fn(I) -> R>,
}

impl<I, F, R> ScalingBench<I, F, R>
where
    I: Clone,
    F: FnMut(&I) -> R,
{
    pub fn new<T>(name: &str, inputs: T, function: F) -> Self
    where
        T: Into<Vec<I>>,
    {
        Self {
            name: name.to_string(),
            description: None,
            inputs: inputs.into(),
            function,
            _marker: PhantomData,
        }
    }
}
