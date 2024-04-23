//! # Resource Logics
//!
//! Resource logics communicate that generic or specific resources can be created, destroyed, converted, or transferred between abstract or concrete locations. They create, destroy, and exchange (usually) discrete quantities of generic or specific resources in or between abstract or concrete locations on demand or over time, and trigger other actions when these transactions take place.

use crate::{Event, EventType, LendingIterator, Logic, Reaction};
use num_traits::{Num, Signed};
use std::collections::BTreeMap;
use std::fmt::Debug;

pub struct PoolValues<Value> {
    pub val: Value,
    pub min: Value,
    pub max: Value,
}

/// A resource logic that queues transactions, then applies them all at once when updating.
pub struct QueuedResources<ID, Value>
where
    ID: Copy + Ord + Debug,
    Value: Num + Signed + Copy + PartialOrd,
{
    /// The items involved, and their values, as a tuple of (actual value, minimum value, maximum value).
    pub items: BTreeMap<ID, PoolValues<Value>>,
    /// Each transaction is a list of items involved in the transaction and the amount they're being changed.
    pub transactions: Vec<(ID, Transaction<Value, ID>)>,
    /// A Vec of all transactions and if they were able to be completed or not. If not, also report an error (see [ResourceEvent] and [ResourceError]).
    pub completed: Vec<ResourceEvent<ID, Value>>,
}

