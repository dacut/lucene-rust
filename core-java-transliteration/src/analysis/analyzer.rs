use {
    crate::{
        analysis::{token_stream::TokenStream, tokenizer::Tokenizer},
        util::consumer::Consumer,
    },
    tokio::io::AsyncRead,
};

/// An Analyzer builds [TokenStreams], which analyze text. It thus represents a policy for extracting
/// index terms from text.
///
/// In order to define what analysis is done, implementations must define their [TokenStreamComponents]
/// in [Analyzer::create_components]. The components
/// are then reused in each call to [Analyzer::token_stream].
///
/// Simple example:
///
/// TODO: Convert to Rust.
///
/// ```ignore
/// Analyzer analyzer = new Analyzer() {
///  {@literal @Override}
///   protected TokenStreamComponents createComponents(String fieldName) {
///     Tokenizer source = new FooTokenizer(reader);
///     TokenStream filter = new FooFilter(source);
///     filter = new BarFilter(filter);
///     return new TokenStreamComponents(source, filter);
///   }
///   {@literal @Override}
///   protected TokenStream normalize(TokenStream in) {
///     // Assuming FooFilter is about normalization and BarFilter is about
///     // stemming, only FooFilter should be applied
///     return new FooFilter(in);
///   }
/// };
/// ```
///
/// For more examples, see the [Analysis module documentation]([crate::analysis]).
///
/// For some concrete implementations bundled with Lucene, look in the analysis modules:
///
/// <ul>
///   <li><a href="{@docRoot}/../analysis/common/overview-summary.html">Common</a>: Analyzers for
///       indexing content in different languages and domains.
///   <li><a href="{@docRoot}/../analysis/icu/overview-summary.html">ICU</a>: Exposes functionality
///       from ICU to Apache Lucene.
///   <li><a href="{@docRoot}/../analysis/kuromoji/overview-summary.html">Kuromoji</a>: Morphological
///       analyzer for Japanese text.
///   <li><a href="{@docRoot}/../analysis/morfologik/overview-summary.html">Morfologik</a>:
///       Dictionary-driven lemmatization for the Polish language.
///   <li><a href="{@docRoot}/../analysis/phonetic/overview-summary.html">Phonetic</a>: Analysis for
///       indexing phonetic signatures (for sounds-alike search).
///   <li><a href="{@docRoot}/../analysis/smartcn/overview-summary.html">Smart Chinese</a>: Analyzer
///       for Simplified Chinese, which indexes words.
///   <li><a href="{@docRoot}/../analysis/stempel/overview-summary.html">Stempel</a>: Algorithmic
///       Stemmer for the Polish Language.
/// </ul>
pub trait Analyzer {
    /// Creates a new [TokenStreamComponents] instance for this analyzer.
    ///
    /// # Parameters
    /// * `field_name`: the name of the fields content passed to the [TokenStreamComponents]
    ///    sink as a reader
    ///
    /// # Returns
    /// The [TokenStreamComponents] for this analyzer.
    fn create_components(&mut self, field_name: &str) -> TokenStreamComponents;

    /// Wrap the given [TokenStream] in order to apply normalization filters. The default
    /// implementation returns the [TokenStream] as-is. This is used by [Analyzer::normalize_text].
    fn normalize(&self, field_name: &str, r#in: Box<dyn TokenStream>) -> Box<dyn TokenStream> {
        r#in
    }

    /// Returns a TokenStream suitable for `field_name`, tokenizing the contents of `reader`.
    ///
    /// This method uses [Analyzer::create_components] to obtain an instance of
    /// [TokenStreamComponents]. It returns the sink of the components and stores the components
    /// internally. Subsequent calls to this method will reuse the previously stored components after
    /// resetting them through [TokenStreamComponents::set_reader].
    ///
    /// # Note
    /// After calling this method, the consumer must follow the workflow described in
    /// [TokenStream] to properly consume its contents. See the
    /// [Analysis module documentation]([crate::analysis])
    /// for some examples demonstrating this.
    ///
    /// If your data is available as a string, use [Analyzer::token_stream_for_text]
    /// which reuses a StringReader-like instance internally.
    ///
    /// # Parameters
    /// * `field_name`: the name of the field the created TokenStream is used for
    /// * `reader`: the reader the streams source reads from
    ///
    /// # Returns
    /// [TokenStream] for iterating the analyzed content of `reader`
    fn token_stream(&mut self, field_name: &str, reader: Box<dyn AsyncRead>) -> Box<dyn TokenStream>;

    /// Returns a TokenStream suitable for `field_name`, tokenizing the contents of `text`.
    ///
    /// This method uses [Analyzer::create_components] to obtain an instance of
    /// [TokenStreamComponents]. It returns the sink of the components and stores the components
    /// internally. Subsequent calls to this method will reuse the previously stored components after
    /// resetting them through [TokenStreamComponents::set_reader].
    ///
    /// # Note
    /// After calling this method, the consumer must follow the workflow described in
    /// [TokenStream] to properly consume its contents. See the
    /// [Analysis module documentation]([crate::analysis])
    /// for some examples demonstrating this.
    ///
    /// # Parameters
    /// * `field_name`: the name of the field the created TokenStream is used for
    /// * `text`: the String the streams source reads from
    ///
    /// # Returns
    /// [TokenStream] for iterating the analyzed content of `text`
    fn token_stream_for_text(&mut self, field_name: &str, text: &str) -> Box<dyn TokenStream>;

