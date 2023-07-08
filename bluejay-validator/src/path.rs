use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathElement<'a> {
    Key(&'a str),
    Index(usize),
}

impl<'a> From<&'a str> for PathElement<'a> {
    fn from(s: &'a str) -> Self {
        Self::Key(s)
    }
}

impl From<usize> for PathElement<'_> {
    fn from(i: usize) -> Self {
        Self::Index(i)
    }
}

impl std::fmt::Display for PathElement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathElement::Key(s) => write!(f, "{}", s),
            PathElement::Index(i) => write!(f, "{}", i),
        }
    }
}

impl From<PathElement<'_>> for String {
    fn from(value: PathElement<'_>) -> Self {
        value.to_string()
    }
}

#[derive(Debug, PartialEq)]
struct PathInner<'a> {
    element: PathElement<'a>,
    parent: Option<Rc<Self>>,
}

impl<'a> PathInner<'a> {
    fn new(element: impl Into<PathElement<'a>>) -> Self {
        Self {
            element: element.into(),
            parent: None,
        }
    }

    fn push(rc_self: Rc<Self>, element: impl Into<PathElement<'a>>) -> Rc<Self> {
        Rc::new(Self {
            element: element.into(),
            parent: Some(rc_self),
        })
    }

    fn len(&self) -> usize {
        1 + self.parent.as_ref().map_or(0, |p| p.len())
    }

    fn append_to_vec<T: From<PathElement<'a>>>(&self, vec: &mut Vec<T>) {
        if let Some(parent) = &self.parent {
            parent.append_to_vec(vec);
        }

        vec.push(self.element.into());
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Path<'a>(Option<Rc<PathInner<'a>>>);

impl<'a> Path<'a> {
    pub fn new(element: impl Into<PathElement<'a>>) -> Self {
        Self(Some(Rc::new(PathInner::new(element))))
    }

    pub fn push(&self, element: impl Into<PathElement<'a>>) -> Self {
        match &self.0 {
            Some(inner) => Self(Some(PathInner::push(inner.clone(), element))),
            None => Self(Some(Rc::new(PathInner::new(element)))),
        }
    }

    fn len(&self) -> usize {
        self.0.as_ref().map_or(0, |i| i.len())
    }

    pub fn to_vec<T: From<PathElement<'a>>>(&self) -> Vec<T> {
        let mut vec = Vec::with_capacity(self.len());

        if let Some(inner) = &self.0 {
            inner.append_to_vec(&mut vec);
        }

        vec
    }
}

#[cfg(test)]
mod tests {
    use super::{Path, PathElement};

    #[test]
    fn test_len() {
        assert_eq!(0, Path::default().len());
        assert_eq!(1, Path::new("key").len());
        assert_eq!(2, Path::new("key").push("nested_key").len());
    }

    #[test]
    fn test_to_vec() {
        assert_eq!(Vec::<PathElement>::new(), Path::default().to_vec());
        assert_eq!(vec![PathElement::Key("key")], Path::new("key").to_vec());
        assert_eq!(
            vec![PathElement::Key("key"), PathElement::Key("nested_key")],
            Path::new("key").push("nested_key").to_vec(),
        );
    }

    #[test]
    fn test_partial_eq() {
        assert_ne!(Path::new("key").push("nested_key"), Path::new("nested_key"));
        assert_ne!(Path::new("key").push("nested_key"), Path::new("key"));
        assert_eq!(
            Path::new("key").push("nested_key"),
            Path::new("key").push("nested_key"),
        );
    }
}
