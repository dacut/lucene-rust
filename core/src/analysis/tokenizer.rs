use {crate::analysis::token_stream::TokenStream, tokio::io::AsyncRead};

/// A Tokenizer is a TokenStream whose input is an [AsyncRead]er.
pub trait Tokenizer: TokenStream {
    /// Return the corrected offset. If [Tokenizer::input] is a [CharFilter] subclass this method
    /// calls [CharFilter::correct_offset], else returns `current_off`.
    fn correct_offset(&self, current_off: u32) -> u32;

    /// Expert: Set a new reader on the Tokenizer. Typically, an analyzer (in its token_stream method)
    /// will use this to re-use a previously created tokenizer.
    fn set_reader(&mut self, reader: Box<dyn AsyncRead>);

    /// Clones this [Tokenizer] as a [TokenStream].
    fn as_token_stream(&self) -> Box<dyn TokenStream>;
}
