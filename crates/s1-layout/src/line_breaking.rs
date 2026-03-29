//! Line breaking algorithms — Knuth-Plass optimal and greedy fallback.

/// Item types for the Knuth-Plass line breaking algorithm.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum BreakItem {
    /// Content with a fixed width (a shaped run or sub-range of one).
    Box {
        run_idx: usize,
        width: f64,
        height: f64,
        /// Start glyph index within the run (inclusive).
        glyph_start: usize,
        /// End glyph index within the run (exclusive).
        glyph_end: usize,
        /// Start byte offset within the run's text.
        text_byte_start: usize,
        /// End byte offset within the run's text.
        text_byte_end: usize,
    },
    /// Stretchable/shrinkable space between boxes.
    Glue {
        width: f64,
        stretch: f64,
        shrink: f64,
    },
    /// A possible break point with a penalty cost.
    Penalty {
        penalty: f64,
        /// If true, a hyphen should be inserted when the line breaks here.
        flagged: bool,
    },
    /// A forced line break (from LineBreak node).
    ForcedBreak {
        #[allow(dead_code)]
        run_idx: usize,
    },
}

/// Knuth-Plass optimal line breaking.
///
/// Returns break indices into the items array, or `None` if the algorithm
/// cannot find a feasible solution (falls back to greedy).
pub(crate) fn knuth_plass_breaks(
    items: &[BreakItem],
    available_width: f64,
    first_line_indent: f64,
) -> Option<Vec<usize>> {
    if items.is_empty() {
        return Some(vec![0, 0]);
    }

    // Active node: (item_index, line_number, total_demerits, total_width)
    #[derive(Clone)]
    struct ActiveNode {
        index: usize,
        line: usize,
        demerits: f64,
        total_width: f64,
        prev: Option<usize>, // index into nodes vec
    }

    let mut nodes: Vec<ActiveNode> = vec![ActiveNode {
        index: 0,
        line: 0,
        demerits: 0.0,
        total_width: 0.0,
        prev: None,
    }];
    let mut active: Vec<usize> = vec![0]; // indices into nodes

    for (i, item) in items.iter().enumerate() {
        // Determine if this item is a feasible break point and its penalty
        let (is_feasible_break, penalty_cost) = match item {
            BreakItem::Glue { .. } => (true, 0.0),
            BreakItem::ForcedBreak { .. } => (true, 0.0),
            BreakItem::Penalty { penalty, .. } => (true, *penalty),
            _ => (false, 0.0),
        };

        if !is_feasible_break {
            continue;
        }

        let mut new_active: Vec<usize> = Vec::new();
        let mut best_node: Option<ActiveNode> = None;

        for &a_idx in &active {
            let a = &nodes[a_idx];

            // Compute width from this active node to current position
            let mut width = a.total_width;
            for item_between in &items[a.index..i] {
                match item_between {
                    BreakItem::Box { width: w, .. } => width += w,
                    BreakItem::Glue { width: w, .. } => width += w,
                    _ => {}
                }
            }

            // Line width depends on whether this is the first line
            let line_width = if a.line == 0 {
                available_width - first_line_indent
            } else {
                available_width
            };

            let ratio = line_width - (width - a.total_width);

            // Check feasibility: allow lines to be slightly overfull (5%)
            if ratio >= -line_width * 0.05 {
                let badness = if ratio.abs() < 0.01 {
                    0.0
                } else if ratio > 0.0 {
                    // Underfull
                    (100.0 * (ratio / line_width).powi(3)).min(10000.0)
                } else {
                    // Overfull
                    10000.0
                };

                let is_forced = matches!(item, BreakItem::ForcedBreak { .. });

                // Standard Knuth-Plass demerit calculation:
                // Forced breaks get minimal demerits, penalties add their cost
                let demerits = if is_forced {
                    a.demerits
                } else {
                    (1.0 + badness + penalty_cost).powi(2) + a.demerits
                };

                match &best_node {
                    None => {
                        best_node = Some(ActiveNode {
                            index: i + 1,
                            line: a.line + 1,
                            demerits,
                            total_width: width,
                            prev: Some(a_idx),
                        });
                    }
                    Some(best) if demerits < best.demerits => {
                        best_node = Some(ActiveNode {
                            index: i + 1,
                            line: a.line + 1,
                            demerits,
                            total_width: width,
                            prev: Some(a_idx),
                        });
                    }
                    _ => {}
                }

                // For forced breaks, deactivate the current node (must break here).
                // For regular breaks, keep the node active if the line isn't too long.
                if !is_forced && ratio > -line_width * 0.05 {
                    new_active.push(a_idx);
                }
            } else {
                // Line too long — deactivate
            }
        }

        if let Some(node) = best_node {
            let idx = nodes.len();
            nodes.push(node);
            new_active.push(idx);
        }

        if !new_active.is_empty() {
            active = new_active;
        }
        // L-21: Cap active node count to prevent unbounded growth on very long paragraphs.
        // Keep only the best 100 candidates by demerits.
        if active.len() > 100 {
            active.sort_by(|&a, &b| {
                nodes[a]
                    .demerits
                    .partial_cmp(&nodes[b].demerits)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            active.truncate(100);
        }
        // If active becomes empty, KP fails — return None for greedy fallback
        if active.is_empty() {
            return None;
        }
    }

    // Add a final break at the end of items
    let final_idx = items.len();
    let mut best_final: Option<ActiveNode> = None;
    for &a_idx in &active {
        let a = &nodes[a_idx];
        let mut width = a.total_width;
        for item_between in &items[a.index..final_idx] {
            match item_between {
                BreakItem::Box { width: w, .. } => width += w,
                BreakItem::Glue { width: w, .. } => width += w,
                _ => {}
            }
        }
        let demerits = a.demerits;
        match &best_final {
            None => {
                best_final = Some(ActiveNode {
                    index: final_idx,
                    line: a.line + 1,
                    demerits,
                    total_width: width,
                    prev: Some(a_idx),
                });
            }
            Some(best) if demerits < best.demerits => {
                best_final = Some(ActiveNode {
                    index: final_idx,
                    line: a.line + 1,
                    demerits,
                    total_width: width,
                    prev: Some(a_idx),
                });
            }
            _ => {}
        }
    }

    let final_node = best_final?;
    let final_node_idx = nodes.len();
    nodes.push(final_node);

    // Trace back to get break points
    let mut breaks = Vec::new();
    let mut current = Some(final_node_idx);
    while let Some(idx) = current {
        breaks.push(nodes[idx].index);
        current = nodes[idx].prev;
    }
    breaks.reverse();

    // Ensure we start at 0
    if breaks.first() != Some(&0) {
        breaks.insert(0, 0);
    }

    Some(breaks)
}

/// Greedy line breaking fallback.
pub(crate) fn greedy_breaks(
    items: &[BreakItem],
    available_width: f64,
    first_line_indent: f64,
) -> Vec<usize> {
    let mut breaks = vec![0];
    let mut current_width = 0.0;
    let mut is_first_line = true;
    // Track the last feasible break point (Glue or Penalty) for deferred breaking
    let mut last_break_opportunity: Option<usize> = None;
    let mut width_at_last_break: f64 = 0.0;

    for (i, item) in items.iter().enumerate() {
        match item {
            BreakItem::Box { width, .. } => {
                let line_w = if is_first_line {
                    available_width - first_line_indent
                } else {
                    available_width
                };

                if current_width + width > line_w + 0.01 && i > *breaks.last().unwrap_or(&0) {
                    // If we have a previous break opportunity, break there instead
                    if let Some(bp) = last_break_opportunity {
                        if bp > *breaks.last().unwrap_or(&0) {
                            breaks.push(bp + 1);
                            // Subtract the glue width at bp so the new line doesn't
                            // inherit the trailing space from the previous line.
                            let glue_w = match &items[bp] {
                                BreakItem::Glue { width, .. } => *width,
                                _ => 0.0,
                            };
                            current_width = current_width - width_at_last_break - glue_w + width;
                            is_first_line = false;
                            last_break_opportunity = None;
                            continue;
                        }
                    }
                    breaks.push(i);
                    current_width = *width;
                    is_first_line = false;
                    last_break_opportunity = None;
                } else {
                    current_width += width;
                }
            }
            BreakItem::Glue { width, .. } => {
                last_break_opportunity = Some(i);
                width_at_last_break = current_width;
                current_width += width;
            }
            BreakItem::ForcedBreak { .. } => {
                breaks.push(i + 1);
                current_width = 0.0;
                is_first_line = false;
                last_break_opportunity = None;
            }
            BreakItem::Penalty { .. } => {
                // Penalty is a valid break opportunity (e.g. hyphenation point)
                last_break_opportunity = Some(i);
                width_at_last_break = current_width;
            }
        }
    }

    breaks.push(items.len());
    breaks
}
