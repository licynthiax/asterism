//! # Resource Logics
//!
//! Resource logics communicate that generic or specific resources can be created, destroyed,
//! converted, or transferred between abstract or concrete locations. They create, destroy, and
//! exchange (usually) discrete quantities of generic or specific resources in or between abstract
//! or concrete locations on demand or over time, and trigger other actions when these transactions
//! take place.

use std::collections::BTreeMap;

/// A resource logic that queues transactions, then applies them all at once when updating.
pub struct QueuedResources<ID: PoolInfo> {
    /// The items involved, and their values.
    pub items: BTreeMap<ID, f64>,
    /// Each transaction is a list of items involved in the transaction and the amount they're
    /// being changed.
    pub transactions: Vec<Vec<(ID, Transaction)>>,
    /// A Vec of all transactions and if they were able to be completed or not. If yes, supply
    /// a Vec of the IDs of successful transactions; if no, supply the ID of the pool that
    /// caused the error and a reason (see [ResourceError]).
    pub completed: Vec<Result<Vec<ID>, (ID, ResourceError)>>,
}

impl<ID: PoolInfo> QueuedResources<ID> {
    pub fn new() -> Self {
        Self {
            items: BTreeMap::new(),
            transactions: Vec::new(),
            completed: Vec::new(),
        }
    }

    /// Updates the values of resources based on the queued transactions. If a transaction cannot
    /// be completed (if the value goes below zero), a snapshot of the resources before the
    /// transaction occurred is restored, and the transaction is marked as incomplete, and we
    /// continue to process the remaining transactions.
    pub fn update(&mut self) {
        self.completed.clear();
        'exchange: for exchange in self.transactions.iter() {
            let mut snapshot = BTreeMap::new();
            for (item_type, ..) in exchange {
                snapshot.insert(*item_type, *self.items.get(&item_type).unwrap());
            }

            let mut item_types = Vec::new();
            for (item_type, change) in exchange.iter() {
                match self.is_possible(item_type, change) {
                    Ok(_) => {}
                    Err(err) => {
                        self.completed.push(Err((*item_type, err)));
                        for (item_type, val) in snapshot.iter() {
                            *self.items.get_mut(&item_type).unwrap() = *val;
                        }
                        continue 'exchange;
                    }
                }
                match change {
                    Transaction::Change(amt) => {
                        *self.items.get_mut(&item_type).unwrap() += *amt as f64;
                        item_types.push(*item_type);
                    }
                }
            }
            self.completed.push(Ok(item_types));
        }
        self.transactions.clear();
    }

    /// Checks if the transaction is possible or not
    fn is_possible(&self, item_type: &ID, transaction: &Transaction) -> Result<(), ResourceError> {
        if let Some(value) = self.items.get(item_type) {
            match transaction {
                Transaction::Change(amt) => {
                    if *value + *amt > item_type.max_value() {
                        Err(ResourceError::TooBig)
                    } else if *value + *amt < item_type.min_value() {
                        Err(ResourceError::TooSmall)
                    } else {
                        Ok(())
                    }
                }
            }
        } else {
            Err(ResourceError::PoolNotFound)
        }
    }

    /// Gets the value of the item based on its ID.
    pub fn get_value_by_itemtype(&self, item_type: &ID) -> Option<f64> {
        self.items.get(item_type).and_then(|value| Some(*value))
    }
}

/// A transaction holding the amount the value should change by.
#[derive(Clone, Copy)]
pub enum Transaction {
    Change(f64),
}

/// information for the min/max values the entities in this pool can take, inclusive (I think)
pub trait PoolInfo: Copy + Ord {
    fn min_value(&self) -> f64 {
        std::f64::MIN
    }
    fn max_value(&self) -> f64 {
        std::f64::MAX
    }
}

/// Errors possible when trying to complete a transaction.
#[derive(Debug)]
pub enum ResourceError {
    PoolNotFound,
    TooBig,
    TooSmall,
}
