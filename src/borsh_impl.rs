use crate::Maybe;
use borsh_1::{io, BorshDeserialize, BorshSerialize};

impl<R: BorshSerialize, T: BorshSerialize> BorshSerialize for Maybe<R, T> {
  fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
    match self {
      Self::Ref(r) => r.serialize(writer),
      Self::Owned(t) => t.serialize(writer),
    }
  }
}

impl<R, T: BorshDeserialize> BorshDeserialize for Maybe<R, T> {
  fn deserialize_reader<B: io::Read>(reader: &mut B) -> io::Result<Self> {
    T::deserialize_reader(reader).map(Self::Owned)
  }
}
