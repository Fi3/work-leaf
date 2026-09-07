//! Benchmark-private author enrollment and private-test feedback.
//!
//! Public launch values stay unchanged. Tickets belong to their preparing
//! CommandChat instance and clones, and are consumed before provider injection.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Component, Path};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use serde::{Deserialize, Serialize};

use crate::agent::{AgentId, AgentLaunch};

#[path = "bench_private_test_first_bridge.rs"]
mod bridge;
pub(crate) use bridge::execute as execute_preview;
pub(super) use bridge::initialize as initialize_bridge;
pub(super) use bridge::parse_manifest;

pub(crate) fn record(event: &str, agent_id: &AgentId, value: serde_json::Value) -> io::Result<()> {
    let experiment = super::active()?
        .filter(|experiment| experiment.manifest.schema == super::SCHEMA_V6)
        .ok_or_else(|| invalid("private trace has no active experiment"))?;
    let mut evidence = experiment.evidence.lock().map_err(super::invalid)?;
    evidence.sequence += 1;
    let row = serde_json::json!({"event":event,"schema":super::SCHEMA_V6,
        "sequence":evidence.sequence,"run_id":experiment.manifest.run_id,
        "condition":experiment.manifest.condition,"agent_id":agent_id,"private_preview":value});
    use std::io::Write;
    serde_json::to_writer(&mut evidence.file, &row).map_err(super::invalid)?;
    evidence.file.write_all(b"\n")?;
    evidence.file.flush()
}

pub(crate) fn run_id() -> io::Result<String> {
    Ok(super::active()?
        .filter(|experiment| experiment.manifest.schema == super::SCHEMA_V6)
        .ok_or_else(|| invalid("private execution has no active run"))?
        .manifest
        .run_id
        .clone())
}

pub(crate) fn active() -> io::Result<bool> {
    Ok(super::active()?.is_some_and(|experiment| experiment.manifest.schema == super::SCHEMA_V6))
}

pub(crate) fn forward_policy(
    agent_id: &AgentId,
    feature: &str,
    prompt: &str,
    original: String,
    spans: Vec<super::PromptSpan>,
) -> io::Result<String> {
    let Some(experiment) = super::active()? else {
        return Ok(original);
    };
    if experiment.manifest.schema != super::SCHEMA_V6 {
        return Ok(original);
    }
    if resumes()
        .lock()
        .map_err(|_| invalid("resume registry poisoned"))?
        .contains_key(agent_id)
    {
        return Err(invalid(
            "a continuation must resume its existing session, not inject a new launch policy",
        ));
    }
    let context = INJECTION.with(|slot| slot.borrow().clone());
    let (role, generation) = if let Some(context) = context {
        let launch = AgentLaunch::new(agent_id.clone(), context.launch.kind, feature, prompt);
        scoped_injection(&launch)?.ok_or_else(|| invalid("owned injection scope disappeared"))?
    } else {
        if author_identities()
            .lock()
            .map_err(|_| invalid("author identity registry poisoned"))?
            .contains(agent_id)
        {
            return Err(invalid(
                "prepared-author injection requires its synchronous owned launch scope",
            ));
        }
        (Role::Other, 0)
    };
    let (candidate, owned_spans) = render_policy(&original, &spans, role)?;
    let mut evidence = experiment.evidence.lock().map_err(super::invalid)?;
    evidence.sequence += 1;
    let row = serde_json::json!({"event":"prompt","schema":super::SCHEMA_V6,
        "sequence":evidence.sequence,"run_id":experiment.manifest.run_id,
        "condition":experiment.manifest.condition,"site":"policy-injection","agent_id":agent_id,
        "launch_generation":generation,"owned_role":if role==Role::Author {"author"}else{"other"},
        "original_prompt":original,"candidate_prompt":candidate,"forwarded_prompt":candidate,
        "spans":owned_spans,"changed":original!=candidate});
    use std::io::Write;
    serde_json::to_writer(&mut evidence.file, &row).map_err(super::invalid)?;
    evidence.file.write_all(b"\n")?;
    evidence.file.flush()?;
    Ok(candidate)
}