    /// Normalize a string down to the representation that it would have in the index.
    ///
    /// This is typically used by query parsers in order to generate a query on a given term,
    /// without tokenizing or stemming, which are undesirable if the string to analyze is a partial
    /// word (eg. in case of a wildcard or fuzzy query).
    ///
    /// This method uses [Analyzer::init_reader_for_normalization] in order to apply
    /// necessary character-level normalization and then [Analyzer::normalize] in
    /// order to apply the normalizing token filters.
    fn normalize_text(&self, field_name: &str, text: &str) -> String;

    /// Override this if you want to add a CharFilter chain.
    ///
    /// The default implementation returns `reader` unchanged
    ///
    /// # Parameters
    /// * `field_name`: IndexableField name being indexed
    /// * `reader`: Original reader
    ///
    /// # Returns
    /// The reader, opetionally decorated with CharFilter(s)
    fn init_reader(&self, field_name: &str, reader: Box<dyn AsyncRead>) -> Box<dyn AsyncRead> {
        reader
    }

    /// Wrap the given [AsyncRead]er with [CharFilter]s that make sense for normalization. This
    /// is typically a subset of the [CharFilter]s that are applied in [Analyzer::init_reader].
    ///  This is used by [Analyzer::normalize_text].
    fn init_reader_for_normalization(&self, field_name: &str, reader: Box<dyn AsyncRead>) -> Box<dyn AsyncRead> {
        reader
    }

    /// Invoked before indexing a IndexableField instance if terms have already been added to that
    /// field. This allows custom analyzers to place an automatic position increment gap between
    /// IndexbleField instances using the same field name. The default value position increment gap is
    /// 0. With a 0 position increment gap and the typical default token position increment of 1, all
    /// terms in a field, including across IndexableField instances, are in successive positions,
    /// allowing exact PhraseQuery matches, for instance, across IndexableField instance boundaries.
    ///
    /// # Parameters
    /// * `field_name`: IndexableField name being indexed.
    ///
    /// # Returns
    /// The position increment gap, added to the next token emitted from [Analyzer::token_stream].
    fn get_position_increment_gap(&self, field_name: &str) -> u32 {
        0
    }

    /// Just like [Analyzer::get_position_increment_gap], except for Token offsets instead. By default this
    /// returns 1. This method is only called if the field produced at least one token for indexing.
    ///
    /// # Parameters
    /// * `field_name`: the field just indexed
    ///
    /// # Returns
    /// The offset gap, added to the next token emitted from [Analyzer::token_stream]
    fn get_offset_gap(&self, field_name: &str) -> u32 {
        1
    }
}

/// This class encapsulates the outer components of a token stream. It provides access to the
/// source (an [AsyncRead] Consumer ?? and the outer end (sink), an instance of [TokenFilter]
/// which also serves as the [TokenStream] returned by [Analyzer::token_stream].
pub struct TokenStreamComponents {
    /// Original source of the tokens.
    source: Box<dyn Consumer<Box<dyn AsyncRead>>>,

    /// Sink tokenstream, such as the outer tokenfilter decorating the chain. This can be the source
    /// if there are no filters.
    sink: Box<dyn TokenStream>,
}

impl TokenStreamComponents {
    /// Creates a new [TokenStreamComponents] instance.
    ///
    /// # Parameters
    /// * `source`: The source to set the reader on.
    /// * `result`: The analyzer's resulting token stream.
    pub fn new(source: Box<dyn Consumer<Box<dyn AsyncRead>>>, result: Box<dyn TokenStream>) -> Self {
        TokenStreamComponents {
            source,
            sink: result,
        }
    }

    /// Creates a new [TokenStreamComponents] instance.
    ///
    /// # Parameters
    /// * `tokenizer`: The analyzer's Tokenizer.
    /// * `result`: The analyzer's resulting token stream.
    pub fn from_tokenizer(tokenizer: Box<dyn Tokenizer>) -> Self {
        let ts = tokenizer.as_token_stream();
        Self::new(Box::new(SetTokenizerReader::from(tokenizer)), ts)
    }

    /// Creates a new [TokenStreamComponents] instance.
    ///
    /// # Parameters
    /// * `tokenizer`: The analyzer's Tokenizer.
    /// * `result`: The analyzer's resulting token stream.
    pub fn from_tokenizer_and_result(tokenizer: Box<dyn Tokenizer>, result: Box<dyn TokenStream>) -> Self {
        Self::new(Box::new(SetTokenizerReader::from(tokenizer)), result)
    }
}

struct SetTokenizerReader {
    tokenizer: Box<dyn Tokenizer>,
}

impl From<Box<dyn Tokenizer>> for SetTokenizerReader {
    fn from(tokenizer: Box<dyn Tokenizer>) -> Self {
        SetTokenizerReader {
            tokenizer,
        }
    }
}

impl Consumer<Box<dyn AsyncRead>> for SetTokenizerReader {
    fn accept(&mut self, reader: Box<dyn AsyncRead>) {
        self.tokenizer.set_reader(reader)
    }
}
