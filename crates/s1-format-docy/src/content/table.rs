use crate::constants::*;
use crate::writer::DocyWriter;
use crate::content::paragraph;
use crate::props::borders;
use s1_model::{DocumentModel, NodeType, NodeId, AttributeKey, AttributeValue};

/// Table type IDs matching c_oSerDocTableType in Serialize2.js:480
mod tbl {
    pub const TBL_PR: u8 = 0;
    pub const TBL_GRID: u8 = 1;
    pub const TBL_GRID_ITEM_TWIPS: u8 = 13;
    pub const CONTENT: u8 = 3;
    pub const ROW: u8 = 4;
    pub const ROW_CONTENT: u8 = 5;
    pub const CELL: u8 = 6;
    pub const CELL_PR: u8 = 7;
    pub const CELL_CONTENT: u8 = 8;
}

pub fn write(w: &mut DocyWriter, model: &DocumentModel, table_id: NodeId) {
    let table = match model.node(table_id) {
        Some(n) => n,
        None => return,
    };

    let rows: Vec<NodeId> = table.children.iter()
        .filter(|id| model.node(**id).map_or(false, |n| n.node_type == NodeType::TableRow))
        .copied()
        .collect();
    if rows.is_empty() { return; }

    let max_cols = rows.iter().map(|rid| {
        model.node(*rid).map_or(0, |r| {
            r.children.iter().filter(|id| {
                model.node(**id).map_or(false, |n| n.node_type == NodeType::TableCell)
            }).count()
        })
    }).max().unwrap_or(0);

    // tblPr — Read1 container, Read1 items inside
    w.write_item(tbl::TBL_PR, |w| {
        write_table_props(w, table);
    });

    // tblGrid — Read1 container, Read2 items inside. Use actual column widths.
    w.write_item(tbl::TBL_GRID, |w| {
        let col_widths: Vec<f64> = table.attributes
            .get_string(&AttributeKey::TableColumnWidths)
            .map(|s| s.split(',').filter_map(|v| v.trim().parse::<f64>().ok()).collect())
            .unwrap_or_default();

        if col_widths.len() == max_cols {
            for &width in &col_widths {
                w.write_prop_long(tbl::TBL_GRID_ITEM_TWIPS, pts_to_twips(width) as u32);
            }
        } else {
            let default_col_width = 9360u32 / (max_cols.max(1) as u32);
            for _ in 0..max_cols {
                w.write_prop_long(tbl::TBL_GRID_ITEM_TWIPS, default_col_width);
            }
        }
    });

    // Content — Read1 container
    w.write_item(tbl::CONTENT, |w| {
        for row_id in &rows {
            w.write_item(tbl::ROW, |w| {
                write_row(w, model, *row_id);
            });
        }
    });
}

