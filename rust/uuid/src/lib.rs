use {
    _uuid::Bytes,
    serde::{Deserialize, Serialize},
    std::{fmt::Display, str::FromStr},
};

#[cfg(feature = "backend")]
use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types};

#[cfg_attr(feature = "backend", derive(FromSqlRow, AsExpression))]
#[cfg_attr(feature = "backend", diesel(sql_type = sql_types::Uuid))]
#[derive(
    Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Debug, Hash, Default, Deserialize, Serialize,
)]
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

#[cfg(feature = "typescript")]
mod typescript {
    use std::str::FromStr;
    use wasm_bindgen::{
        JsValue,
        convert::{
            FromWasmAbi, IntoWasmAbi, OptionFromWasmAbi, OptionIntoWasmAbi, TryFromJsValue,
            VectorFromWasmAbi, VectorIntoWasmAbi, js_value_vector_from_abi,
            js_value_vector_into_abi,
        },
        describe::{WasmDescribe, WasmDescribeVector},
    };

    impl WasmDescribe for super::Uuid {
        fn describe() {
            String::describe()
        }
    }

    impl Into<JsValue> for super::Uuid {
        fn into(self) -> JsValue {
            self.to_string().into()
        }
    }

    impl TryFromJsValue for super::Uuid {
        type Error = _uuid::Error;

        fn try_from_js_value(value: JsValue) -> Result<Self, Self::Error> {
            Ok(Self::from_str(&String::try_from_js_value(value).unwrap())?)
        }
    }

    impl IntoWasmAbi for super::Uuid {
        type Abi = <String as IntoWasmAbi>::Abi;

        fn into_abi(self) -> Self::Abi {
            self.to_string().into_abi()
        }
    }

    impl FromWasmAbi for super::Uuid {
        type Abi = <Self as IntoWasmAbi>::Abi;

        unsafe fn from_abi(js: Self::Abi) -> Self {
            Self(unsafe { String::from_abi(js).parse().unwrap() })
        }
    }

    impl OptionIntoWasmAbi for super::Uuid {
        fn none() -> Self::Abi {
            <String as OptionIntoWasmAbi>::none()
        }
    }

    impl OptionFromWasmAbi for super::Uuid {
        fn is_none(abi: &Self::Abi) -> bool {
            <String as OptionFromWasmAbi>::is_none(abi)
        }
    }

    impl WasmDescribeVector for super::Uuid {
        fn describe_vector() {
            Vec::<String>::describe()
        }
    }

    impl VectorIntoWasmAbi for super::Uuid {
        type Abi = <String as VectorIntoWasmAbi>::Abi;

        fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
            js_value_vector_into_abi(vector)
        }
    }

    impl VectorFromWasmAbi for super::Uuid {
        type Abi = <String as VectorFromWasmAbi>::Abi;

        unsafe fn vector_from_abi(js: Self::Abi) -> Box<[Self]> {
            js_value_vector_from_abi(js)
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
