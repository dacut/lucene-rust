/// Encapsulates all required internal state to position the associated {@link TermsEnum} without
/// re-seeking.
pub trait TermState { }

impl TermState for () { }