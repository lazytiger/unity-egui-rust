struct Placement<T> {
    value: T,
}

impl Placement<i32> {
    fn calculate(&self) -> i32 {
        (1..=self.value).product()
    }
}

fn main() {}
