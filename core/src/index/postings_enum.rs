use {crate::search::doc_id_set_iterator::DocIdSetIterator, std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not}};


pub trait PostingsEnum: DocIdSetIterator {
    // FIXME: Add methods.
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PostingsEnumFlags(u16);

impl BitAnd for PostingsEnumFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for PostingsEnumFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for PostingsEnumFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for PostingsEnumFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Not for PostingsEnumFlags {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl PostingsEnumFlags {
    #[inline]
    pub fn contains(self, other: PostingsEnumFlags) -> bool {
        self.0 & other.0 == other.0
    }

    /// Flag to pass to [crate::index::terms_enum::TermsEnum::postings] if you don't require per-document
    /// postings in the returned enum.
    pub const None: PostingsEnumFlags = PostingsEnumFlags(0);

    /// Flag to pass to [crate::index::terms_enum::TermsEnum::postings] if you require term frequencies
    /// in the returned enum.
    pub const Freqs: PostingsEnumFlags = PostingsEnumFlags(1 << 3);

    /// Flag to pass to [crate::index::terms_enum::TermsEnum::postings] if you require term positions in
    /// the returned enum.
    pub const Positions: PostingsEnumFlags = PostingsEnumFlags(PostingsEnumFlags::Freqs.0 | 1 << 4);

    /// Flag to pass to [crate::index::terms_enum::TermsEnum::postings] if you require offsets in the
    /// returned enum.
    pub const Offsets: PostingsEnumFlags = PostingsEnumFlags(PostingsEnumFlags::Positions.0 | 1 << 5);

    /// Flag to pass to [crate::index::terms_enum::TermsEnum::postings] if you require payloads in the
    /// returned enum.
    pub const Payloads: PostingsEnumFlags = PostingsEnumFlags(PostingsEnumFlags::Positions.0 | 1 << 6);

    /// Flag to pass to [crate::index::terms_enum::TermsEnum::postings] to get positions, payloads and
    /// offsets in the returned enum
    pub const All: PostingsEnumFlags = PostingsEnumFlags(PostingsEnumFlags::Offsets.0 | PostingsEnumFlags::Payloads.0);
}
