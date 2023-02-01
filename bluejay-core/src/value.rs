use crate::Variable;
use std::collections::HashMap;

pub trait ObjectValue<V> {
    type Key: AsRef<str>;

    fn fields(&self) -> &[(Self::Key, V)];
}

pub trait IntegerValue {
    fn to_i32(&self) -> i32;
}

impl IntegerValue for i32 {
    fn to_i32(&self) -> i32 {
        *self
    }
}

pub trait FloatValue {
    fn to_f64(&self) -> f64;
}

impl FloatValue for f64 {
    fn to_f64(&self) -> f64 {
        *self
    }
}

pub trait StringValue: AsRef<str> {
    fn contains_enum_values() -> bool {
        false
    }
}

impl StringValue for String {
    fn contains_enum_values() -> bool {
        true
    }
}

pub trait BooleanValue {
    fn to_bool(&self) -> bool;
}

impl BooleanValue for bool {
    fn to_bool(&self) -> bool {
        *self
    }
}

pub trait EnumValue: AsRef<str> {}

impl EnumValue for String {}

pub trait ListValue<V>: AsRef<[V]> {}

pub trait AbstractValue<const CONST: bool>:
    Into<
        Value<
            CONST,
            Self::Variable,
            Self::Integer,
            Self::Float,
            Self::String,
            Self::Boolean,
            Self::Null,
            Self::Enum,
            Self::List,
            Self::Object,
        >,
    > + AsRef<
        Value<
            CONST,
            Self::Variable,
            Self::Integer,
            Self::Float,
            Self::String,
            Self::Boolean,
            Self::Null,
            Self::Enum,
            Self::List,
            Self::Object,
        >,
    >
{
    type Variable: Variable;
    type Integer: IntegerValue;
    type Float: FloatValue;
    type String: StringValue;
    type Boolean: BooleanValue;
    type Null;
    type Enum: EnumValue;
    type List: ListValue<
        Value<
            CONST,
            Self::Variable,
            Self::Integer,
            Self::Float,
            Self::String,
            Self::Boolean,
            Self::Null,
            Self::Enum,
            Self::List,
            Self::Object,
        >,
    >;
    type Object: ObjectValue<
        Value<
            CONST,
            Self::Variable,
            Self::Integer,
            Self::Float,
            Self::String,
            Self::Boolean,
            Self::Null,
            Self::Enum,
            Self::List,
            Self::Object,
        >,
    >;
}

pub trait AbstractConstValue: AbstractValue<true> {}
pub trait AbstractVariableValue: AbstractValue<false> {}

#[derive(Debug, Clone)]
pub enum Value<
    const CONST: bool,
    V: Variable,
    I: IntegerValue,
    F: FloatValue,
    S: StringValue,
    B: BooleanValue,
    N,
    E: EnumValue,
    L: ListValue<Self>,
    O: ObjectValue<Self>,
> {
    Variable(V),
    Integer(I),
    Float(F),
    String(S),
    Boolean(B),
    Null(N),
    Enum(E),
    List(L),
    Object(O),
}

impl<
        const CONST: bool,
        V: Variable,
        I: IntegerValue,
        F: FloatValue,
        S: StringValue,
        B: BooleanValue,
        N,
        E: EnumValue,
        L: ListValue<Self>,
        O: ObjectValue<Self>,
    > AsRef<Value<CONST, V, I, F, S, B, N, E, L, O>> for Value<CONST, V, I, F, S, B, N, E, L, O>
{
    fn as_ref(&self) -> &Value<CONST, V, I, F, S, B, N, E, L, O> {
        self
    }
}

impl<
        const CONST: bool,
        V: Variable,
        I: IntegerValue,
        F: FloatValue,
        S: StringValue,
        B: BooleanValue,
        N,
        E: EnumValue,
        L: ListValue<Self>,
        O: ObjectValue<Self>,
    > std::cmp::PartialEq for Value<CONST, V, I, F, S, B, N, E, L, O>
{
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Variable(v) => {
                matches!(other, Self::Variable(other_v) if v.name() == other_v.name())
            }
            Self::Integer(i) => {
                matches!(other, Self::Integer(other_i) if i.to_i32() == other_i.to_i32())
            }
            Self::Float(f) => {
                matches!(other, Self::Float(other_f) if f.to_f64() == other_f.to_f64())
            }
            Self::String(s) => {
                matches!(other, Self::String(other_s) if s.as_ref() == other_s.as_ref())
            }
            Self::Boolean(b) => {
                matches!(other, Self::Boolean(other_b) if b.to_bool() == other_b.to_bool())
            }
            Self::Null(_) => matches!(other, Self::Null(_)),
            Self::Enum(e) => matches!(other, Self::Enum(other_e) if e.as_ref() == other_e.as_ref()),
            Self::List(l) => matches!(other, Self::List(other_l) if l.as_ref() == other_l.as_ref()),
            Self::Object(o) => matches!(other, Self::Object(other_o) if {
                let lhs: HashMap<&str, &Self> = HashMap::from_iter(o.fields().iter().map(|(k, v)| (k.as_ref(), v)));
                let rhs: HashMap<&str, &Self> = HashMap::from_iter(other_o.fields().iter().map(|(k, v)| (k.as_ref(), v)));
                lhs == rhs
            }),
        }
    }
}

impl<
        const CONST: bool,
        V: Variable,
        I: IntegerValue,
        F: FloatValue,
        S: StringValue,
        B: BooleanValue,
        N,
        E: EnumValue,
        L: ListValue<Self>,
        O: ObjectValue<Self>,
    > AbstractValue<CONST> for Value<CONST, V, I, F, S, B, N, E, L, O>
{
    type Variable = V;
    type Integer = I;
    type Float = F;
    type String = S;
    type Boolean = B;
    type Null = N;
    type Enum = E;
    type List = L;
    type Object = O;
}

impl<T: AbstractValue<true>> AbstractConstValue for T {}
impl<T: AbstractValue<false>> AbstractVariableValue for T {}
