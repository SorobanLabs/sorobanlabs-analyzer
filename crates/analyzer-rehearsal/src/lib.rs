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
//! [`input::RehearsalInput`] (what to rehearse),
//! [`observation::InvocationObservation`] (what was observed for one
//! invocation on one side, including
//! [`observation::ExecutionOutcome::Blocked`] for an invocation that
//! could not be safely or meaningfully executed), and
//! [`trace::RehearsalTrace`] (the bounded, two-sided collection of
//! observations for a completed rehearsal) establish the typed model.
//! [`host`] is a real, verified (not assumed) local execution backend
//! built on the official `soroban-env-host`; see that module's docs for
//! the feasibility evidence and its current scope (return
//! value/success/error/trap/blocked outcome only; events, state
//! access, and resource usage are not yet captured). Rehearsal
//! fixtures beyond the single minimal one this module's tests use, and
//! comparing the two sides' observations into a behavioral diff, are
//! later steps.

pub mod diff;
pub mod host;
pub mod input;
pub mod observation;
pub mod trace;

pub use diff::{diff_invocations, BehavioralDiff, Difference};
pub use host::rehearse_invocation;
pub use input::{ExecutionLimits, RehearsalInput, RehearsalInvocation};
pub use observation::{
    EventObservation, ExecutionOutcome, InvocationObservation, ResourceUsage,
    StateAccessObservation,
};
pub use trace::RehearsalTrace;
