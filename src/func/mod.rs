pub mod output;

pub enum Bench<I> {
    Instant(I),
    Scaling(Vec<I>),
}

pub struct BenchPattern<I> {
    pub name: String,
    pub description: String,
    pub input: Bench<I>,
}

pub struct Func<I>
where
    I: Clone,
{
    name: String,
    description: Option<String>,
    functions: Vec<(String, Box<dyn FnMut(&I)>)>,
    patterns: Vec<BenchPattern<I>>,
}

impl<I> Func<I>
where
    I: Clone,
{
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            functions: Vec::new(),
            patterns: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn add_function<F, R>(mut self, name: &str, mut func: F) -> Self
    where
        F: FnMut(&I) -> R + 'static,
    {
        let wrapped = move |i: &I| {
            let _ = std::hint::black_box(func(i));
        };
        self.functions.push((name.to_string(), Box::new(wrapped)));
        self
    }

    pub fn add_bench(mut self, name: &str, description: &str, input: Bench<I>) -> Self {
        self.patterns.push(BenchPattern {
            name: name.to_string(),
            description: description.to_string(),
            input,
        });
        self
    }
}