/// Table properties — ALL use Read1 format (WriteItem), NOT Read2
fn write_table_props(w: &mut DocyWriter, table: &s1_model::Node) {
    // Alignment: WriteItem → WriteByte
    if let Some(AttributeValue::Alignment(a)) = table.attributes.get(&AttributeKey::TableAlignment) {
        let val = match a {
            s1_model::Alignment::Left => align::LEFT,
            s1_model::Alignment::Center => align::CENTER,
            s1_model::Alignment::Right => align::RIGHT,
            _ => align::LEFT,
        };
        w.write_item(tbl_pr::JC, |w| w.write_byte(val));
    }

    // Table width: WriteItem → Read2 content (WriteW format)
    if let Some(AttributeValue::TableWidth(tw)) = table.attributes.get(&AttributeKey::TableWidth) {
        w.write_item(tbl_pr::TABLE_W, |w| {
            write_w_content(w, tw);
        });
    }

    // Table indentation: WriteItem → WriteLong (twips)
    if let Some(v) = table.attributes.get_f64(&AttributeKey::TableIndent) {
        w.write_item(tbl_pr::TABLE_IND_TWIPS, |w| {
            w.write_long(pts_to_twips(v) as u32);
        });
    }

    // Table borders: WriteItem → Read1 border items
    if let Some(brd) = table.attributes.get_borders(&AttributeKey::TableBorders) {
        w.write_item(tbl_pr::TABLE_BORDERS, |w| {
            borders::write_borders(w, brd);
        });
    }

    // Table cell margins: WriteItem → Read1 padding items
    if let Some(m) = table.attributes.get_margins(&AttributeKey::TableDefaultCellMargins) {
        w.write_item(tbl_pr::TABLE_CELL_MAR, |w| {
            // CellMargins uses Read1 with padding items
            w.write_item(padding::LEFT_TWIPS, |w| w.write_long(pts_to_twips(m.left) as u32));
            w.write_item(padding::TOP_TWIPS, |w| w.write_long(pts_to_twips(m.top) as u32));
            w.write_item(padding::RIGHT_TWIPS, |w| w.write_long(pts_to_twips(m.right) as u32));
            w.write_item(padding::BOTTOM_TWIPS, |w| w.write_long(pts_to_twips(m.bottom) as u32));
        });
    }

    // Table layout: WriteItem → WriteByte
    if let Some(layout) = table.attributes.get_table_layout(&AttributeKey::TableLayout) {
        let val = match layout {
            s1_model::TableLayoutMode::AutoFit => 0u8,
            s1_model::TableLayoutMode::Fixed => 1,
            _ => 0,
        };
        w.write_item(tbl_pr::TABLE_LAYOUT, |w| w.write_byte(val));
    }
}

fn write_w_content(w: &mut DocyWriter, tw: &s1_model::TableWidth) {
    // WriteW uses Read2: c_oSerWidthType.Type=0 (Byte) + c_oSerWidthType.WDocx=2 (Long)
    match tw {
        s1_model::TableWidth::Auto => {
            w.write_prop_byte(0, 0); // Type=auto
            w.write_prop_long(2, 0); // WDocx=0
        }
        s1_model::TableWidth::Fixed(v) => {
            w.write_prop_byte(0, 3); // Type=dxa
            w.write_prop_long(2, pts_to_twips(*v) as u32);
        }
        s1_model::TableWidth::Percent(v) => {
            w.write_prop_byte(0, 1); // Type=pct
            w.write_prop_long(2, (*v * 50.0) as u32);
        }
        _ => {
            w.write_prop_byte(0, 0);
            w.write_prop_long(2, 0);
        }
    }
}

fn write_row(w: &mut DocyWriter, model: &DocumentModel, row_id: NodeId) {
    let row = match model.node(row_id) {
        Some(n) => n,
        None => return,
    };

    // Row properties — Read2 format (Read_Row calls bcr.Read2 for Row_Pr)
    w.write_item(tbl::ROW, |w| {
        // TableHeader: Read2 Byte
        if let Some(true) = row.attributes.get_bool(&AttributeKey::TableHeaderRow) {
            w.write_prop_bool(row_pr::TABLE_HEADER, true);
        }
        // Height: Read2 Variable containing Read2 sub-props
        if let Some(h) = row.attributes.get_f64(&AttributeKey::RowHeight) {
            w.write_prop_item(row_pr::HEIGHT, |w| {
                w.write_prop_byte(row_pr::HEIGHT_RULE, 1); // atLeast
                w.write_prop_long(row_pr::HEIGHT_VALUE_TWIPS, pts_to_twips(h) as u32);
            });
        }
    });

    // Row content (cells)
    let cells: Vec<NodeId> = row.children.iter()
        .filter(|id| model.node(**id).map_or(false, |n| n.node_type == NodeType::TableCell))
        .copied()
        .collect();

    w.write_item(tbl::ROW_CONTENT, |w| {
        for cell_id in &cells {
            w.write_item(tbl::CELL, |w| {
                write_cell(w, model, *cell_id);
            });
        }
    });
}

