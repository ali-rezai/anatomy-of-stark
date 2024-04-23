use crate::{consts::*, element::FieldElement, xgcd};
use primitive_types::U256;
use serde::{
    de,
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};
use std::fmt;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Field {
    pub p: U256,
}

impl Field {
    pub fn new(p: U256) -> Self {
        Field { p }
    }

    pub fn zero(&self) -> FieldElement {
        FieldElement {
            value: ZERO,
            field: *self,
        }
    }

    pub fn one(&self) -> FieldElement {
        FieldElement {
            value: ONE,
            field: *self,
        }
    }

    pub fn generator(&self) -> FieldElement {
        assert!(self.p == *PRIME);
        return FieldElement::new(*GENERATOR, *self);
    }

    pub fn primitive_nth_root(&self, n: U256) -> FieldElement {
        assert!(self.p == *PRIME);
        assert!(n <= (1u128 << 119).into() && n & (n - 1) == ZERO);
        let mut root = self.generator();
        let mut order: U256 = (1u128 << 119).into();
        while order != n {
            root = &root ^ *TWO;
            order = order >> 1;
        }
        root
    }

    pub fn sample(&self, byte_array: &[u8]) -> FieldElement {
        let mut acc: U256 = ZERO;
        byte_array.iter().for_each(|b| {
            acc = (acc << 8) ^ (*b).into();
        });
        FieldElement::new(acc % self.p, *self)
    }

    pub fn add(&self, left: &FieldElement, right: &FieldElement) -> FieldElement {
        FieldElement {
            value: (left.value + right.value) % self.p,
            field: *self,
        }
    }
    pub fn sub(&self, left: &FieldElement, right: &FieldElement) -> FieldElement {
        FieldElement {
            value: (self.p + left.value - right.value) % self.p,
            field: *self,
        }
    }
    pub fn mul(&self, left: &FieldElement, right: &FieldElement) -> FieldElement {
        FieldElement {
            value: (left.value * right.value) % self.p,
            field: *self,
        }
    }
    pub fn div(&self, left: &FieldElement, right: &FieldElement) -> FieldElement {
        assert!(right.value != ZERO);
        let (a, _, _, a_neg, _) = xgcd(right.value, self.p);
        FieldElement {
            value: if a_neg {
                self.p - (left.value * a) % self.p
            } else {
                left.value * a
            } % self.p,
            field: *self,
        }
    }
    pub fn neg(&self, operand: &FieldElement) -> FieldElement {
        FieldElement {
            value: (self.p - operand.value) % self.p,
            field: *self,
        }
    }

    pub fn inv(&self, operand: &FieldElement) -> FieldElement {
        let (a, _, _, a_neg, _) = xgcd(operand.value, self.p);
        FieldElement {
            value: if a_neg { self.p - a } else { a } % self.p,
            field: *self,
        }
    }
}

impl Serialize for Field {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Field", 4)?;
        s.serialize_field("llow", &((self.p).low_u64() as i64))?;
        s.serialize_field("hlow", &((self.p >> 64).low_u64() as i64))?;
        s.serialize_field("lhigh", &((self.p >> 128).low_u64() as i64))?;
        s.serialize_field("hhigh", &((self.p >> 192).low_u64() as i64))?;

        s.end()
    }
}

impl<'de> Deserialize<'de> for Field {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Fields {
            LLOW,
            HLOW,
            LHIGH,
            HHIGH,
        }

        struct FieldVisitor;
        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Field struct")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Field, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut llow: Option<u64> = None;
                let mut hlow: Option<u64> = None;
                let mut lhigh: Option<u64> = None;
                let mut hhigh: Option<u64> = None;

                while let Some(key) = map.next_key()? {
                    match key {
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

                let mut p: U256 = llow.ok_or_else(|| de::Error::missing_field("llow"))?.into();
                let hlow: U256 = hlow.ok_or_else(|| de::Error::missing_field("hlow"))?.into();
                let lhigh: U256 = lhigh
                    .ok_or_else(|| de::Error::missing_field("lhigh"))?
                    .into();
                let hhigh: U256 = hhigh
                    .ok_or_else(|| de::Error::missing_field("hhigh"))?
                    .into();

                p = p | (hlow << 64);
                p = p | (lhigh << 128);
                p = p | (hhigh << 192);

                Ok(Field { p })
            }
        }

        const FIELDS: &[&str] = &["llow", "hlow", "lhigh", "hhigh"];
        deserializer.deserialize_struct("Field", FIELDS, FieldVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_test() {
        let f = Field::new(*PRIME);
        assert_eq!(f.p, *PRIME);

        let root = f.primitive_nth_root((1u128 << 119).into());
        assert_eq!(root.value, *GENERATOR);

        let root = f.primitive_nth_root((1u128 << 118).into());
        assert_eq!(root.value, *GENERATOR * *GENERATOR % *PRIME);

        let root = f.primitive_nth_root((1u128 << 117).into());
        assert_eq!(
            root.value,
            (*GENERATOR * *GENERATOR % *PRIME) * (*GENERATOR * *GENERATOR % *PRIME) % *PRIME
        );

        let gen = f.generator();
        assert_eq!(gen.value, *GENERATOR);

        let s = f.sample(&[1u8, 2u8, 3u8]);
        assert_eq!(s.value, 66051.into());
    }

    #[test]
    fn serialization_test() {
        let f = Field::new(*PRIME);
        let serialized = serde_pickle::to_vec(&f, Default::default()).unwrap();
        let deserialized: Field =
            serde_pickle::from_slice(&serialized, Default::default()).unwrap();
        assert_eq!(f, deserialized);
    }
}
