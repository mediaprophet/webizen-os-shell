// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use crate::state::{contribution_lens, ShellState, WebizenRefractModel};
use aether::prelude::*;
use aether::refract::{ContributionStage, ContributionStageView};
use webizen_ua_os::validate_human_centric_host_nomenclature;

#[component]
pub fn ProjectsVolume(state: Signal<ShellState>) -> Element {
    let vault = state.read().storage_display.clone();
    rsx! {
        div {
            class: "settings-volume",
            header {
                class: "settings-head",
                h3 { "Projects · Cooperative Work-Graph & Nine-Stage Contribution Lifecycle" }
                p {
                    "Transport-neutral cooperative project ledger (qualia-cooperative-core & work-graph). Effort (integer minutes) and capital/reimbursable expenses (signed minor units, banker's rounding) merge add-wins by stable entry ID so replayed sync frames never double-count obligations."
                }
            }
            section {
                class: "id-list",
                h4 { "Nine-Stage Contribution, Valuation, Obligation & Licence-Transition Pipeline" }
                ul {
                    li { "Stage 1: Request & Agreement (Commissioned, OpenBounty, CooperativeMandate, StewardAuthorised; ContractedService, ConditionalPoolShare, VoluntaryCommonsGift)" }
                    li { "Stage 2: Activity & Resources (effort minutes, reimbursable expenses, and capital advanced logged without prematurely minting debt)" }
                    li { "Stages 3-5: Contextual Assertion Review, Steward Acceptance (M-of-N Ed25519 + ML-DSA-65 quorum), and Bounded Obligation Derivation" }
                    li { "Stages 6-8: Pre-Funded Credit Allocation Drawdown (no automatic overdraft), Periodic Reconciliation, and Two-Loop Permissive Commons Settlement" }
                    li { "Stage 9: Verified Obligation Discharge & Authorised Licence-Transition Decision (evaluated by licence-transition-readiness Semantic Instrument)" }
                }
            }
            ProjectObligationLedger {}
            details {
                summary { "Shared contribution example" }
                ContributionLifecyclePreview {}
            }
            p { class: "muted", "Local work-graph & .q42j journal root: {vault}" }
        }
    }
}

const SAMPLE_OBLIGATION_MINOR: i64 = 10_000;

pub(super) fn draft_party_label(raw: &str) -> Result<&str, &'static str> {
    let label = raw.trim();
    if label.is_empty() || label.len() > 160 || label.chars().any(char::is_control) {
        return Err("Enter a party label of up to 160 bytes.");
    }
    validate_human_centric_host_nomenclature(label)
        .map_err(|_| "Use Human-Centric terminology for the party label.")?;
    Ok(label)
}

pub(super) fn aud_minor_units(raw: &str) -> Result<i64, &'static str> {
    let raw = raw.trim();
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("Enter a positive whole number of AUD cents.");
    }
    match raw.parse() {
        Ok(value) if value > 0 => Ok(value),
        _ => Err("Enter a positive amount within the supported AUD range."),
    }
}

#[derive(Clone)]
struct ContributorDraft {
    label: String,
    obligation_minor: i64,
    settled_minor: i64,
    no_charge_uses: u64,
}

#[derive(Clone, Default)]
struct ObligationPoolDraft {
    contributors: Vec<ContributorDraft>,
    funded_minor: i64,
    transition_approved: bool,
}

impl ObligationPoolDraft {
    fn sample() -> Self {
        let contributions = [
            ("Design contributor", SAMPLE_OBLIGATION_MINOR * 3 / 5),
            ("Fieldwork contributor", SAMPLE_OBLIGATION_MINOR * 2 / 5),
        ];
        Self {
            contributors: contributions
                .into_iter()
                .map(|(label, obligation_minor)| ContributorDraft {
                    label: label.into(),
                    obligation_minor,
                    settled_minor: 0,
                    no_charge_uses: 0,
                })
                .collect(),
            ..Self::default()
        }
    }

    fn obligation_minor(&self) -> i64 {
        self.contributors
            .iter()
            .map(|row| row.obligation_minor)
            .sum()
    }

    fn settled_minor(&self) -> i64 {
        self.contributors.iter().map(|row| row.settled_minor).sum()
    }

    fn view(&self) -> ContributionStageView {
        let target = self.obligation_minor();
        let settled = self.settled_minor();
        let stage = if self.transition_approved {
            ContributionStage::TransitionDecision
        } else if settled > 0 {
            ContributionStage::SettlementAllocation
        } else if target > 0 {
            ContributionStage::Obligation
        } else {
            ContributionStage::RequestAndAgreement
        };
        ContributionStageView {
            current_stage: stage,
            target_threshold_cents: target,
            pool_funded_cents: self.funded_minor,
            contributor_settled_cents: settled,
            waived_obligation_cents: 0,
            unresolved_challenges: 0,
            human_transition_approved: self.transition_approved,
        }
    }

