use crate::{
    consts::{ONE, ZERO},
    field::Field,
};
use primitive_types::U256;
use serde::{
    de,
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};
use std::fmt;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct FieldElement {
    pub value: U256,
    pub field: Field,
}

impl FieldElement {
    pub fn new(value: U256, field: Field) -> Self {
        FieldElement { value, field }
    }

    pub fn inv(&self) -> FieldElement {
        self.field.inv(&self)
    }

    pub fn is_zero(&self) -> bool {
        self.value == ZERO
    }
}

impl std::ops::Add<&FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn add(self, rhs: &FieldElement) -> FieldElement {
        self.field.add(self, rhs)
    }
}

impl std::ops::Sub<&FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn sub(self, rhs: &FieldElement) -> FieldElement {
        self.field.sub(self, rhs)
    }
}

impl std::ops::Mul<&FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn mul(self, rhs: &FieldElement) -> FieldElement {
        self.field.mul(self, rhs)
    }
}

impl std::ops::Div<&FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn div(self, rhs: &FieldElement) -> FieldElement {
        self.field.div(self, rhs)
    }
}

impl std::ops::Neg for &FieldElement {
    type Output = FieldElement;

    fn neg(self) -> FieldElement {
        self.field.neg(self)
    }
}

impl std::ops::BitXor<U256> for &FieldElement {
    type Output = FieldElement;

    fn bitxor(self, rhs: U256) -> FieldElement {
        let mut acc = self.field.one();

        let mut i: U256 = 128.into();
        while i > ZERO {
            i -= ONE;
            if (rhs >> i) & ONE == ONE {
                break;
            }
        }

        i += ONE;
        while i > ZERO {
            i -= ONE;
            acc = &acc * &acc;
            if (ONE << i) & rhs != ZERO {
                acc = &acc * &self;
            }
        }

        acc
    }
}

impl Serialize for FieldElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("FieldElement", 5)?;
        s.serialize_field("field", &self.field)?;
        s.serialize_field("llow", &((self.value).low_u64() as i64))?;
        s.serialize_field("hlow", &((self.value >> 64).low_u64() as i64))?;
        s.serialize_field("lhigh", &((self.value >> 128).low_u64() as i64))?;
        s.serialize_field("hhigh", &((self.value >> 192).low_u64() as i64))?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for FieldElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Fields {
            FIELD,
            LLOW,
            HLOW,
            LHIGH,
            HHIGH,
        }

        struct FieldElementVisitor;
        impl<'de> Visitor<'de> for FieldElementVisitor {
            type Value = FieldElement;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("FieldElement struct")
            }

            fn visit_map<V>(self, mut map: V) -> Result<FieldElement, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut field: Option<Field> = None;
                let mut llow: Option<u64> = None;
                let mut hlow: Option<u64> = None;
                let mut lhigh: Option<u64> = None;
                let mut hhigh: Option<u64> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Fields::FIELD => {
                            if llow.is_some() {
                                return Err(de::Error::duplicate_field("field"));
                            }
                            field = Some(map.next_value()?);
                        }
                        Fields::LLOW => {
                            if llow.is_some() {
                                return Err(de::Error::duplicate_field("llow"));
                            }
                            let v: i64 = map.next_value()?;
                            llow = Some(v as u64);
                        }
                        Fields::HLOW => {
                            if hlow.is_some() {
                                return Err(de::Error::duplicate_field("hlow"));
                            }
                            let v: i64 = map.next_value()?;
                            hlow = Some(v as u64);
                        }
                        Fields::LHIGH => {
                            if lhigh.is_some() {
                                return Err(de::Error::duplicate_field("lhigh"));
                            }
                            let v: i64 = map.next_value()?;
                            lhigh = Some(v as u64);
                        }
                        Fields::HHIGH => {
                            if hhigh.is_some() {
                                return Err(de::Error::duplicate_field("hhigh"));
                            }
                            let v: i64 = map.next_value()?;
                            hhigh = Some(v as u64);
                        }
                    }
                }

                let field = field
                    .ok_or_else(|| de::Error::missing_field("field"))?
                    .into();
                let mut value: U256 = llow.ok_or_else(|| de::Error::missing_field("llow"))?.into();
                let hlow: U256 = hlow.ok_or_else(|| de::Error::missing_field("hlow"))?.into();
                let lhigh: U256 = lhigh
                    .ok_or_else(|| de::Error::missing_field("lhigh"))?
                    .into();
                let hhigh: U256 = hhigh
                    .ok_or_else(|| de::Error::missing_field("hhigh"))?
                    .into();

                value = value | (hlow << 64);
                value = value | (lhigh << 128);
                value = value | (hhigh << 192);

                Ok(FieldElement { value, field })
            }
        }

        const FIELDS: &[&str] = &["field", "llow", "hlow", "lhigh", "hhigh"];
        deserializer.deserialize_struct("FieldElement", FIELDS, FieldElementVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::PRIME;

    use super::*;

    #[test]
    fn element_test() {
        let f = Field::new(7.into());
        let e1 = FieldElement::new(ONE, f);
        let e2 = FieldElement::new(3.into(), f);
        let e3 = FieldElement::new(3.into(), f);
        assert_eq!(e1.field, e2.field);
        assert_eq!(e1.value, ONE);
        assert_eq!(e2.value, 3.into());
        assert_eq!(e2, e3);
        assert_ne!(e2, e1);
    }

    #[test]
    fn arithmetic_test() {
        let f = Field::new(7.into());
        let e1 = FieldElement::new(ONE, f);
        let e2 = FieldElement::new(3.into(), f);
        assert_eq!((&e1 + &e2).value, 4.into());
        assert_eq!((&e1 - &e2).value, 5.into());
        assert_eq!((&e1 * &e2).value, 3.into());
        assert_eq!((&e1 / &e2).value, 5.into());
        assert_eq!((-&e1).value, 6.into());
        assert_eq!((&e2.inv()).value, 5.into());
        assert_eq!((&e2 ^ 4.into()).value, 4.into());
        assert_eq!((&e2 ^ 2.into()).value, 2.into());
        assert_eq!((&e1 ^ 2.into()).value, 1.into());
    }

    #[test]
    fn serialization_test() {
        let f = Field::new(*PRIME);
        let serialized = serde_pickle::to_vec(&f.generator(), Default::default()).unwrap();
        let deserialized: FieldElement =
            serde_pickle::from_slice(&serialized, Default::default()).unwrap();
        assert_eq!(f.generator(), deserialized);
    }
}
