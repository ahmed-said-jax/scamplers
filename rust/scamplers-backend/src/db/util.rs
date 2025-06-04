use diesel::{pg::Pg, prelude::*, sql_types};

pub(super) type BoxedDieselExpression<'a, Table> =
    Box<dyn BoxableExpression<Table, Pg, SqlType = sql_types::Bool> + 'a>;

pub(super) struct DieselExpressionBuilder<'a, Table>(Option<BoxedDieselExpression<'a, Table>>);
impl<Table> Default for DieselExpressionBuilder<'_, Table> {
    fn default() -> Self {
        Self(None)
    }
}

impl<'a, Table: 'a> DieselExpressionBuilder<'a, Table> {
    pub fn and<Q>(self, other: Q) -> Self
    where
        Q: BoxableExpression<Table, Pg, SqlType = sql_types::Bool> + 'a,
    {
        let other: BoxedDieselExpression<Table> = Box::new(other);

        let Self(Some(current)) = self else {
            return Self(Some(other));
        };

        let current = Box::new(current.and(other));

        Self(Some(current))
    }

    pub fn build(self) -> Option<BoxedDieselExpression<'a, Table>> {
        let Self(query) = self;

        query
    }
}

pub(super) trait AsIlike {
    fn as_ilike(&self) -> String;
}

impl AsIlike for &str {
    fn as_ilike(&self) -> String {
        format!("%{self}%")
    }
}

impl AsIlike for String {
    fn as_ilike(&self) -> String {
        self.as_str().as_ilike()
    }
}
