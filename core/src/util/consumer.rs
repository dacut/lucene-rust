/// Rust analog for java.util.function.Consumer
pub trait Consumer<T> {
    fn accept(&mut self, t: T);    
}