fn write_cell(w: &mut DocyWriter, model: &DocumentModel, cell_id: NodeId) {
    let cell = match model.node(cell_id) {
        Some(n) => n,
        None => return,
    };

    // Cell properties — Read2 format (ReadCell calls bcr.Read2 for Cell_Pr)
    w.write_item(tbl::CELL_PR, |w| {
        // GridSpan: Read2 Long
        if let Some(AttributeValue::Int(span)) = cell.attributes.get(&AttributeKey::ColSpan) {
            if *span > 1 {
                w.write_prop_long(cell_pr::GRID_SPAN, *span as u32);
            }
        }

        // VMerge: Read2 Byte
        if let Some(merge) = cell.attributes.get_string(&AttributeKey::RowSpan) {
            let val = match merge {
                "restart" => 1u8,
                "continue" => 2,
                _ => 0,
            };
            if val > 0 {
                w.write_prop_byte(cell_pr::VMERGE, val);
            }
        }

        // Cell width: Read2 Variable → Read2 W content
        if let Some(AttributeValue::TableWidth(tw)) = cell.attributes.get(&AttributeKey::CellWidth) {
            w.write_prop_item(cell_pr::CELL_W, |w| {
                write_w_content(w, tw);
            });
        }

        // Vertical alignment: Read2 Byte
        if let Some(AttributeValue::VerticalAlignment(va)) = cell.attributes.get(&AttributeKey::VerticalAlign) {
            let val = match va {
                s1_model::VerticalAlignment::Top => 0u8,
                s1_model::VerticalAlignment::Center => 1,
                s1_model::VerticalAlignment::Bottom => 2,
                _ => 0,
            };
            w.write_prop_byte(cell_pr::VALIGN, val);
        }

        // Cell background: Read2 Variable → ReadDocumentShd Read2 content
        if let Some(AttributeValue::Color(c)) = cell.attributes.get(&AttributeKey::CellBackground) {
            w.write_prop_item(cell_pr::SHD, |w| {
                w.write_prop_byte(0, 1); // ShdType.Value=0, ShdClear
                w.write_byte(1); w.write_byte(3); // ShdType.Color=1, lenType=Three
                w.write_color_rgb(c.r, c.g, c.b);
            });
        }

        // Cell borders: Read2 Variable → Read1 border items
        if let Some(brd) = cell.attributes.get_borders(&AttributeKey::CellBorders) {
            w.write_prop_item(cell_pr::BORDERS, |w| {
                borders::write_borders(w, brd);
            });
        }

        // Cell margins: Read2 Variable → Read1 padding items
        if let Some(m) = cell.attributes.get_margins(&AttributeKey::CellPadding) {
            w.write_prop_item(cell_pr::CELL_MAR, |w| {
                w.write_item(padding::LEFT_TWIPS, |w| w.write_long(pts_to_twips(m.left) as u32));
                w.write_item(padding::TOP_TWIPS, |w| w.write_long(pts_to_twips(m.top) as u32));
                w.write_item(padding::RIGHT_TWIPS, |w| w.write_long(pts_to_twips(m.right) as u32));
                w.write_item(padding::BOTTOM_TWIPS, |w| w.write_long(pts_to_twips(m.bottom) as u32));
            });
        }
    });

    // Cell content
    w.write_item(tbl::CELL_CONTENT, |w| {
        for child_id in &cell.children {
            let child = match model.node(*child_id) {
                Some(n) => n,
                None => continue,
            };
            match child.node_type {
                NodeType::Paragraph => {
                    w.write_item(par::PAR, |w| {
                        paragraph::write(w, model, *child_id);
                    });
                }
                NodeType::Table => {
                    w.write_item(par::TABLE, |w| {
                        write(w, model, *child_id);
                    });
                }
                _ => {}
            }
        }
    });
}
