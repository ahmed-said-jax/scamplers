use uuid::Uuid;

// Type parametrization here so we can use the same trait multiple times for the same struct, since tables are
// frequently children of more than one table
pub trait Child<T: diesel::Table> {
    fn set_parent_id(&mut self, parent_id: Uuid);
}

pub trait Children<T, U>
where
    T: Child<U>,
    U: diesel::Table,
{
    fn set_parent_ids(&mut self, parent_ids: &[Uuid]);
}

impl<T, U> Children<T, U> for Vec<T>
where
    T: Child<U>,
    U: diesel::Table,
{
    fn set_parent_ids(&mut self, parent_ids: &[Uuid]) {
        for (item, parent_id) in self.iter_mut().zip(parent_ids) {
            item.set_parent_id(*parent_id);
        }
    }
}

pub trait ChildrenSets<T, U, V>: IntoIterator<Item = T> + Sized
where
    T: IntoIterator<Item = U>,
    U: Child<V>,
    V: diesel::Table,
{
    fn flatten_and_set_parent_ids(self, parent_ids: &[Uuid], n_children: usize) -> Vec<U> {
        let mut flattened_children = Vec::with_capacity(n_children);

        for (children, parent_id) in self.into_iter().zip(parent_ids) {
            for mut child in children {
                child.set_parent_id(*parent_id);
                flattened_children.push(child);
            }
        }

        flattened_children
    }
}
impl<T, U, V> ChildrenSets<T, U, V> for Vec<T>
where
    T: IntoIterator<Item = U>,
    U: Child<V>,
    V: diesel::Table,
{
}

pub trait MappingStruct: Sized {
    fn new(id1: Uuid, id2: Uuid) -> Self;
    fn from_grouped_ids<I1, I2, I3, U>(parents: I1, children_sets: I2, n_relationships: usize) -> Vec<Self>
    where
        I1: IntoIterator<Item = U>,
        I2: IntoIterator<Item = I3>,
        I3: IntoIterator<Item = U>,
        U: AsRef<Uuid>,
    {
        let mut mapping_structs = Vec::with_capacity(n_relationships);

        for (parent_id, children) in parents.into_iter().zip(children_sets) {
            for child_id in children {
                mapping_structs.push(Self::new(*parent_id.as_ref(), *child_id.as_ref()))
            }
        }

        mapping_structs
    }
}

// pub trait ChildrenSets2<T, U> where T: Child<U>, U: diesel::Table {
//     fn flatten_and_set_parent_ids(self, parent_ids: &[Uuid], n_children_per_parent: usize) -> Vec<T> ;
// }

// impl<T, U> ChildrenSets<T, U> for Vec<Vec<T>> where T: Child<U>, U: diesel::Table {
//     fn flatten_and_set_parent_ids(self, parent_ids: &[Uuid], n_children_per_parent: usize) -> Vec<T> {
//         let mut flattened_children = Vec::with_capacity(n_children_per_parent * self.len());

//         for (children, parent_id) in self.into_iter().zip(parent_ids) {
//             for mut child in children {
//                 child.set_parent_id(*parent_id);
//                 flattened_children.push(child);
//             }
//         }

//         flattened_children
//     }
// }

// impl <T, U> ChildrenSets<T, U> for Vec<&Vec<T>> where T: Child<U>, U: diesel::Table {
//     fn flatten_and_set_parent_ids(self, parent_ids: &[Uuid], n_children_per_parent: usize) -> Vec<T> {
//         let mut flattened_children = Vec::with_capacity(n_children_per_parent * self.len());

//         for (children, parent_id) in self.into_iter().zip(parent_ids) {
//             for mut child in children {
//                 child.set_parent_id(*parent_id);
//                 flattened_children.push(child);
//             }
//         }

//         flattened_children
//     }
// }
