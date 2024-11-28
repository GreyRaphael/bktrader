pub trait OnQuote<T> {
    fn on_quote(&mut self, quote: &T);
}