    fn add_contributor(&mut self, raw_label: &str, raw_amount: &str) -> Result<(), &'static str> {
        if self.transition_approved {
            return Err("The approved draft is closed to new obligations.");
        }
        let label = draft_party_label(raw_label)?;
        let amount = aud_minor_units(raw_amount)?;
        if self
            .contributors
            .iter()
            .any(|row| row.label.eq_ignore_ascii_case(label))
        {
            return Err("This contributor already has an attributed obligation.");
        }
        self.obligation_minor()
            .checked_add(amount)
            .ok_or("The total obligation would exceed the supported AUD range.")?;
        self.contributors.push(ContributorDraft {
            label: label.into(),
            obligation_minor: amount,
            settled_minor: 0,
            no_charge_uses: 0,
        });
        Ok(())
    }

    fn credit_pool(&mut self, raw_amount: &str) -> Result<(), &'static str> {
        let amount = aud_minor_units(raw_amount)?;
        if amount > self.obligation_minor() - self.funded_minor {
            return Err("Funding is bounded by the remaining project obligation.");
        }
        self.funded_minor += amount;
        Ok(())
    }

    fn settle_contributor(&mut self, index: usize, raw_amount: &str) -> Result<(), &'static str> {
        let amount = aud_minor_units(raw_amount)?;
        let available = self.funded_minor - self.settled_minor();
        let row = self
            .contributors
            .get_mut(index)
            .ok_or("Select a contributor.")?;
        if amount > available || amount > row.obligation_minor - row.settled_minor {
            return Err("Settlement is bounded by unspent funding and this contributor’s balance.");
        }
        row.settled_minor += amount;
        Ok(())
    }

    fn record_no_charge_use(&mut self, index: usize) -> Result<(), &'static str> {
        let row = self
            .contributors
            .get_mut(index)
            .ok_or("Select a contributor.")?;
        row.no_charge_uses = row
            .no_charge_uses
            .checked_add(1)
            .ok_or("Use count is full.")?;
        Ok(())
    }

    fn approve_transition(&mut self) -> Result<(), &'static str> {
        let view = self.view();
        if view.target_threshold_cents == 0 || !view.is_contributor_fully_settled() {
            return Err("Every attributed obligation must be settled before draft approval.");
        }
        self.transition_approved = true;
        Ok(())
    }
}

