pub trait QuoteHandler<T> {
    fn on_quote(&mut self, quote: &T);
}
