//! Renders an [`NftRuleset`] AST into the exact text handed to `nft -f`.
//!
//! Kept as a separate step from [`super::nft::build_ruleset`] so the
//! *shape* of the ruleset (chain order, rule order) and its *textual
//! syntax* can be tested independently, and so `anonveil-priv` never has
//! to string-format a single rule itself — it only ever calls this
//! function and writes the result to a file for `nft -c -f` / `nft -f`.

use super::types::{NftChain, NftRuleset, NftSet};

/// Render a ruleset to nftables script syntax, ready to be written to a
/// file and loaded with `nft -f <path>` (after a `nft -c -f <path>`
/// dry-run check, performed by `anonveil-priv`, not here).
pub fn render(ruleset: &NftRuleset) -> String {
    let mut out = String::new();
    out.push_str("# Managed by AnonVeil — do not edit by hand.\n");
    out.push_str("# Regenerate via the AnonVeil config, not manually.\n");
    out.push_str(&format!(
        "table {} {} {{\n",
        ruleset.table_family, ruleset.table_name
    ));

    for set in &ruleset.sets {
        render_set(&mut out, set);
    }
    if !ruleset.sets.is_empty() {
        out.push('\n');
    }

    for (i, chain) in ruleset.chains.iter().enumerate() {
        render_chain(&mut out, chain);
        if i + 1 < ruleset.chains.len() {
            out.push('\n');
        }
    }

    out.push_str("}\n");
    out
}

fn render_set(out: &mut String, set: &NftSet) {
    out.push_str(&format!("    set {} {{\n", set.name));
    out.push_str(&format!("        type {}\n", set.set_type));
    if set.interval {
        out.push_str("        flags interval\n");
    }
    let elements = set
        .elements
        .iter()
        .map(|e| e.0.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!("        elements = {{ {elements} }}\n"));
    out.push_str("    }\n");
}

fn render_chain(out: &mut String, chain: &NftChain) {
    out.push_str(&format!("    chain {} {{\n", chain.name));
    if let Some(hook) = &chain.hook {
        out.push_str(&format!(
            "        type {} hook {} priority {}; policy {};\n",
            hook.chain_type, hook.hook, hook.priority, hook.policy
        ));
    }
    for rule in &chain.rules {
        out.push_str(&format!("        {rule}\n"));
    }
    out.push_str("    }\n");
}
