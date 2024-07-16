#[derive(Default)]
struct InnerBuilder {
    pub name: String,
    pub parallelism: usize,
}

#[derive(Default)]
struct OuterBuilder {
    pub title: String,
    pub size: usize,
    pub inner: InnerBuilder,
}

impl OuterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.inner = self.inner.name(name);
        self
    }

    pub fn parallelism(mut self, parallelism: usize) -> Self {
        self.inner = self.inner.parallelism(parallelism);
        self
    }

    pub fn build(self) -> String {
        format!("{:?} | {:?}", self.title, self.inner.build())
    }
}

impl InnerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn parallelism(mut self, parallelism: usize) -> Self {
        self.parallelism = parallelism;
        self
    }

    pub fn build(self) -> String {
        self.name
    }
}

fn main() {
    let r = OuterBuilder::new()
        .parallelism(23)
        .title("title")
        .name("name")
        .build();

    println!("huh? {:?}", r);
}
