//! # Resource Logics
//!
//! Resource logics communicate that generic or specific resources can be created, destroyed, converted, or transferred between abstract or concrete locations. They create, destroy, and exchange (usually) discrete quantities of generic or specific resources in or between abstract or concrete locations on demand or over time, and trigger other actions when these transactions take place.

use crate::{Event, EventType, LendingIterator, Logic, Reaction};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::ops::{Add, AddAssign};

pub struct PoolValues<Value> {
    pub val: Value,
    pub min: Value,
    pub max: Value,
}

/// A resource logic that queues transactions, then applies them all at once when updating.
pub struct QueuedResources<ID, Value>
where
    ID: Copy + Ord + Debug + 'static,
    Value: Add<Output = Value> + AddAssign + Ord + Copy + 'static,
{
    /// The items involved, and their values, as a tuple of (actual value, minimum value, maximum value).
    pub items: BTreeMap<ID, PoolValues<Value>>,
    /// Each transaction is a list of items involved in the transaction and the amount they're being changed.
    pub transactions: Vec<(ID, Transaction<Value>)>,
    /// A Vec of all transactions and if they were able to be completed or not. If not, also report an error (see [ResourceEvent] and [ResourceError]).
    pub completed: Vec<ResourceEvent<ID, Value>>,
}

impl<ID, Value> Logic for QueuedResources<ID, Value>
where
    ID: Copy + Ord + Debug,
    Value: Add<Output = Value> + AddAssign + Ord + Copy,
{
    type Event = ResourceEvent<ID, Value>;
    type Reaction = ResourceReaction<ID, Value>;

    type Ident = ID;
    type IdentData<'a> = &'a mut PoolValues<Value> where Self: 'a;

    type DataIter<'a> = RsrcDataIter<'a, ID, Value> where Self: 'a;

    fn handle_predicate(&mut self, reaction: &Self::Reaction) {
        self.transactions.push(*reaction);
    }

    fn get_ident_data(&mut self, ident: Self::Ident) -> Self::IdentData<'_> {
        self.items
            .get_mut(&ident)
            .unwrap_or_else(|| panic!("requested pool {:?} doesn't exist in resource logic", ident))
    }

    fn data_iter(&mut self) -> Self::DataIter<'_> {
        RsrcDataIter {
            resources: self.items.iter_mut(),
        }
    }

    fn events(&self) -> &[Self::Event] {
        &self.completed
    }
}

impl<ID, Value> QueuedResources<ID, Value>
where
    ID: Copy + Ord + Debug,
    Value: Add<Output = Value> + AddAssign + Ord + Copy,
{
    pub fn new() -> Self {
        Self {
            items: BTreeMap::new(),
            transactions: Vec::new(),
            completed: Vec::new(),
        }
    }

    /// Updates the values of resources based on the queued transactions. If a transaction cannot be completed (if the value goes below its min or max), a snapshot of the resources before the transaction occurred is restored, and the transaction is marked as incomplete, and we continue to process the remaining transactions.
    pub fn update(&mut self) {
        self.completed.clear();

        for (id, transaction) in self.transactions.iter() {
            if let Err(err) = self.is_possible(id, transaction) {
                self.completed.push(ResourceEvent {
                    pool: *id,
                    transaction: *transaction,
                    event_type: ResourceEventType::TransactionUnsuccessful(err),
                });
                continue;
            }

            let PoolValues { val, min, max } = self.items.get_mut(id).unwrap();
            match transaction {
                Transaction::Change(amt) => {
                    *val += *amt;
                }
                Transaction::Set(amt) => {
                    *val = *amt;
                }
                Transaction::SetMax(new_max) => {
                    *max = *new_max;
                }
                Transaction::SetMin(new_min) => {
                    *min = *new_min;
                }
            }
            self.completed.push(ResourceEvent {
                pool: *id,
                transaction: *transaction,
                event_type: ResourceEventType::PoolUpdated,
            });
        }
        self.transactions.clear();
    }

    /// Checks if the transaction is possible or not
    fn is_possible(
        &self,
        item_type: &ID,
        transaction: &Transaction<Value>,
    ) -> Result<(), ResourceError> {
        if let Some(PoolValues { val, min, max }) = self.items.get(item_type) {
            match transaction {
                Transaction::Change(amt) => {
                    if *val + *amt > *max {
                        Err(ResourceError::TooBig)
                    } else if *val + *amt < *min {
                        Err(ResourceError::TooSmall)
                    } else {
                        Ok(())
                    }
                }
                _ => Ok(()),
            }
        } else {
            Err(ResourceError::PoolNotFound)
        }
    }

    /// Gets the value of the item based on its ID.
    pub fn get_value_by_itemtype(&self, item_type: &ID) -> Option<Value> {
        self.items.get(item_type).map(|PoolValues { val, .. }| *val)
    }
}

/// A transaction holding the amount the value should change by.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Transaction<Value>
where
    Value: Add + AddAssign,
{
    Change(Value),
    Set(Value),
    SetMax(Value),
    SetMin(Value),
}

/// Errors possible when trying to complete a transaction.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ResourceError {
    PoolNotFound,
    TooBig,
    TooSmall,
}

pub type ResourceReaction<ID, Value> = (ID, Transaction<Value>);

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ResourceEvent<ID, Value>
where
    Value: Eq + Add + AddAssign,
{
    pub pool: ID,
    pub transaction: Transaction<Value>,
    pub event_type: ResourceEventType,
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum ResourceEventType {
    PoolUpdated,
    TransactionUnsuccessful(ResourceError),
}

impl EventType for ResourceEventType {}

impl<ID: Ord, Value: Add + AddAssign> Reaction for ResourceReaction<ID, Value> {}

impl<ID: Ord, Value: Add + AddAssign + Eq> Event for ResourceEvent<ID, Value> {
    type EventType = ResourceEventType;
    fn get_type(&self) -> &Self::EventType {
        &self.event_type
    }
}

pub struct RsrcDataIter<'rsrc, ID, Value>
where
    ID: Copy + Ord + Debug + 'static,
    Value: Add<Output = Value> + AddAssign + Ord + Copy + 'static,
{
    resources: std::collections::btree_map::IterMut<'rsrc, ID, PoolValues<Value>>,
}

impl<'rsrc, ID, Value> LendingIterator for RsrcDataIter<'rsrc, ID, Value>
where
    ID: Copy + Ord + Debug + 'static,
    Value: Add<Output = Value> + AddAssign + Ord + Copy + 'static,
{
    type Item<'a> = (
        <QueuedResources<ID, Value> as Logic>::Ident,
        <QueuedResources<ID, Value> as Logic>::IdentData<'a>,
    ) where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        self.resources.next().map(|(id, vals)| (*id, vals))
    }
}
