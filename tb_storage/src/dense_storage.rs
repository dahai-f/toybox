use std::mem::MaybeUninit;

use hibitset::BitSet;

use tb_core::Id;

use crate::{util, Storage, StorageItems};

pub struct DenseStorageItems<D> {
    base_id: Option<Id>,
    indices: Vec<MaybeUninit<usize>>,
    data: Vec<D>,
    data_id: Vec<Id>,
}

pub type DenseStorage<D> = Storage<DenseStorageItems<D>>;

impl<D> Default for DenseStorageItems<D> {
    fn default() -> Self {
        Self {
            base_id: Default::default(),
            indices: Default::default(),
            data: Default::default(),
            data_id: Default::default(),
        }
    }
}

impl<D> StorageItems for DenseStorageItems<D> {
    type Data = D;

    unsafe fn clear(&mut self, _mask: &BitSet) {
        self.base_id = None;
        self.data.clear();
        self.data_id.set_len(0);
        self.indices.set_len(0);
    }

    unsafe fn insert(&mut self, id: Id, data: D) -> &mut D {
        let index_in_data = self.data.len();
        self.data.push(data);
        self.data_id.push(id);

        let index_in_indices =
            util::setup_index_with_base(&mut self.base_id, &mut self.indices, id);
        self.indices
            .get_unchecked_mut(index_in_indices)
            .as_mut_ptr()
            .write(index_in_data);
        self.data.get_unchecked_mut(index_in_data)
    }

    unsafe fn remove(&mut self, id: Id) {
        let index_in_indices = util::get_index_with_base(self.base_id, id);
        let index_in_data = self.indices.get_unchecked(index_in_indices).assume_init();
        let last_data_id = *self.data_id.last().unwrap();
        let last_data_index_in_indices = util::get_index_with_base(self.base_id, last_data_id);

        self.data.swap_remove(index_in_data);
        self.data_id.swap_remove(index_in_data);

        self.indices
            .get_unchecked_mut(last_data_index_in_indices)
            .as_mut_ptr()
            .write(index_in_data);
    }

    unsafe fn get(&self, id: Id) -> &D {
        let index = self
            .indices
            .get_unchecked(util::get_index_with_base(self.base_id, id))
            .assume_init();
        self.data.get_unchecked(index)
    }

    unsafe fn get_mut(&mut self, id: Id) -> &mut D {
        let index = self
            .indices
            .get_unchecked(util::get_index_with_base(self.base_id, id))
            .assume_init();
        self.data.get_unchecked_mut(index)
    }
}

#[cfg(test)]
mod tests {
    use testdrop::{Item, TestDrop};

    use tb_core::Id;

    use crate::DenseStorage;

