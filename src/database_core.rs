use heapless::Vec;
use mutex_traits::{ConstInit, ScopedRawMutex};

use crate::{
    AllVariants, DataFieldAccessor, DatabaseDescription, DynamicKeySet, FocusConstraints,
    FocusHandler, FolderFocus, FolderHandler, KeySet, Pair, PathConstraints, SubscriberData, ToKey,
    VariantCount, focus_handler, mutex::ScopedLocked,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseError {
    LockFail,
    SubscriberOverflow,
    ParameterCountMissmatch,
}

pub(crate) struct InternalMutable<
    Database: DatabaseDescription,
    const ABS_PARAMETER_COUNT: usize,
    const FLAT_PARAMETER_COUNT: usize,
> {
    pub(crate) data: Database::Data,
}

#[derive(PartialEq, Eq)]
enum SetState {
    Updated,
    UpToDate,
}

impl<
    Database: DatabaseDescription,
    const ABS_PARAMETER_COUNT: usize,
    const FLAT_PARAMETER_COUNT: usize,
> InternalMutable<Database, ABS_PARAMETER_COUNT, FLAT_PARAMETER_COUNT>
{
    pub(crate) const fn new(data: Database::Data) -> Self {
        Self { data }
    }

    pub(crate) fn clone<Focus>(
        &mut self,
        other: &Self,
        focus_handler: &Focus,
    ) -> Vec<Database::FlatKey, FLAT_PARAMETER_COUNT>
    where
        Focus: FocusHandler<Database>,
    {
        let mut changed_key_set: KeySet<Database::AbsKey, Database::FlatKey, FLAT_PARAMETER_COUNT> =
            KeySet::new();
        for abs_key in <Database::AbsKey as AllVariants>::ALL_VARIANTS.iter() {
            let this_abs_field = self.data.get(*abs_key);
            let other_abs_field = other.data.get(*abs_key);

            if this_abs_field != other_abs_field {
                self.data.set(other_abs_field);
                let focused_abs_key = focus_handler.get_focus_key((*abs_key).into());
                if focused_abs_key == *abs_key {
                    changed_key_set.insert_flat_key((*abs_key).into()).unwrap()
                }
            }
        }

        changed_key_set.to_vector()
    }

    pub(crate) fn get_absolute(&self, key: Database::AbsKey) -> Database::FlatField {
        self.data.get(key).into()
    }

    pub(crate) fn get_flat<Focus: FocusHandler<Database>>(
        &self,
        focus_handler: &Focus,
        key: Database::FlatKey,
    ) -> Database::FlatField {
        let abs_key = focus_handler.get_focus_key(key);
        self.get_absolute(abs_key)
    }

    pub(crate) fn set_absolute<Focus: FocusHandler<Database>>(
        &mut self,
        focus_handler: &Focus,
        abs_field: Database::AbsField,
    ) -> SetState {
        let abs_key: Database::AbsKey = abs_field.to_key();
        let flat_field: Database::FlatField = abs_field.clone().into();
        let flat_key: Database::FlatKey = flat_field.to_key();

        if self.data.get(abs_key) != abs_field {
            self.data.set(abs_field);

            // If a parameter was set in a non-focused folder, the notifying should be delayed
            // until that folder becomes focused
            if focus_handler.get_focus_key(flat_key) != abs_key {
                SetState::UpToDate
            } else {
                SetState::Updated
            }
        } else {
            SetState::UpToDate
        }
    }

    pub(crate) fn set_flat<Focus: FocusHandler<Database>>(
        &mut self,
        focus_handler: &Focus,
        flat_field: Database::FlatField,
    ) -> SetState {
        let abs_field: Database::AbsField = focus_handler.get_focus_field(flat_field);
        let abs_key: Database::AbsKey = abs_field.to_key();

        if self.data.get(abs_key) != abs_field {
            self.data.set(abs_field);

            // Note that as the changed value was in the focused path, there is always a change
            SetState::Updated
        } else {
            SetState::UpToDate
        }
    }

    pub fn on_focus_change<FocusType, Path, InternalAbsPair>(
        &self,
        lhs: Path,
        rhs: Path,
    ) -> Result<Vec<Database::FlatKey, FLAT_PARAMETER_COUNT>, DatabaseError>
    where
        FocusType: FocusConstraints,
        Path: PathConstraints<(Database::AbsKey, Database::AbsField), InternalAbsPair>,
        InternalAbsPair: Pair,
        Database::Data: FolderHandler<Path, Database>,
    {
        let mut differing_keys: KeySet<Database::AbsKey, Database::FlatKey, FLAT_PARAMETER_COUNT> =
            KeySet::new();

        let other: &<Database::Data as FolderHandler<Path, Database>>::Content = self.data.get_at(rhs);
        match self.data.compare(&mut differing_keys, lhs, other) {
            Ok(()) => Ok(differing_keys.to_vector()),
            Err(error) => Err(error),
        }
    }
}

pub(crate) struct DatabaseCore<
    'a,
    Focus: FocusHandler<Database>,
    Mutex: ScopedRawMutex + ConstInit,
    Database: DatabaseDescription,
    const ABS_PARAMETER_COUNT: usize,
    const FLAT_PARAMETER_COUNT: usize,
> {
    pub(crate) data:
        ScopedLocked<Mutex, InternalMutable<Database, ABS_PARAMETER_COUNT, FLAT_PARAMETER_COUNT>>,
    pub(crate) subscribers: SubscriberData<'a, Mutex, Database, FLAT_PARAMETER_COUNT>,
    pub(crate) focus_handler: Focus,
}

impl<
    'a,
    Focus: FocusHandler<Database>,
    Mutex: ScopedRawMutex + ConstInit,
    Database: DatabaseDescription,
    const ABS_PARAMETER_COUNT: usize,
    const FLAT_PARAMETER_COUNT: usize,
> DatabaseCore<'a, Focus, Mutex, Database, ABS_PARAMETER_COUNT, FLAT_PARAMETER_COUNT>
{
    const _STATIC_ASSERTIONS: () = {
        assert!(ABS_PARAMETER_COUNT == <Database::AbsKey as VariantCount>::COUNT);
        assert!(ABS_PARAMETER_COUNT == <Database::AbsField as VariantCount>::COUNT);
        assert!(FLAT_PARAMETER_COUNT == <Database::FlatKey as VariantCount>::COUNT);
        assert!(FLAT_PARAMETER_COUNT == <Database::FlatField as VariantCount>::COUNT);
    };

    pub(crate) const fn new(data: Database::Data, focus_handler: Focus) -> Self {
        Self {
            focus_handler,
            data: ScopedLocked::new(InternalMutable::new(data)),
            subscribers: SubscriberData::new(),
        }
    }

    pub(crate) fn get_absolute(
        &self,
        key: Database::AbsKey,
    ) -> Result<Database::FlatField, DatabaseError> {
        match self.data.try_with(|internal| internal.get_absolute(key)) {
            Some(field) => Ok(field),
            None => Err(DatabaseError::LockFail),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get_flat(
        &self,
        key: Database::FlatKey,
    ) -> Result<Database::FlatField, DatabaseError> {
        match self
            .data
            .try_with(|internal| internal.get_flat(&self.focus_handler, key))
        {
            Some(field) => Ok(field),
            None => Err(DatabaseError::LockFail),
        }
    }

    pub(crate) fn set_absolute(&self, abs_field: Database::AbsField) -> Result<(), DatabaseError> {
        let abs_key = abs_field.to_key();

        match self
            .data
            .try_with(|internal| internal.set_absolute(&self.focus_handler, abs_field))
        {
            None => Err(DatabaseError::LockFail),
            Some(set_state) => {
                if set_state == SetState::Updated {
                    self.subscribers.on_change(abs_key.into());
                }
                Ok(())
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn set_flat(&self, flat_field: Database::FlatField) -> Result<(), DatabaseError> {
        let flat_key = flat_field.to_key();
        match self
            .data
            .try_with(|internal| internal.set_flat(&self.focus_handler, flat_field))
        {
            None => Err(DatabaseError::LockFail),
            Some(set_state) => {
                if set_state == SetState::Updated {
                    self.subscribers.on_change(flat_key);
                }
                Ok(())
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_focus<FocusType, Path, AbsPair, InternalAbsPair>(&self) -> FocusType
    where
        FocusType: FocusConstraints,
        Path: PathConstraints<(Database::AbsKey, Database::AbsField), InternalAbsPair>,
        InternalAbsPair: Pair,
        Focus:
            FolderFocus<FocusType, Path, (Database::AbsKey, Database::AbsField), InternalAbsPair>,
        Database::Data: FolderHandler<Path, Database>,
    {
        self.focus_handler.get_focus()
    }

    #[allow(dead_code)]
    pub fn set_focus<FocusType, Path, InternalAbsPair>(
        &self,
        focus: FocusType,
    ) -> Result<(), DatabaseError>
    where
        FocusType: FocusConstraints,
        Path: PathConstraints<(Database::AbsKey, Database::AbsField), InternalAbsPair>,
        InternalAbsPair: Pair,
        Focus:
            FolderFocus<FocusType, Path, (Database::AbsKey, Database::AbsField), InternalAbsPair>,
        Database::Data: FolderHandler<Path, Database>,
    {
        let previous_focus_path = self.focus_handler.get_focus_path();
        self.focus_handler.set_focus(focus);
        let next_focus_path = self.focus_handler.get_focus_path();

        // If the active focus changes, then calculate a list of changed keys by comparing the two
        // paths
        if previous_focus_path != next_focus_path {
            match self.data.try_with(|internal| {
                internal.on_focus_change::<FocusType, Path, InternalAbsPair>(
                    next_focus_path,
                    previous_focus_path,
                )
            }) {
                Some(result) => match result {
                    Err(error) => Err(error),
                    Ok(changes) => {
                        // Notify subscribers on all changes as if the parameters where changed
                        if changes.len() > 0 {
                            self.subscribers.on_changes(&changes);
                        }

                        Ok(())
                    }
                },
                None => Err(DatabaseError::LockFail),
            }
        } else {
            Ok(())
        }
    }

    pub fn clone(&self, other: &Self) -> Result<(), DatabaseError> {
        match self.data.try_with(|internal| {
            other
                .data
                .try_with(|other_internal| internal.clone(&other_internal, &self.focus_handler))
        }) {
            None => Err(DatabaseError::LockFail),
            Some(changes) => match changes {
                None => Err(DatabaseError::LockFail),
                Some(changes) => {
                    self.subscribers.on_changes(&changes);
                    Ok(())
                }
            },
        }
    }
}
