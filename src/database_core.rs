use heapless::Vec;
use mutex_traits::{ConstInit, ScopedRawMutex};

use crate::{
    DataFieldAccessor, DatabaseDescription, FocusConstraints, FocusHandler, Folder, FolderFocus,
    FolderHandler, KeySet, Pair, PathConstraints, SubscriberData, ToKey, VariantCount,
    mutex::ScopedLocked,
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
    const fn new(data: Database::Data) -> Self {
        Self { data }
    }

    fn clone(
        &mut self,
        other: &Database::Data,
    ) -> Result<Vec<Database::FlatKey, FLAT_PARAMETER_COUNT>, DatabaseError> {
        let mut differing_keys: KeySet<Database::AbsKey, Database::FlatKey, FLAT_PARAMETER_COUNT> =
            KeySet::new();

        self.data.clone(&mut differing_keys, other)?;

        Ok(differing_keys.to_vector())
    }

    fn get_absolute(&self, key: Database::AbsKey) -> Database::FlatField {
        self.data.get(key).into()
    }

    fn get_flat<Focus: FocusHandler<Database>>(
        &self,
        focus_handler: &Focus,
        key: Database::FlatKey,
    ) -> Database::FlatField {
        let abs_key = focus_handler.get_focus_key(key);
        self.get_absolute(abs_key)
    }

    fn set_absolute<Focus: FocusHandler<Database>>(
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

    fn set_flat<Focus: FocusHandler<Database>>(
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

    fn on_focus_change<FocusType, Path, InternalAbsPair>(
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

        let other: &<Database::Data as FolderHandler<Path, Database>>::Content =
            self.data.get_at(rhs);

        match self.data.compare_path(&mut differing_keys, lhs, other) {
            Ok(()) => Ok(differing_keys.to_vector()),
            Err(error) => Err(error),
        }
    }

    fn clone_path<FocusType, Path, InternalAbsPair>(
        &mut self,
        path: Path,
        other: &<Database::Data as FolderHandler<Path, Database>>::Content,
    ) -> Result<Vec<Database::FlatKey, FLAT_PARAMETER_COUNT>, DatabaseError>
    where
        Path: PathConstraints<(Database::AbsKey, Database::AbsField), InternalAbsPair>,
        InternalAbsPair: Pair,
        Database::Data: FolderHandler<Path, Database>,
    {
        let mut differing_keys: KeySet<Database::AbsKey, Database::FlatKey, FLAT_PARAMETER_COUNT> =
            KeySet::new();

        match self.data.clone_path(&mut differing_keys, path, other) {
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
    pub data:
        ScopedLocked<Mutex, InternalMutable<Database, ABS_PARAMETER_COUNT, FLAT_PARAMETER_COUNT>>,
    pub subscribers: SubscriberData<'a, Mutex, Database, FLAT_PARAMETER_COUNT>,
    pub focus_handler: Focus,
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

    pub(crate) fn get_focus<FocusType, Path, AbsPair, InternalAbsPair>(&self) -> FocusType
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

    pub(crate) fn set_focus<FocusType, Path, InternalAbsPair>(
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

    pub(crate) fn clone(&self, other: &Database::Data) -> Result<(), DatabaseError> {
        match self.data.try_with(|internal| internal.clone(other)) {
            None => Err(DatabaseError::LockFail),
            Some(result) => match result {
                Ok(changes) => {
                    self.subscribers.on_changes(&changes);
                    Ok(())
                }
                Err(error) => Err(error),
            },
        }
    }

    pub(crate) fn clone_path<FocusType, Path, InternalAbsPair>(
        &mut self,
        path: Path,
        other: &<Database::Data as FolderHandler<Path, Database>>::Content,
    ) -> Result<(), DatabaseError>
    where
        Path: PathConstraints<(Database::AbsKey, Database::AbsField), InternalAbsPair>,
        InternalAbsPair: Pair,
        Database::Data: FolderHandler<Path, Database>,
    {
        match self.data.try_with(|internal| {
            internal.clone_path::<FocusType, Path, InternalAbsPair>(path, other)
        }) {
            None => Err(DatabaseError::LockFail),
            Some(result) => match result {
                Ok(changes) => {
                    self.subscribers.on_changes(&changes);
                    Ok(())
                }
                Err(error) => Err(error),
            },
        }
    }
}
