use {
    crate::util::{
        attribute::Attribute,
        attribute_source::{AttributeSource, AttributeSourceBase},
    },
    std::{any::TypeId, collections::HashMap, future::Future, io::Result as IoResult, pin::Pin},
};

/// A `TokenStream` enumerates the sequence of tokens, either from [Field]s of a
/// [Document] or from query text.
///
/// This is a trait with a partial implementation in [TokenStreamBase]; concrete subclasses are:
///
/// * [Tokenizer], a `TokenStream` whose input is an [tokio::io::AsyncRead] stream; and
/// * [TokenFilter], a `TokenStream` whose input is another `TokenStream`
///
/// `TokenStream` has [AttributeSource] as a supertrait, which provides access to all of the
/// token attributes for the `TokenStream`.
///
/// **The workflow of the new `TokenStream` API is as follows:**
///
/// 1. Instantiation of `TokenStream`/[TokenFilter]s which add/get attributes
///    to/from the [AttributeSource].
/// 1. The consumer calls [TokenStream::reset].
/// 1. The consumer retrieves attributes from the stream and stores local references to all
///    attributes it wants to access.
/// 1. The consumer calls [TokenStream::increment_token] until it returns false consuming the
///     attributes after each call.
/// 1. The consumer calls [TokenStream::end] so that any end-of-stream operations can be performed.
/// 1. The consumer drops the TokenStream to release any resource when finished using it.
///
/// To make sure that filters and consumers know which attributes are available, the attributes must
/// be added during instantiation. Filters and consumers are not required to check for availability
/// of attributes in [TokenStream::increment_token].
///
/// You can find some example code for the new API in the analysis module documentation\.
///
/// Sometimes it is desirable to capture a current state of a `TokenStream`, e.g., for
/// buffering purposes (see [CachingTokenFilter], TeeSinkTokenFilter). For this usecase
/// [AttributeSource::get_state] and [AttributeSource::set_state] can be used.
///
/// The `TokenStream`-API in Lucene is based on the decorator pattern.
pub trait TokenStream: AttributeSource {
    /// Consumers (i.e., [IndexWriter]) use this method to advance the stream to the next token.
    /// Implementing classes must implement this method and update the appropriate attributes
    /// with the attributes of the next token.
    ///
    /// The producer must make no assumptions about the attributes after the method has been
    /// returned: the caller may arbitrarily change it. If the producer needs to preserve the state for
    /// subsequent calls, it can use [AttributeSource::get_state] to create a copy of the current attribute
    /// state.
    ///
    /// This method is called for every token of a document, so an efficient implementation is
    /// crucial for good performance. To avoid calls to [AttributeSource::add_attribute] and
    /// [AttributeSource::get_attribute], references to all attributes that this stream uses should be
    /// retrieved during instantiation.
    ///
    /// To ensure that filters and consumers know which attributes are available, the attributes
    /// must be added during instantiation. Filters and consumers are not required to check for
    /// availability of attributes in [Self::increment_token].
    ///
    /// # Returns
    /// `false` for end of stream; `true` otherwise
    fn increment_token(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<bool>>>>;

    /// This method is called by the consumer after the last token has been consumed, after
    /// [TokenStream::increment_token] returned `false` (using the new `TokenStream` API).
    /// Streams implementing the old API should upgrade to use this feature.
    ///
    /// This method can be used to perform any end-of-stream operations, such as setting the final
    /// offset of a stream. The final offset of a stream might differ from the offset of the last token
    /// e.g. in case one or more whitespaces followed after the last token, but a WhitespaceTokenizer was
    /// used.
    ///
    /// Additionally any skipped positions (such as those removed by a stopfilter) can be applied to
    /// the position increment, or any adjustment of other attributes where the end-of-stream value may
    /// be important.
    fn end(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<()>>>>;

    /// This method is called by a consumer before it begins consumption using
    /// [TokenStream::increment_token].
    ///
    /// Resets this stream to a clean state. Stateful implementations must implement this method so
    /// that they can be reused, just as if they had been created fresh.
    fn reset(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<()>>>>;
}

/// Partial implementation of TokenStream
pub struct TokenStreamBase {
    attribute_source_base: AttributeSourceBase,
}

impl AttributeSource for TokenStreamBase {
    fn add_attribute(&mut self, attribute: Box<dyn Attribute>) {
        self.attribute_source_base.add_attribute(attribute)
    }

    fn get_attribute(&self, r#type: TypeId) -> Option<&dyn Attribute> {
        self.attribute_source_base.get_attribute(r#type)
    }

    fn has_attributes(&self) -> bool {
        self.attribute_source_base.has_attributes()
    }

    fn has_attribute(&self, r#type: TypeId) -> bool {
        self.attribute_source_base.has_attribute(r#type)
    }

    fn clear_attributes(&mut self) {
        self.attribute_source_base.clear_attributes()
    }

    fn end_attributes(&mut self) {
        self.attribute_source_base.end_attributes()
    }

    fn remove_all_attributes(&mut self) {
        self.attribute_source_base.remove_all_attributes()
    }

    fn get_state(&self) -> HashMap<TypeId, Box<dyn Attribute>> {
        self.attribute_source_base.get_state()
    }

    fn set_state(&mut self, state: HashMap<TypeId, Box<dyn Attribute>>) {
        self.attribute_source_base.set_state(state)
    }
}

impl TokenStreamBase {
    /// A TokenStream using the default attribute factory.
    pub fn new() -> Self {
        Self {
            attribute_source_base: AttributeSourceBase::new(),
        }
    }

    pub fn from_attribute_source(attribute_source: AttributeSourceBase) -> Self {
        Self {
            attribute_source_base: attribute_source,
        }
    }

    fn end(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<()>>>> {
        Box::pin(async move {
            self.end_attributes();
            Ok(())
        })
    }
}