#[component]
fn ProjectObligationLedger() -> Element {
    let mut draft = use_signal(ObligationPoolDraft::default);
    let mut contributor = use_signal(String::new);
    let mut obligation = use_signal(String::new);
    let mut pool_credit = use_signal(String::new);
    let mut drawdown = use_signal(String::new);
    let mut status = use_signal(|| "");
    let snapshot = (*draft.read()).clone();
    let view = snapshot.view();
    let available = snapshot.funded_minor - view.contributor_settled_cents;
    let no_rows = snapshot.contributors.is_empty();
    let notice = status();
    let contributor_value = contributor();
    let obligation_value = obligation();
    let pool_credit_value = pool_credit();
    let drawdown_value = drawdown();

    rsx! {
        section {
            class: "id-list",
            h4 { "Permissive Commons obligation pool and contributor attribution" }
            p { "Local draft for this open volume. Amounts are whole AUD cents. A payout reduces the attributed contributor’s outstanding balance. Human Rights no-charge use has a separate counter." }
            {render_contribution_lifecycle_bar(&view)}
            p { "Unspent commercial backing: {available} AUD cents" }
            form {
                onsubmit: move |event| {
                    event.prevent_default();
                    let result = draft.write().add_contributor(&contributor(), &obligation());
                    status.set(match result {
                        Ok(()) => {
                            contributor.set(String::new());
                            obligation.set(String::new());
                            "Contributor obligation added to the draft."
                        }
                        Err(message) => message,
                    });
                },
                label {
                    "Originating contributor "
                    input {
                        value: contributor_value,
                        maxlength: 160,
                        oninput: move |event| contributor.set(event.value()),
                    }
                }
                label {
                    "Obligation (AUD cents) "
                    input {
                        inputmode: "numeric",
                        value: obligation_value,
                        oninput: move |event| obligation.set(event.value()),
                    }
                }
                button { r#type: "submit", disabled: view.human_transition_approved, "Add draft obligation" }
                if no_rows {
                    button {
                        r#type: "button",
                        onclick: move |_| {
                            let is_empty = draft.peek().contributors.is_empty();
                            if is_empty {
                                draft.set(ObligationPoolDraft::sample());
                                status.set("Loaded a two-contributor example.");
                            }
                        },
                        "Load example"
                    }
                }
            }
            form {
                onsubmit: move |event| {
                    event.prevent_default();
                    let result = draft.write().credit_pool(&pool_credit());
                    status.set(match result {
                        Ok(()) => {
                            pool_credit.set(String::new());
                            "Backing credited to the draft pool."
                        }
                        Err(message) => message,
                    });
                },
                label {
                    "Commercial backing (AUD cents) "
                    input {
                        inputmode: "numeric",
                        value: pool_credit_value,
                        oninput: move |event| pool_credit.set(event.value()),
                    }
                }
                button { r#type: "submit", disabled: no_rows || view.is_pool_threshold_met(), "Credit draft pool" }
            }
            if !no_rows {
                label {
                    "Payout amount (AUD cents) "
                    input {
                        inputmode: "numeric",
                        value: drawdown_value,
                        oninput: move |event| drawdown.set(event.value()),
                    }
                }
            }
            ul {
                class: "plane-grid",
                for (index, row) in snapshot.contributors.into_iter().enumerate() {
                    {
                        let label = row.label;
                        let outstanding = row.obligation_minor - row.settled_minor;
                        let settled = row.settled_minor;
                        let uses = row.no_charge_uses;
                        rsx! {
                            li {
                                key: "{index}",
                                class: "plane-card",
                                h5 { "{label}" }
                                p { "Settled in draft: {settled} · Outstanding: {outstanding} AUD cents" }
                                p { "Human Rights no-charge uses: {uses}" }
                                button {
                                    r#type: "button",
                                    disabled: available == 0 || outstanding == 0,
                                    onclick: move |_| {
                                        let result = draft.write().settle_contributor(index, &drawdown());
                                        status.set(match result {
                                            Ok(()) => {
                                                drawdown.set(String::new());
                                                "Payout recorded against the selected contributor in the draft."
                                            }
                                            Err(message) => message,
                                        });
                                    },
                                    "Record draft payout"
                                }
                                button {
                                    r#type: "button",
                                    onclick: move |_| {
                                        let result = draft.write().record_no_charge_use(index);
                                        status.set(result.err().unwrap_or("No-charge use recorded separately from settlement."));
                                    },
                                    "Record no-charge use"
                                }
                            }
                        }
                    }
                }
            }
            button {
                r#type: "button",
                disabled: no_rows || !view.is_contributor_fully_settled() || view.human_transition_approved,
                onclick: move |_| {
                    let result = draft.write().approve_transition();
                    status.set(result.err().unwrap_or("Commons transition approved in the local draft."));
                },
                "Approve Commons transition in draft"
            }
            if !notice.is_empty() {
                p { role: "status", "{notice}" }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum PreviewAction {
    Start,
    Fund,
    Settle,
    Approve,
}

impl PreviewAction {
    fn enabled(self, view: &ContributionStageView) -> bool {
        if view.human_transition_approved {
            return false;
        }
        if matches!(self, Self::Start) {
            return view.target_threshold_cents <= 0;
        }
        if view.target_threshold_cents <= 0 || view.current_stage < ContributionStage::Obligation {
            return false;
        }
        match self {
            Self::Start => false,
            Self::Fund => !view.is_pool_threshold_met(),
            Self::Settle => available_settlement(view) > 0,
            Self::Approve => {
                view.is_contributor_fully_settled()
                    && view.unresolved_challenges == 0
                    && matches!(
                        view.current_stage,
                        ContributionStage::SettlementAllocation
                            | ContributionStage::TransitionDecision
                    )
            }
        }
    }

    fn apply(self, view: &mut ContributionStageView) {
        if !self.enabled(view) {
            return;
        }
        match self {
            Self::Start => {
                *view = ContributionStageView {
                    current_stage: ContributionStage::Obligation,
                    target_threshold_cents: SAMPLE_OBLIGATION_MINOR,
                    pool_funded_cents: 0,
                    contributor_settled_cents: 0,
                    waived_obligation_cents: 0,
                    unresolved_challenges: 0,
                    human_transition_approved: false,
                };
            }
            Self::Fund => view.pool_funded_cents = view.target_threshold_cents,
            Self::Settle => {
                let payout = available_settlement(view);
                view.contributor_settled_cents += payout;
                view.current_stage = view
                    .current_stage
                    .max(ContributionStage::SettlementAllocation);
            }
            Self::Approve => {
                view.human_transition_approved = true;
                view.current_stage = ContributionStage::TransitionDecision;
            }
        }
    }
}

fn available_settlement(view: &ContributionStageView) -> i64 {
    view.pool_funded_cents
        .saturating_sub(view.contributor_settled_cents)
        .max(0)
        .min(view.unpaid_settled_gap_cents())
}

#[component]
pub(super) fn ContributionLifecyclePreview() -> Element {
    let Some(store) = aether::core::try_consume_context::<RefractStore<WebizenRefractModel>>()
    else {
        return aether::core::VNode::empty();
    };
    let view = store.get_cloned(contribution_lens());
    let actions = [
        (PreviewAction::Start, "Start sample obligation"),
        (PreviewAction::Fund, "Preview pool funding"),
        (PreviewAction::Settle, "Preview contributor settlement"),
        (
            PreviewAction::Approve,
            "Approve Commons transition in preview",
        ),
    ];
    let buttons = actions.into_iter().map(|(action, label)| {
        let store = store.clone();
        let disabled = !action.enabled(&view);
        rsx! {
            button {
                key: "{label}",
                r#type: "button",
                disabled,
                onclick: move |_| store.write(contribution_lens(), |view| action.apply(view)),
                "{label}"
            }
        }
    });

    rsx! {
        section {
            class: "id-list",
            h4 { "Contribution lifecycle preview" }
            p { "Funding, settlement, and approval are modelled in the shell’s local preview." }
            {render_contribution_lifecycle_bar(&view)}
            div { {buttons} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_funding_leaves_the_contributor_unpaid_until_settlement() {
        let mut view = WebizenRefractModel::placeholder().contribution;
        PreviewAction::Start.apply(&mut view);
        PreviewAction::Fund.apply(&mut view);
        assert_eq!(view.pool_funded_cents, SAMPLE_OBLIGATION_MINOR);
        assert_eq!(view.contributor_settled_cents, 0);
        PreviewAction::Approve.apply(&mut view);
        assert!(!view.can_transition_to_commons());

        PreviewAction::Settle.apply(&mut view);
        assert_eq!(view.contributor_settled_cents, SAMPLE_OBLIGATION_MINOR);
        assert!(!view.can_transition_to_commons());
        view.unresolved_challenges = 1;
        PreviewAction::Approve.apply(&mut view);
        assert!(!view.can_transition_to_commons());
        view.unresolved_challenges = 0;
        PreviewAction::Approve.apply(&mut view);
        assert!(view.can_transition_to_commons());
    }

    #[test]
    fn settlement_is_bounded_by_pool_funds_and_the_unpaid_gap() {
        let mut view = WebizenRefractModel::placeholder().contribution;
        PreviewAction::Start.apply(&mut view);
        view.pool_funded_cents = 2_500;
        PreviewAction::Settle.apply(&mut view);
        PreviewAction::Settle.apply(&mut view);
        assert_eq!(view.contributor_settled_cents, 2_500);
        assert_eq!(view.unpaid_settled_gap_cents(), 7_500);

        view.waived_obligation_cents = 1_000;
        PreviewAction::Fund.apply(&mut view);
        PreviewAction::Settle.apply(&mut view);
        assert_eq!(view.contributor_settled_cents, 9_000);
        assert_eq!(view.unpaid_settled_gap_cents(), 0);
    }

    #[test]
    fn attributed_balances_require_backing_and_explicit_payouts() {
        let mut draft = ObligationPoolDraft::sample();
        draft.credit_pool("10000").unwrap();
        assert!(draft.credit_pool("1").is_err());
        assert!(draft.view().is_pool_threshold_met());
        assert!(draft.approve_transition().is_err());

        let unpaid_before = draft.view().unpaid_settled_gap_cents();
        draft.record_no_charge_use(0).unwrap();
        assert_eq!(draft.contributors[0].no_charge_uses, 1);
        assert_eq!(draft.view().unpaid_settled_gap_cents(), unpaid_before);
        assert!(draft.settle_contributor(0, "6001").is_err());
        assert_eq!(draft.settled_minor(), 0);

        draft.settle_contributor(0, "6000").unwrap();
        assert_eq!(draft.contributors[1].settled_minor, 0);
        assert!(draft.approve_transition().is_err());
        draft.settle_contributor(1, "4000").unwrap();
        assert!(!draft.view().can_transition_to_commons());
        draft.approve_transition().unwrap();
        assert!(draft.view().can_transition_to_commons());
        assert!(draft.add_contributor("Another contributor", "200").is_err());
    }

    #[test]
    fn draft_rejects_currency_fractions_duplicates_and_amount_overflow() {
        let mut draft = ObligationPoolDraft::default();
        draft
            .add_contributor("Contributor", &i64::MAX.to_string())
            .unwrap();
        assert!(draft.add_contributor("contributor", "1").is_err());
        assert!(draft.add_contributor("Distinct origin", "1").is_err());
        assert!(aud_minor_units("1.1").is_err());
        assert!(aud_minor_units("-2").is_err());
        assert!(aud_minor_units("0").is_err());
        assert_eq!(draft.obligation_minor(), i64::MAX);
    }
}
