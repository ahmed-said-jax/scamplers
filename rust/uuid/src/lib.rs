use {std::fmt::Display};

#[cfg(feature = "backend")]
use diesel::{deserialize::{FromSql, FromSqlRow}, expression::AsExpression, sql_types};

#[cfg_attr(feature = "python", pyclass(transparent))]
#[cfg_attr(feature = "backend", derive(Deserialize, Serialize, FromSqlRow, AsExpression))]
#[cfg_attr(feature = "backend", diesel(sql_type = sql_types::Uuid))]
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct Uuid(_uuid::Uuid);

impl Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        inner.fmt(f)
    }
}

#[cfg(feature = "web")]
mod web {
    use wasm_bindgen::{convert::{IntoWasmAbi, FromWasmAbi, OptionFromWasmAbi, OptionIntoWasmAbi}, describe::WasmDescribe};

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
            Self(String::from_abi(js).parse().unwrap_or_default())
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
    use {valuable::{Valuable, Value}, diesel::{deserialize::{FromSql}, serialize::{ToSql, Output}, sql_types}};

    impl Valuable for Uuid {
        fn as_value(&self) -> Value<'_> {
            self.to_string().as_value()
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
            inner.to_sql(out)
        }
    }
}