static RESUMES: OnceLock<Mutex<BTreeMap<AgentId, ()>>> = OnceLock::new();
// Retain explicit author identities after failed/retired scopes too: deferred
// provider work must not become an unowned ordinary launch after guard rollback.
static AUTHOR_IDENTITIES: OnceLock<Mutex<BTreeSet<AgentId>>> = OnceLock::new();
fn author_identities() -> &'static Mutex<BTreeSet<AgentId>> {
    AUTHOR_IDENTITIES.get_or_init(Mutex::default)
}
fn resumes() -> &'static Mutex<BTreeMap<AgentId, ()>> {
    RESUMES.get_or_init(Mutex::default)
}

pub(crate) struct ResumeGuard {
    agent_id: AgentId,
}

pub(crate) fn resume_scope(agent_id: &AgentId) -> io::Result<Option<ResumeGuard>> {
    if !active()? {
        return Ok(None);
    }
    let mut active = resumes()
        .lock()
        .map_err(|_| invalid("resume registry poisoned"))?;
    if active.insert(agent_id.clone(), ()).is_some() {
        return Err(invalid("another continuation owns this agent session"));
    }
    Ok(Some(ResumeGuard {
        agent_id: agent_id.clone(),
    }))
}

impl Drop for ResumeGuard {
    fn drop(&mut self) {
        if let Ok(mut active) = resumes().lock() {
            active.remove(&self.agent_id);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Role {
    Author,
    Other,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Registry(Arc<Mutex<State>>);

#[derive(Debug, Default)]
struct State {
    shutting_down: bool,
    prepared: BTreeMap<AgentId, Ticket>,
    owners: BTreeMap<AgentId, Owner>,
    previews: BTreeMap<(AgentId, u64, String), Preview>,
}

#[derive(Debug)]
struct Ticket {
    launch: AgentLaunch,
    role: Role,
    generation: u64,
    started: bool,
    injected: bool,
    cancelled: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct Owner {
    pub(crate) generation: u64,
    qualifying_delivered: bool,
    cancellation: Option<Arc<AtomicBool>>,
    executing: bool,
    current_proposal: Option<String>,
    cancelled: bool,
}

#[derive(Debug)]
struct Preview {
    proposal: Proposal,
    result: Option<String>,
    qualifying: bool,
    delivered: bool,
    cancellation: Arc<AtomicBool>,
}

pub(crate) enum Reservation {
    New(ReservationToken),
    Replay(ReservationToken, String),
}

#[derive(Clone)]
pub(crate) struct ReservationToken {
    pub(crate) agent_id: AgentId,
    pub(crate) generation: u64,
    pub(crate) proposal_id: String,
    pub(crate) cancel: Arc<AtomicBool>,
}

impl ReservationToken {
    pub(crate) fn ensure_not_cancelled(&self) -> io::Result<()> {
        if self.cancel.load(Ordering::Acquire) {
            Err(invalid(
                "private preview was cancelled; no continuation is authorized",
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone)]
struct InjectionContext {
    registry: Registry,
    launch: AgentLaunch,
    generation: u64,
    role: Role,
}

thread_local! {
    static INJECTION: RefCell<Option<InjectionContext>> = const { RefCell::new(None) };
}

pub(crate) fn scoped_injection(launch: &AgentLaunch) -> io::Result<Option<(Role, u64)>> {
    let context = INJECTION.with(|slot| slot.borrow().clone());
    let Some(context) = context else {
        return Ok(None);
    };
    mark_injected(
        &context.registry,
        &context.launch.id,
        context.generation,
        launch,
    )?;
    Ok(Some((context.role, context.generation)))
}

fn mark_injected(
    registry: &Registry,
    agent_id: &AgentId,
    generation: u64,
    launch: &AgentLaunch,
) -> io::Result<()> {
    let mut state = registry.state()?;
    let ticket = state
        .prepared
        .get_mut(agent_id)
        .ok_or_else(|| invalid("provisional launch ticket is absent"))?;
    if ticket.generation != generation
        || ticket.cancelled
        || !ticket.started
        || ticket.injected
        || ticket.launch != *launch
    {
        return Err(invalid(
            "policy injection differs, repeats, or has no provisional owner",
        ));
    }
    ticket.injected = true;
    Ok(())
}

pub(crate) struct LaunchGuard {
    registry: Registry,
    launch: AgentLaunch,
    role: Role,
    generation: u64,
    committed: bool,
}

fn invalid(message: &str) -> io::Error {
    io::Error::other(format!("private test-first: {message}"))
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Proposal {
    pub(crate) id: String,
    pub(crate) revision_of: Option<String>,
    pub(crate) format: String,
    pub(crate) reason: String,
    pub(crate) test_purpose: String,
    pub(crate) test_paths: Vec<String>,
    pub(crate) command: String,
    pub(crate) lock_paths: Vec<String>,
    #[serde(default)]
    pub(crate) body: String,
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_.".contains(&byte))
        && value != "."
        && value != ".."
}

fn relative_path(value: &str, root_allowed: bool) -> bool {
    if root_allowed && value == "." {
        return true;
    }
    !value.is_empty()
        && !value.contains('\0')
        && !value.contains('\\')
        && !value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        && Path::new(value)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

/// Only a standalone explicit envelope is private work. Copied/fenced text and
/// ordinary directives retain their ordinary parser and cannot create a preview.
pub(crate) fn parse_preview(text: &str) -> io::Result<Option<Proposal>> {
    let text = text.trim_start_matches(['\n', '\r']);
    let Some(header) = text.strip_prefix("@work-leaf test-preview ") else {
        return Ok(None);
    };
    if text.len() > 8 * 1024 * 1024 {
        return Err(invalid("preview envelope exceeds its byte bound"));
    }
    let (header, remainder) = header
        .split_once('\n')
        .ok_or_else(|| invalid("preview envelope is incomplete"))?;
    let mut proposal: Proposal = serde_json::from_str(header)
        .map_err(|_| invalid("preview header must match its exact typed schema"))?;
    if !proposal.body.is_empty() {
        return Err(invalid(
            "preview body belongs only in the raw body component",
        ));
    }
    let end = remainder
        .find("\n@work-leaf end")
        .ok_or_else(|| invalid("preview envelope requires its explicit terminal boundary"))?;
    if !remainder[end + "\n@work-leaf end".len()..]
        .trim()
        .is_empty()
    {
        return Err(invalid(
            "preview cannot be mixed with other output or directives",
        ));
    }
    proposal.body = remainder[..=end].to_string();
    if proposal
        .body
        .lines()
        .any(|line| line.starts_with("@work-leaf "))
    {
        return Err(invalid("preview body contains another top-level directive"));
    }
    if !valid_id(&proposal.id)
        || proposal
            .revision_of
            .as_deref()
            .is_some_and(|id| !valid_id(id) || id == proposal.id)
        || !matches!(proposal.format.as_str(), "edit" | "patch")
        || proposal.reason.trim().is_empty()
        || proposal.test_purpose.trim().is_empty()
        || proposal.body.trim().is_empty()
        || proposal.command.trim().is_empty()
        || proposal.command.len() > 16 * 1024
        || proposal.command.contains('\0')
        || proposal.lock_paths.is_empty()
        || proposal.lock_paths.len() > 128
        || proposal
            .lock_paths
            .iter()
            .any(|path| !relative_path(path, true))
        || proposal.test_paths.is_empty()
        || proposal.test_paths.len() > 128
        || proposal
            .test_paths
            .iter()
            .any(|path| !relative_path(path, false))
        || proposal
            .test_paths
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != proposal.test_paths.len()
    {
        return Err(invalid(
            "preview declaration contains unsupported identities, units or command",
        ));
    }
    Ok(Some(proposal))
}

pub(crate) fn preview_terminal(text: &str) -> bool {
    matches!(parse_preview(text), Ok(Some(_)))
}

const PRIVATE_TIMING: &str = "Before your first shared edit or patch, submit a private test-only proposal and receive its actual check result. The private image is isolated; no shared file, commit or applied acknowledgment is produced. Then submit the ordinary combined tests-and-implementation patch and follow the unchanged applied-ACK validation/review flow. Do not request a private implementation check. Private test feedback includes a fresh private build/environment and is not proof that the shared tree passes.\nPrivate protocol: emit only `@work-leaf test-preview <one JSON metadata line>`, the ordinary exact edit or unified diff body, and a separate `@work-leaf end`. Metadata fields: `id` (new short identifier), `revision_of` (null or a previously delivered proposal id), `format` (`edit` or `patch`), `reason`, `test_purpose`, `test_paths` (normalized relative paths containing your proposed tests), `command` (your focused shell check), and `lock_paths` (the ordinary normalized command-write paths). Include tests, not the missing implementation, in this proposal. Work Leaf retains exact private after-images; do not calculate byte offsets or repeat test bodies in metadata. A malformed proposal or unsupported snapshot does not execute a check; retain the blocker or submit an explicit new revision. A first GREEN is retained as GREEN; never manufacture RED. Reusing an id cannot rerun a check or change its bytes. Private results are not shared ACKs. Never copy private Git/build state into the shared tree; later ordinary shared patch/conflict handling is authoritative.";
const PRIVATE_TESTS: &str = "Submit the needed tests through the private test-preview protocol before implementation. After its actual result, submit tests with the implementation as an ordinary combined shared patch; keep shared-worktree safety and every test obligation. Declare any test revision explicitly rather than hiding a changed or removed assertion.";

fn render_policy(
    original: &str,
    spans: &[super::PromptSpan],
    role: Role,
) -> io::Result<(String, Vec<serde_json::Value>)> {
    let mut candidate = String::with_capacity(original.len());
    let mut evidence = Vec::with_capacity(spans.len());
    let mut cursor = 0;
    let mut owned_timing = 0;
    for span in spans {
        let (expected, varied) = match span.id {
            "policy-buildable-work-unit" => {
                owned_timing += 1;
                (super::WORK_UNIT, PRIVATE_TIMING)
            }
            "instruction-tests-work-unit" => (super::TESTS_WORK_UNIT, PRIVATE_TESTS),
            _ => return Err(invalid("unsupported private policy span")),
        };
        if span.cue.start < cursor || original.get(span.cue.clone()) != Some(expected) {
            return Err(invalid(
                "private policy spans must match ordered renderer-owned bytes",
            ));
        }
        candidate.push_str(&original[cursor..span.cue.start]);
        let start = candidate.len();
        candidate.push_str(if role == Role::Author {
            varied
        } else {
            expected
        });
        evidence.push(
            serde_json::json!({"id":span.id,"original_start":span.cue.start,
            "original_end":span.cue.end,"candidate_start":start,"candidate_end":candidate.len()}),
        );
        cursor = span.cue.end;
    }
    if role == Role::Author && owned_timing != 1 {
        return Err(invalid(
            "author policy must contain exactly one owned timing span",
        ));
    }
    candidate.push_str(&original[cursor..]);
    Ok((candidate, evidence))
}

impl Registry {
    fn state(&self) -> io::Result<MutexGuard<'_, State>> {
        self.0
            .lock()
            .map_err(|_| invalid("owner registry poisoned"))
    }

    pub(crate) fn prepare(&self, launch: &AgentLaunch, role: Role) -> io::Result<u64> {
        static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);
        let mut state = self.state()?;
        if state.shutting_down {
            return Err(invalid("private owner registry is shutting down"));
        }
        if state
            .owners
            .get(&launch.id)
            .is_some_and(|owner| owner.executing)
        {
            return Err(invalid(
                "cannot replace an executing or undelivered preview owner",
            ));
        }
        if state.prepared.contains_key(&launch.id) {
            return Err(invalid("another prepared launch owns this agent identity"));
        }
        if role == Role::Author {
            let mut authors = author_identities()
                .lock()
                .map_err(|_| invalid("author identity registry poisoned"))?;
            if authors.len() >= 100_000 && !authors.contains(&launch.id) {
                return Err(invalid(
                    "explicit author identity registry reached its bound",
                ));
            }
            authors.insert(launch.id.clone());
        }
        let generation = NEXT_GENERATION
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| invalid("launch generation exhausted"))?;
        state.prepared.insert(
            launch.id.clone(),
            Ticket {
                launch: launch.clone(),
                role,
                generation,
                started: false,
                injected: false,
                cancelled: false,
            },
        );
        Ok(generation)
    }

    pub(crate) fn revise(&self, old: &AgentLaunch, new: &AgentLaunch) -> io::Result<()> {
        if old.id != new.id || old.kind != new.kind {
            return Err(invalid(
                "a preparation revision cannot change identity or backend kind",
            ));
        }
        let mut state = self.state()?;
        let ticket = state
            .prepared
            .get_mut(&old.id)
            .ok_or_else(|| invalid("prepared launch is absent"))?;
        if ticket.started || ticket.cancelled || ticket.launch != *old {
            return Err(invalid(
                "preparation revision does not match its unconsumed predecessor",
            ));
        }
        ticket.launch = new.clone();
        Ok(())
    }

    pub(crate) fn cancel_prepared(&self, agent_id: &AgentId) -> io::Result<()> {
        let mut state = self.state()?;
        if state
            .prepared
            .get(agent_id)
            .is_some_and(|ticket| ticket.started)
        {
            return Err(invalid("cannot cancel preparation after launch has begun"));
        }
        state.prepared.remove(agent_id);
        Ok(())
    }

    pub(crate) fn begin(&self, launch: &AgentLaunch) -> io::Result<LaunchGuard> {
        if INJECTION.with(|slot| slot.borrow().is_some()) {
            return Err(invalid(
                "another provisional launch owns this thread's injection scope",
            ));
        }
        let mut state = self.state()?;
        let ticket = state
            .prepared
            .get_mut(&launch.id)
            .ok_or_else(|| invalid("launch has no owned preparation ticket"))?;
        if ticket.started || ticket.cancelled || ticket.launch != *launch {
            return Err(invalid(
                "launch differs from its unconsumed preparation ticket",
            ));
        }
        ticket.started = true;
        INJECTION.with(|slot| {
            *slot.borrow_mut() = Some(InjectionContext {
                registry: self.clone(),
                launch: launch.clone(),
                generation: ticket.generation,
                role: ticket.role,
            })
        });
        Ok(LaunchGuard {
            registry: self.clone(),
            launch: launch.clone(),
            role: ticket.role,
            generation: ticket.generation,
            committed: false,
        })
    }

    pub(crate) fn owner(&self, agent_id: &AgentId) -> io::Result<Option<Owner>> {
        Ok(self.state()?.owners.get(agent_id).cloned())
    }

    pub(crate) fn may_apply(&self, agent_id: &AgentId) -> io::Result<bool> {
        Ok(self
            .state()?
            .owners
            .get(agent_id)
            .is_none_or(|owner| owner.qualifying_delivered && !owner.cancelled))
    }

    pub(crate) fn reserve(
        &self,
        agent_id: &AgentId,
        proposal: &Proposal,
    ) -> io::Result<Reservation> {
        let mut state = self.state()?;
        if state.shutting_down
            || state
                .owners
                .get(agent_id)
                .is_some_and(|owner| owner.cancelled)
        {
            return Err(invalid(
                "cancelled author cannot start or replay a private preview",
            ));
        }
        if state.prepared.contains_key(agent_id) {
            return Err(invalid(
                "private preview cannot overlap a prepared or provisional relaunch",
            ));
        }
        let generation = state
            .owners
            .get(agent_id)
            .ok_or_else(|| invalid("private preview requires a successfully launched author"))?
            .generation;
        let key = (agent_id.clone(), generation, proposal.id.clone());
        if let Some(prior) = state.previews.get(&key) {
            let owner = state.owners.get(agent_id).expect("owner validated above");
            if owner.executing && owner.current_proposal.as_deref() != Some(&proposal.id) {
                return Err(invalid(
                    "replay cannot displace another outstanding preview",
                ));
            }
            if prior.proposal != *proposal {
                return Err(invalid("proposal identity was reused with different bytes"));
            }
            let token = ReservationToken {
                agent_id: agent_id.clone(),
                generation,
                proposal_id: proposal.id.clone(),
                cancel: Arc::clone(&prior.cancellation),
            };
            return prior
                .result
                .clone()
                .map(|text| Reservation::Replay(token, text))
                .ok_or_else(|| invalid("proposal already started without a replayable result"));
        }
        if state
            .owners
            .get(agent_id)
            .expect("owner validated above")
            .executing
        {
            return Err(invalid(
                "another private preview is still executing for this author",
            ));
        }
        if let Some(revision) = &proposal.revision_of {
            let predecessor = state
                .previews
                .get(&(agent_id.clone(), generation, revision.clone()));
            if predecessor.is_none_or(|previous| !previous.delivered) {
                return Err(invalid(
                    "revision must name a delivered proposal in this launch generation",
                ));
            }
        }
        if state.previews.len() >= 100_000 {
            return Err(invalid(
                "private preview registry reached its admitted bound",
            ));
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let owner = state
            .owners
            .get_mut(agent_id)
            .expect("owner validated above");
        owner.cancellation = Some(Arc::clone(&cancel));
        owner.executing = true;
        owner.current_proposal = Some(proposal.id.clone());
        state.previews.insert(
            key,
            Preview {
                proposal: proposal.clone(),
                result: None,
                qualifying: false,
                delivered: false,
                cancellation: Arc::clone(&cancel),
            },
        );
        Ok(Reservation::New(ReservationToken {
            agent_id: agent_id.clone(),
            generation,
            proposal_id: proposal.id.clone(),
            cancel,
        }))
    }

    pub(crate) fn executed(
        &self,
        token: &ReservationToken,
        result: String,
        qualifying: bool,
    ) -> io::Result<()> {
        token.ensure_not_cancelled()?;
        let mut state = self.state()?;
        if state
            .owners
            .get(&token.agent_id)
            .is_some_and(|owner| owner.cancelled)
        {
            return Err(invalid("cancelled owner cannot publish a private result"));
        }
        let generation = state
            .owners
            .get(&token.agent_id)
            .ok_or_else(|| invalid("preview owner disappeared"))?
            .generation;
        if generation != token.generation {
            return Err(invalid(
                "preview execution belongs to a retired launch generation",
            ));
        }
        let preview = state
            .previews
            .get_mut(&(
                token.agent_id.clone(),
                generation,
                token.proposal_id.clone(),
            ))
            .ok_or_else(|| invalid("preview execution has no reserved proposal"))?;
        if preview.result.is_some() || !Arc::ptr_eq(&preview.cancellation, &token.cancel) {
            return Err(invalid("preview result cannot be replaced"));
        }
        preview.result = Some(result);
        preview.qualifying = qualifying;
        Ok(())
    }

    pub(crate) fn delivered(&self, token: &ReservationToken) -> io::Result<()> {
        token.ensure_not_cancelled()?;
        let mut state = self.state()?;
        if state
            .owners
            .get(&token.agent_id)
            .is_some_and(|owner| owner.cancelled)
        {
            return Err(invalid("cancelled owner cannot accept private delivery"));
        }
        let generation = state
            .owners
            .get(&token.agent_id)
            .ok_or_else(|| invalid("preview delivery owner disappeared"))?
            .generation;
        if generation != token.generation {
            return Err(invalid(
                "preview delivery belongs to a retired launch generation",
            ));
        }
        let preview = state
            .previews
            .get_mut(&(
                token.agent_id.clone(),
                generation,
                token.proposal_id.clone(),
            ))
            .ok_or_else(|| invalid("preview delivery belongs to an absent launch/proposal"))?;
        if preview.result.is_none() || !Arc::ptr_eq(&preview.cancellation, &token.cancel) {
            return Err(invalid("preview has no matching executed result"));
        }
        preview.delivered = true;
        let qualifying = preview.qualifying;
        let owner = state
            .owners
            .get_mut(&token.agent_id)
            .expect("owner validated above");
        owner.qualifying_delivered |= qualifying;
        if owner.current_proposal.as_deref() == Some(&token.proposal_id) {
            owner.executing = false;
            owner.current_proposal = None;
        }
        Ok(())
    }

    pub(crate) fn cancel(&self, agent_id: &AgentId) -> io::Result<()> {
        let mut state = self.state()?;
        if let Some(ticket) = state.prepared.get_mut(agent_id) {
            ticket.cancelled = true;
        }
        if let Some(owner) = state.owners.get_mut(agent_id) {
            owner.cancelled = true;
            if let Some(cancel) = &owner.cancellation {
                cancel.store(true, Ordering::Release);
            }
        }
        Ok(())
    }

    pub(crate) fn cancel_all(&self) {
        if let Ok(mut state) = self.state() {
            state.shutting_down = true;
            for ticket in state.prepared.values_mut() {
                ticket.cancelled = true;
            }
            for owner in state.owners.values_mut() {
                owner.cancelled = true;
                if let Some(cancel) = &owner.cancellation {
                    cancel.store(true, Ordering::Release);
                }
            }
        }
    }
}

impl LaunchGuard {
    #[cfg(test)]
    pub(crate) fn role(&self) -> Role {
        self.role
    }
    #[cfg(test)]
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    #[cfg(test)]
    pub(crate) fn mark_injected(&self, launch: &AgentLaunch) -> io::Result<()> {
        mark_injected(&self.registry, &self.launch.id, self.generation, launch)
    }

    pub(crate) fn commit(mut self) -> io::Result<()> {
        {
            let mut state = self.registry.state()?;
            let ticket = state
                .prepared
                .get(&self.launch.id)
                .ok_or_else(|| invalid("successful launch lost its provisional owner"))?;
            if ticket.generation != self.generation || !ticket.injected || ticket.cancelled {
                return Err(invalid(
                    "successful launch did not use its owned policy injection",
                ));
            }
            state.prepared.remove(&self.launch.id);
            if self.role == Role::Author {
                state.owners.insert(
                    self.launch.id.clone(),
                    Owner {
                        generation: self.generation,
                        qualifying_delivered: false,
                        cancellation: None,
                        executing: false,
                        current_proposal: None,
                        cancelled: false,
                    },
                );
            } else {
                state.owners.remove(&self.launch.id);
            }
        }
        self.committed = true;
        Ok(())
    }
}

impl Drop for LaunchGuard {
    fn drop(&mut self) {
        INJECTION.with(|slot| {
            if slot.borrow().as_ref().is_some_and(|context| {
                Arc::ptr_eq(&context.registry.0, &self.registry.0)
                    && context.generation == self.generation
            }) {
                *slot.borrow_mut() = None;
            }
        });
        if !self.committed
            && let Ok(mut state) = self.registry.state()
            && state
                .prepared
                .get(&self.launch.id)
                .is_some_and(|ticket| ticket.generation == self.generation)
        {
            state.prepared.remove(&self.launch.id);
        }
    }
}

#[cfg(test)]
#[path = "bench_private_test_first_tests.rs"]
mod tests;

#[cfg(all(test, target_os = "linux"))]
#[path = "bench_private_test_first_integration_tests.rs"]
mod integration_tests;
