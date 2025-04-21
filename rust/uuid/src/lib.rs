use {
    _uuid::Bytes,
    std::{fmt::Display, str::FromStr},
};

#[cfg(feature = "backend")]
use {
    diesel::{
        deserialize::{FromSql, FromSqlRow},
        expression::AsExpression,
        sql_types,
    },
    serde::{Deserialize, Serialize},
};

#[cfg_attr(
    feature = "backend",
    derive(Deserialize, Serialize, FromSqlRow, AsExpression)
)]
#[cfg_attr(feature = "backend", diesel(sql_type = sql_types::Uuid))]
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Debug, Hash)]
pub struct Uuid(_uuid::Uuid);

impl Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        inner.fmt(f)
    }
}

impl FromStr for Uuid {
    type Err = <_uuid::Uuid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(_uuid::Uuid::from_str(s)?))
    }
}

impl Uuid {
    pub fn as_bytes(&self) -> &Bytes {
        let Self(inner) = self;
        inner.as_bytes()
    }

    #[cfg(feature = "backend")]
    pub fn now_v7() -> Self {
        Self(_uuid::Uuid::now_v7())
    }
}

#[cfg(feature = "web")]
mod web {
    use wasm_bindgen::{
        convert::{FromWasmAbi, IntoWasmAbi, OptionFromWasmAbi, OptionIntoWasmAbi},
        describe::WasmDescribe,
    };

    impl WasmDescribe for super::Uuid {
        fn describe() {
            String::describe()
        }
    }

    impl IntoWasmAbi for super::Uuid {
        type Abi = <Vec<u8> as IntoWasmAbi>::Abi;

        fn into_abi(self) -> Self::Abi {
            self.to_string().into_abi()
        }
    }

    impl FromWasmAbi for super::Uuid {
        type Abi = <Self as IntoWasmAbi>::Abi;

        unsafe fn from_abi(js: Self::Abi) -> Self {
            Self(unsafe {String::from_abi(js).parse().unwrap_or_default()})
        }
    }

    impl OptionFromWasmAbi for super::Uuid {
        fn is_none(abi: &Self::Abi) -> bool {
            <String as OptionFromWasmAbi>::is_none(abi)
        }
    }

    impl OptionIntoWasmAbi for super::Uuid {
        fn none() -> Self::Abi {
            <String as OptionIntoWasmAbi>::none()
        }
    }
}

#[cfg(feature = "backend")]
mod backend {
    use {
        super::Uuid,
        diesel::{
            backend::Backend,
            deserialize::FromSql,
            pg::Pg,
            serialize::{Output, ToSql},
            sql_types,
        },
        valuable::{Valuable, Value},
    };

    impl Valuable for Uuid {
        fn as_value(&self) -> Value<'_> {
            self.as_bytes().as_value()
        }

        fn visit(&self, visit: &mut dyn valuable::Visit) {
            self.as_bytes().visit(visit)
        }
    }

    impl FromSql<sql_types::Uuid, Pg> for Uuid {
        fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
            Ok(Self(_uuid::Uuid::from_sql(bytes)?))
        }
    }

    impl ToSql<sql_types::Uuid, Pg> for Uuid {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
            let Self(inner) = self;
            <_uuid::Uuid as ToSql<sql_types::Uuid, Pg>>::to_sql(inner, out)
        }
    }
}
