//! Activation Module
//!
//! Handles extension activation events and orchestration.
//! Manages the activation lifecycle for extensions.

pub mod ActivationContext;

pub mod ActivationEngine;

pub mod ActivationEvent;

pub(crate) mod ActivationHandler;

pub mod ActivationRecord;

pub(crate) mod WildMatch;

#[cfg(test)]
mod Tests;
