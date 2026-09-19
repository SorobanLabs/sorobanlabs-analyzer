//! Controlled upgrade rehearsal for the SorobanLabs Analyzer.
//!
//! A rehearsal compares the current and candidate executables' observed
//! behavior under the same representative state and the same set of
//! invocations: return values, errors, events, state reads/writes, and
//! (when trustworthy) resource usage. This crate is separate from
//! `analyzer-executable`/`analyzer-state`/`analyzer-auth` because it has
//! a genuinely distinct dependency footprint: a real execution backend
//! needs a full Soroban host implementation, which the rest of this
//! workspace's static analysis does not, and should not be forced to
//! depend on.
//!
//! # Current state
//!
//! This version establishes the typed model only: [`input::RehearsalInput`]
//! (what to rehearse), [`observation::InvocationObservation`] (what was
//! observed for one invocation on one side, including
//! [`observation::ExecutionOutcome::Blocked`] for an invocation that
//! could not be safely or meaningfully executed), and
//! [`trace::RehearsalTrace`] (the bounded, two-sided collection of
//! observations for a completed rehearsal). **No execution backend
//! exists yet.** Building one requires a documented feasibility
//! assessment of the official Soroban host implementation against this
//! workspace (a later step); this crate does not substitute a generic
//! WASM runtime for that assessment, and does not claim Soroban
//! execution support until that assessment succeeds.

pub mod input;
pub mod observation;
pub mod trace;

pub use input::{ExecutionLimits, RehearsalInput, RehearsalInvocation};
pub use observation::{
    EventObservation, ExecutionOutcome, InvocationObservation, ResourceUsage,
    StateAccessObservation,
};
pub use trace::RehearsalTrace;
