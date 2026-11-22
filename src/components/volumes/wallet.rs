// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use super::projects::{aud_minor_units, draft_party_label, ContributionLifecyclePreview};
use crate::state::ShellState;
use aether::prelude::*;

#[component]
pub fn WalletVolume(state: Signal<ShellState>) -> Element {
    let daemon = state.read().daemon_label.clone();
    rsx! {
        div {
            class: "settings-volume",
            header {
                class: "settings-head",
                h3 { "Wallet · Lightning · Nym · eCash (XEC) · Tokens & Permissive Commons" }
                p { "Chain policy: Prefer Lightning, Nym Mixnet, eCash (XEC), and semantic tokens (QualiaPQ-Ledger / ML-DSA-65). ETH is not a target. Wallet handle != who; an AI instrument may propose as a tool, but only a human signs via KeyVault." }
            }
            p { class: "muted", "Qualia KeyVault sidecar status: {daemon}" }
            ContributionLifecyclePreview {}
            ProviderCreditInspector {}
            div {
                class: "plane-grid",
                div { class: "plane-card", h5 { "Lightning Network [Live]" } p { "Instant satoshi invoices & settlement after explicit human KeyVault signature." } }
                div { class: "plane-card", h5 { "Nym Mixnet [Live]" } p { "Privacy-routed mixnet credentials bound to KeyVault mixnet role." } }
                div { class: "plane-card", h5 { "eCash (XEC) & SLP RDF [Live]" } p { "Semantic token & RDF ledger settlement (xec-slp-rdf / semantic-tokens-xec)." } }
                div { class: "plane-card", h5 { "Two-Loop Permissive Commons [Live]" } p { "Pre-funded credit allocations (no overdraft/loan conflation) & 9-stage obligation discharge." } }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct CreditPeriod {
    opening_minor: i64,
    drawdown_minor: i64,
}

impl CreditPeriod {
    fn new(opening_minor: i64) -> Self {
        Self {
            opening_minor,
            drawdown_minor: 0,
        }
    }

    fn remainder(self) -> i64 {
        self.opening_minor - self.drawdown_minor
    }
}

#[derive(Clone)]
enum UnusedCreditDisposition {
    Carry,
    Return,
    Delegate(String),
}

#[derive(Clone)]
struct ReconciledPeriod {
    credit: CreditPeriod,
    disposition: UnusedCreditDisposition,
}

#[derive(Clone)]
struct ProviderAllocation {
    provider: String,
    authorized_backing_minor: i64,
    open_period: Option<CreditPeriod>,
    reconciliation: Vec<ReconciledPeriod>,
}

impl ProviderAllocation {
    fn new(provider: &str, backing: &str) -> Result<Self, &'static str> {
        let provider = draft_party_label(provider)?.to_owned();
        let authorized_backing_minor = aud_minor_units(backing)?;
        Ok(Self {
            provider,
            authorized_backing_minor,
            open_period: Some(CreditPeriod::new(authorized_backing_minor)),
            reconciliation: Vec::new(),
        })
    }

    fn draw(&mut self, raw_amount: &str) -> Result<(), &'static str> {
        let amount = aud_minor_units(raw_amount)?;
        let period = self
            .open_period
            .as_mut()
            .ok_or("This allocation has been closed.")?;
        if amount > period.remainder() {
            return Err("Drawdown exceeds this provider’s unused backing.");
        }
        period.drawdown_minor += amount;
        Ok(())
    }

    fn reconcile(&mut self, disposition: UnusedCreditDisposition) -> Result<(), &'static str> {
        let credit = self.open_period.ok_or("This allocation has been closed.")?;
        let remainder = credit.remainder();
        let next_period = match &disposition {
            UnusedCreditDisposition::Carry if remainder > 0 => Some(CreditPeriod::new(remainder)),
            UnusedCreditDisposition::Carry => return Err("There is no unused backing to carry."),
            UnusedCreditDisposition::Return => None,
            UnusedCreditDisposition::Delegate(recipient) => {
                if remainder == 0 {
                    return Err("There is no unused backing to reserve for delegation.");
                }
                if draft_party_label(recipient)?.eq_ignore_ascii_case(&self.provider) {
                    return Err("Choose a different recipient for delegation.");
                }
                None
            }
        };
        self.reconciliation.push(ReconciledPeriod {
            credit,
            disposition,
        });
        self.open_period = next_period;
        Ok(())
    }
}

#[component]
fn ProviderCreditInspector() -> Element {
    let mut allocations = use_signal(Vec::<ProviderAllocation>::new);
    let mut provider_name = use_signal(String::new);
    let mut backing = use_signal(String::new);
    let mut drawdown = use_signal(String::new);
    let mut delegation_recipient = use_signal(String::new);
    let mut status = use_signal(|| "");
    let snapshot = (*allocations.read()).clone();
    let has_open_allocations = snapshot
        .iter()
        .any(|allocation| allocation.open_period.is_some());
    let notice = status();
    let provider_value = provider_name();
    let backing_value = backing();
    let drawdown_value = drawdown();
    let recipient_value = delegation_recipient();

    let reconcile = move |index: usize, disposition: UnusedCreditDisposition| {
        let mut allocations = allocations;
        let mut status = status;
        let result = allocations
            .write()
            .get_mut(index)
            .ok_or("Select a provider allocation.")
            .and_then(|allocation| allocation.reconcile(disposition));
        status.set(
            result
                .err()
                .unwrap_or("Provider-attributed period reconciliation recorded in the draft."),
        );
    };

    rsx! {
        section {
            class: "id-list",
            h4 { "Pre-funded provider credit and periodic reconciliation" }
            p { "Local draft for this open volume. Valuation uses whole AUD cents (100 cents = 1 AUD). Effort minutes and scores remain separate records." }
            p { "Each provider’s drawdown is limited to their remaining authorized backing. A delegation reserves that provider’s unused credit for separate approval." }
            form {
                onsubmit: move |event| {
                    event.prevent_default();
                    let result = ProviderAllocation::new(&provider_name(), &backing()).and_then(|allocation| {
                        let mut records = allocations.write();
                        if records.iter().any(|row| row.provider.eq_ignore_ascii_case(&allocation.provider)) {
                            return Err("This provider already has an allocation in the draft.");
                        }
                        records.push(allocation);
                        Ok(())
                    });
                    status.set(match result {
                        Ok(()) => {
                            provider_name.set(String::new());
                            backing.set(String::new());
                            "Added provider backing to the draft."
                        }
                        Err(message) => message,
                    });
                },
                label {
                    "Provider "
                    input {
                        maxlength: 160,
                        value: provider_value,
                        oninput: move |event| provider_name.set(event.value()),
                    }
                }
                label {
                    "Authorized backing (AUD cents) "
                    input {
                        inputmode: "numeric",
                        value: backing_value,
                        oninput: move |event| backing.set(event.value()),
                    }
                }
                button { r#type: "submit", "Add draft allocation" }
            }
            if has_open_allocations {
                label {
                    "Drawdown amount (AUD cents) "
                    input {
                        inputmode: "numeric",
                        value: drawdown_value,
                        oninput: move |event| drawdown.set(event.value()),
                    }
                }
                label {
                    "Proposed delegation recipient "
                    input {
                        maxlength: 160,
                        value: recipient_value,
                        oninput: move |event| delegation_recipient.set(event.value()),
                    }
                }
            }
            for (index, allocation) in snapshot.into_iter().enumerate() {
                {
                    let authorized = allocation.authorized_backing_minor;
                    let provider = allocation.provider;
                    let period_number = allocation.reconciliation.len() + 1;
                    rsx! {
                        section {
                            key: "{index}",
                            class: "plane-card",
                            h5 { "{provider}" }
                            p { "Originally authorized: {authorized} AUD cents" }
                            if let Some(period) = allocation.open_period {
                                {
                                    let drawn = period.drawdown_minor;
                                    let unused = period.remainder();
                                    rsx! {
                                        p { "Period {period_number} · Drawn: {drawn} · Unused: {unused} AUD cents" }
                                        button {
                                            r#type: "button",
                                            disabled: unused == 0,
                                            onclick: move |_| {
                                                let result = allocations.write()
                                                    .get_mut(index)
                                                    .ok_or("Select a provider allocation.")
                                                    .and_then(|allocation| allocation.draw(&drawdown()));
                                                status.set(match result {
                                                    Ok(()) => {
                                                        drawdown.set(String::new());
                                                        "Drawdown recorded against this provider’s backing in the draft."
                                                    }
                                                    Err(message) => message,
                                                });
                                            },
                                            "Record draft drawdown"
                                        }
                                        button {
                                            r#type: "button",
                                            disabled: unused == 0,
                                            onclick: move |_| reconcile(index, UnusedCreditDisposition::Carry),
                                            "Carry to next period"
                                        }
                                        button {
                                            r#type: "button",
                                            onclick: move |_| reconcile(index, UnusedCreditDisposition::Return),
                                            "Reconcile and return unused backing"
                                        }
                                        button {
                                            r#type: "button",
                                            disabled: unused == 0,
                                            onclick: move |_| {
                                                let recipient = delegation_recipient();
                                                reconcile(index, UnusedCreditDisposition::Delegate(recipient.trim().to_owned()));
                                            },
                                            "Reserve unused backing for delegation"
                                        }
                                    }
                                }
                            } else {
                                p { "Allocation reconciled and closed." }
                            }
                            if !allocation.reconciliation.is_empty() {
                                ol {
                                    for (period_index, period) in allocation.reconciliation.into_iter().enumerate() {
                                        {
                                            let number = period_index + 1;
                                            let opening = period.credit.opening_minor;
                                            let drawn = period.credit.drawdown_minor;
                                            let remaining = period.credit.remainder();
                                            let outcome = match period.disposition {
                                                UnusedCreditDisposition::Carry => "Carried to the next period".to_owned(),
                                                UnusedCreditDisposition::Return => "Earmarked for return to the provider".to_owned(),
                                                UnusedCreditDisposition::Delegate(recipient) => format!("Reserved for proposed delegation to {recipient}"),
                                            };
                                            rsx! {
                                                li {
                                                    key: "{period_index}",
                                                    "Period {number} · Opening: {opening} · Drawn: {drawn} · Unused: {remaining} AUD cents · {outcome}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !notice.is_empty() {
                p { role: "status", "{notice}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carried_provider_backing_cannot_be_spent_twice() {
        let mut provider = ProviderAllocation::new("Site steward", "1000").unwrap();
        provider.draw("600").unwrap();
        assert!(provider.draw("401").is_err());
        assert_eq!(provider.open_period.unwrap().remainder(), 400);
        provider.reconcile(UnusedCreditDisposition::Carry).unwrap();
        assert_eq!(provider.authorized_backing_minor, 1000);
        assert_eq!(provider.open_period.unwrap().opening_minor, 400);
        provider.draw("300").unwrap();
        provider.reconcile(UnusedCreditDisposition::Return).unwrap();
        assert!(provider.open_period.is_none());
        assert_eq!(provider.reconciliation[1].credit.remainder(), 100);
        assert!(provider.draw("1").is_err());
    }

    #[test]
    fn proposed_delegation_preserves_the_origin_and_reserves_unused_credit() {
        let mut provider = ProviderAllocation::new("Site steward", "1000").unwrap();
        provider.draw("125").unwrap();
        assert!(provider
            .reconcile(UnusedCreditDisposition::Delegate("site steward".into()))
            .is_err());
        assert!(provider.reconciliation.is_empty());
        assert_eq!(provider.open_period.unwrap().remainder(), 875);
        provider
            .reconcile(UnusedCreditDisposition::Delegate(
                "Member collective".into(),
            ))
            .unwrap();
        assert_eq!(provider.provider, "Site steward");
        let period = &provider.reconciliation[0];
        assert_eq!(period.credit.remainder(), 875);
        assert!(
            matches!(&period.disposition, UnusedCreditDisposition::Delegate(recipient) if recipient == "Member collective")
        );
        assert!(provider.draw("1").is_err());
    }
}