impl<ID, Value> Logic for QueuedResources<ID, Value>
where
    ID: Copy + Ord + Debug,
    Value: Num + Signed + Copy + PartialOrd,
{
    type Event = ResourceEvent<ID, Value>;
    type Reaction = ResourceReaction<ID, Value>;

    type Ident = ID;
    type IdentData<'rsrc> = &'rsrc mut PoolValues<Value> where Self: 'rsrc;

    type DataIter<'iter> = RsrcDataIter<'iter, ID, Value> where Self: 'iter;

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
    Value: Num + Signed + Copy + PartialOrd,
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
            let err_event = |pool, transaction, err| ResourceEvent {
                pool,
                transaction,
                event_type: ResourceEventType::TransactionUnsuccessful(err),
            };
            if let Transaction::Trade(amt, other) = transaction {
                let zero: Value = num_traits::identities::zero();
                // check if first transaction is possible
                if let Err(err) = self.is_possible(id, &Transaction::Change(zero - *amt)) {
                    self.completed.push(err_event(*id, *transaction, err));
                    continue;
                }
                // check if second is possible
                if let Err(err) = self.is_possible(other, &Transaction::Change(*amt)) {
                    self.completed.push(err_event(*id, *transaction, err));
                    continue;
                }

                let PoolValues { val: val_i, .. } = self.items.get_mut(id).unwrap();
                *val_i = *val_i - *amt;
                let PoolValues { val: val_j, .. } = self.items.get_mut(other).unwrap();
                *val_j = *val_j + *amt;
                continue;
            }

            if let Err(err) = self.is_possible(id, transaction) {
                self.completed.push(err_event(*id, *transaction, err));
                continue;
            }

            let PoolValues { val, min, max } = self.items.get_mut(id).unwrap();
            match transaction {
                Transaction::Change(amt) => {
                    *val = *val + *amt;
                }
                Transaction::Set(amt) => {
                    *val = *val + *amt;
                }
                Transaction::SetMax(new_max) => {
                    *max = *new_max;
                }
                Transaction::SetMin(new_min) => {
                    *min = *new_min;
                }
                _ => {}
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
        transaction: &Transaction<Value, ID>,
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
pub enum Transaction<Value, Ident>
where
    Value: Num + Signed + Copy + PartialOrd,
    Ident: Copy + Ord + Debug,
{
    Trade(Value, Ident),
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

pub type ResourceReaction<ID, Value> = (ID, Transaction<Value, ID>);

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ResourceEvent<ID, Value>
where
    Value: Num + Signed + Copy + PartialOrd,
    ID: Copy + Ord + Debug,
{
    pub pool: ID,
    pub transaction: Transaction<Value, ID>,
    pub event_type: ResourceEventType,
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum ResourceEventType {
    PoolUpdated,
    TransactionUnsuccessful(ResourceError),
}

impl EventType for ResourceEventType {}

impl<ID, Value> Reaction for ResourceReaction<ID, Value>
where
    ID: Ord + Copy + Debug,
    Value: Num + Signed + Copy + PartialOrd,
{
}

impl<ID, Value> Event for ResourceEvent<ID, Value>
where
    ID: Ord + Copy + Debug,
    Value: Num + Signed + Copy + PartialOrd,
{
    type EventType = ResourceEventType;
    fn get_type(&self) -> &Self::EventType {
        &self.event_type
    }
}

pub struct RsrcDataIter<'iter, ID, Value>
where
    ID: Copy + Ord + Debug,
    Value: Num + Signed,
{
    resources: std::collections::btree_map::IterMut<'iter, ID, PoolValues<Value>>,
}

impl<'iter, ID, Value> LendingIterator for RsrcDataIter<'iter, ID, Value>
where
    ID: Copy + Ord + Debug,
    Value: Num + Signed + Copy + PartialOrd,
{
    type Item<'a> = (
        <QueuedResources<ID, Value> as Logic>::Ident,
        <QueuedResources<ID, Value> as Logic>::IdentData<'a>,
    ) where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        self.resources.next().map(|(id, vals)| (*id, vals))
    }
}

/// An instant resource logic updates as it receives reactions and produces events immediately, rather than at the end of each event loop.
pub struct InstantResources<ID, Value>
where
    ID: Copy + Ord + Debug,
    Value: Num + Signed + Copy + PartialOrd,
{
    /// The items involved and their values.
    pub items: BTreeMap<ID, PoolValues<Value>>,
    /// A Vec of all transactions and if they were able to be completed or not. If not, also report an error (see [ResourceEvent] and [ResourceError]).
    pub completed: Vec<ResourceEvent<ID, Value>>,
}

impl<ID, Value> InstantResources<ID, Value>
where
    ID: Copy + Ord + Debug,
    Value: Num + Signed + PartialOrd + Copy + PartialOrd,
{
    pub fn new() -> Self {
        Self {
            items: BTreeMap::new(),
            completed: Vec::new(),
        }
    }

    /// update function-- clears events once per game loop
    pub fn update(&mut self) {
        self.completed.clear();
    }

    /// Checks if the transaction is possible or not
    fn is_possible(
        &self,
        item_type: &ID,
        transaction: &Transaction<Value, ID>,
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

impl<ID, Value> Logic for InstantResources<ID, Value>
where
    ID: Copy + Ord + Debug,
    Value: Num + Signed + Copy + PartialOrd,
{
    type Event = ResourceEvent<ID, Value>;
    type Reaction = ResourceReaction<ID, Value>;

    type Ident = ID;
    type IdentData<'a> = &'a mut PoolValues<Value> where Self: 'a;

    type DataIter<'a> = InstRsrcDataIter<'a, ID, Value> where Self: 'a;

    fn handle_predicate(&mut self, reaction: &Self::Reaction) {
        let (item_type, change) = reaction;

        if let Err(err) = self.is_possible(item_type, change) {
            self.completed.push(ResourceEvent {
                pool: *item_type,
                transaction: *change,
                event_type: ResourceEventType::TransactionUnsuccessful(err),
            });
            return;
        }

        let PoolValues { val, min, max } = self.items.get_mut(item_type).unwrap();
        match change {
            Transaction::Change(amt) => {
                *val = *val + *amt;
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
            Transaction::Trade(_, _) => {}
        }
        self.completed.push(ResourceEvent {
            pool: *item_type,
            transaction: *change,
            event_type: ResourceEventType::PoolUpdated,
        });
    }

    // dislike this panic. is it reasonable to put an option on the type? oh ugh i don't like the way these tables work
    fn get_ident_data(&mut self, ident: Self::Ident) -> Self::IdentData<'_> {
        self.items
            .get_mut(&ident)
            .unwrap_or_else(|| panic!("requested pool {:?} doesn't exist in resource logic", ident))
    }

    fn data_iter(&mut self) -> Self::DataIter<'_> {
        InstRsrcDataIter {
            resources: self.items.iter_mut(),
        }
    }

    fn events(&self) -> &[Self::Event] {
        &self.completed
    }
}

pub struct InstRsrcDataIter<'iter, ID, Value>
where
    ID: Copy + Ord + Debug,
    Value: Num + Signed,
{
    resources: std::collections::btree_map::IterMut<'iter, ID, PoolValues<Value>>,
}

impl<'iter, ID, Value> LendingIterator for InstRsrcDataIter<'iter, ID, Value>
where
    ID: Copy + Ord + Debug,
    Value: Num + Signed + Copy + PartialOrd,
{
    type Item<'a> = (
        <InstantResources<ID, Value> as Logic>::Ident,
        <InstantResources<ID, Value> as Logic>::IdentData<'a>,
    ) where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        self.resources.next().map(|(id, vals)| (*id, vals))
    }
}