    #[derive(Debug)]
    struct DenseStorageDropItemData<'a> {
        id: Id,
        td: &'a TestDrop,
        drop_item: Item<'a>,
    }

    impl<'a> DenseStorageDropItemData<'a> {
        fn new(id: impl Into<Id>, td: &'a TestDrop) -> Self {
            Self {
                id: id.into(),
                td,
                drop_item: td.new_item().1,
            }
        }
    }

    impl<'a> Drop for DenseStorageDropItemData<'a> {
        fn drop(&mut self) {
            println!("TestData in VecStorage dropped. id: {}", self.id);
        }
    }

    impl<'a> PartialEq for DenseStorageDropItemData<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    impl<'a> Eq for DenseStorageDropItemData<'a> {}

    #[test]
    fn drop() {
        let td = TestDrop::new();
        let mut storage: DenseStorage<DenseStorageDropItemData> = Default::default();
        let data_4 = DenseStorageDropItemData::new(4u32, &td);
        let data_3 = DenseStorageDropItemData::new(3u32, &td);
        let data_2 = DenseStorageDropItemData::new(2u32, &td);
        let data_8 = DenseStorageDropItemData::new(8u32, &td);
        let data_6 = DenseStorageDropItemData::new(6u32, &td);

        storage.insert(4u32.into(), data_4);
        storage.insert(3u32.into(), data_3);
        storage.insert(2u32.into(), data_2);
        storage.insert(8u32.into(), data_8);
        storage.insert(6u32.into(), data_6);

        storage.clear();
        assert_eq!(5, td.num_tracked_items());
        assert_eq!(5, td.num_dropped_items());
    }

    #[test]
    fn insert() {
        unsafe {
            let mut storage = DenseStorage::<i32>::default();
            assert_eq!(*storage.insert(3u32.into(), 3), 3);
            assert_eq!(storage.items.indices.len(), 1);
            assert_eq!(*storage.get(3u32.into()), 3);
            assert_eq!(*storage.get_mut(3u32.into()), 3);
            assert_eq!(*storage.insert(1u32.into(), 1), 1);
            assert_eq!(storage.items.indices.len(), 3);
            assert_eq!(storage.items.indices.get_unchecked_mut(0).assume_init(), 1);
            assert_eq!(storage.items.indices.get_unchecked_mut(2).assume_init(), 0);
            assert_eq!(*storage.get(1u32.into()), 1);
            assert_eq!(*storage.get(3u32.into()), 3);
            assert_eq!(*storage.insert(0u32.into(), 0), 0);
            assert_eq!(storage.items.indices.len(), 4);
            assert_eq!(*storage.get(1u32.into()), 1);
            assert_eq!(*storage.get(3u32.into()), 3);
            assert_eq!(*storage.get(0u32.into()), 0);
            assert!(storage.contains(3u32.into()));

            let mut storage = DenseStorage::<i32>::default();
            storage.insert(4u32.into(), 4);
            storage.insert(3u32.into(), 3);
            storage.insert(2u32.into(), 2);
            storage.insert(8u32.into(), 8);
            storage.insert(6u32.into(), 6);
            assert_eq!(storage.items.data, vec![4, 3, 2, 8, 6]);
            assert_eq!(
                storage.items.data_id,
                vec![
                    4u32.into(),
                    3u32.into(),
                    2u32.into(),
                    8u32.into(),
                    6u32.into()
                ]
            );
            assert_eq!(storage.items.base_id, Some(2u32.into()));
            assert_eq!(storage.items.indices[0].assume_init(), 2);
            assert_eq!(storage.items.indices[1].assume_init(), 1);
            assert_eq!(storage.items.indices[2].assume_init(), 0);
            assert_eq!(storage.items.indices[4].assume_init(), 4);
            assert_eq!(storage.items.indices[6].assume_init(), 3);
        }
    }

    #[test]
    fn remove() {
        unsafe {
            let mut storage = DenseStorage::<u32>::default();
            storage.insert(4u32.into(), 4);
            storage.insert(3u32.into(), 3);
            storage.insert(2u32.into(), 2);
            storage.insert(8u32.into(), 8);
            storage.insert(6u32.into(), 6);
            assert_eq!(storage.items.data, vec![4, 3, 2, 8, 6]);
            assert_eq!(
                storage.items.data_id,
                vec![
                    4u32.into(),
                    3u32.into(),
                    2u32.into(),
                    8u32.into(),
                    6u32.into()
                ]
            );
            assert_eq!(storage.items.base_id, Some(2u32.into()));
            assert_eq!(storage.items.indices[0].assume_init(), 2);
            assert_eq!(storage.items.indices[1].assume_init(), 1);
            assert_eq!(storage.items.indices[2].assume_init(), 0);
            assert_eq!(storage.items.indices[4].assume_init(), 4);
            assert_eq!(storage.items.indices[6].assume_init(), 3);
            assert!(storage.contains(3u32.into()));

            storage.remove(3u32.into());
            assert!(!storage.contains(3u32.into()));
            assert_eq!(storage.items.data, vec![4, 6, 2, 8]);
            assert_eq!(
                storage.items.data_id,
                vec![4u32.into(), 6u32.into(), 2u32.into(), 8u32.into()]
            );
            assert_eq!(storage.items.base_id, Some(2u32.into()));
            assert_eq!(storage.items.indices[0].assume_init(), 2);
            assert_eq!(storage.items.indices[2].assume_init(), 0);
            assert_eq!(storage.items.indices[4].assume_init(), 1);
            assert_eq!(storage.items.indices[6].assume_init(), 3);

            storage.remove(8u32.into());
            assert_eq!(storage.items.data, vec![4, 6, 2]);
            assert_eq!(
                storage.items.data_id,
                vec![4u32.into(), 6u32.into(), 2u32.into()]
            );
            assert_eq!(storage.items.base_id, Some(2u32.into()));
            assert_eq!(storage.items.indices[0].assume_init(), 2);
            assert_eq!(storage.items.indices[2].assume_init(), 0);
            assert_eq!(storage.items.indices[4].assume_init(), 1);
        }
    }
}
