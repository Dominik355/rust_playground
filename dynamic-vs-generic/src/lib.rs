pub trait Deserializator<T> {
    fn deserialize(&self, input: &str) -> T;
}

pub struct StringDeserializator;

impl Deserializator<String> for StringDeserializator {
    fn deserialize(&self, input: &str) -> String {
        input.to_string()
    }
}

pub struct IntDeserializator;

impl Deserializator<i32> for IntDeserializator {
    fn deserialize(&self, input: &str) -> i32 {
        input.parse().unwrap_or(0)
    }
}

pub struct DynamicDeserializatorWrapper<T> {
    deserializator: Box<dyn Deserializator<T>>,
}

impl<T> DynamicDeserializatorWrapper<T> {
    pub fn new(deserializator: Box<dyn Deserializator<T>>) -> Self {
        DynamicDeserializatorWrapper { deserializator }
    }

    pub fn deserialize(&self, input: &str) -> T {
        self.deserializator.deserialize(input)
    }
}

pub struct GenericDeserializationWrapper<T, D: Deserializator<T>> {
    deserializator: D,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, D: Deserializator<T>> GenericDeserializationWrapper<T, D> {
    pub fn new(deserializator: D) -> Self {
        GenericDeserializationWrapper {
            deserializator,
            _phantom: Default::default(),
        }
    }

    pub fn deserialize(&self, input: &str) -> T {
        self.deserializator.deserialize(input)
    }
}
