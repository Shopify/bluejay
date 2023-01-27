pub trait AbstractTypeReference: Into<TypeReference<Self::NamedTypeReference, Self::ListTypeReference>> {
    type NamedTypeReference: NamedTypeReference;
    type ListTypeReference: ListTypeReference<NamedTypeReference=Self::NamedTypeReference>;
}

#[derive(Debug)]
pub enum TypeReference<NTR: NamedTypeReference, LTR: ListTypeReference<NamedTypeReference=NTR>> {
    NamedType(NTR),
    ListType(LTR),
}

impl<NTR: NamedTypeReference, LTR: ListTypeReference<NamedTypeReference=NTR>> TypeReference<NTR, LTR> {
    pub fn name(&self) -> &str {
        match self {
            Self::NamedType(ntr) => ntr.name(),
            Self::ListType(ltr) => ltr.inner().name(),
        }
    }
}

impl<NTR: NamedTypeReference, LTR: ListTypeReference<NamedTypeReference=NTR>> AbstractTypeReference for TypeReference<NTR, LTR> {
    type NamedTypeReference = NTR;
    type ListTypeReference = LTR;
}

pub trait NamedTypeReference {
    fn name(&self) -> &str;
    fn required(&self) -> bool;
}

pub trait ListTypeReference: Sized {
    type NamedTypeReference: NamedTypeReference;

    fn inner(&self) -> &TypeReference<Self::NamedTypeReference, Self>;
    fn required(&self) -> bool;
}
