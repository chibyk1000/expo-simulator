use simulator_core::{
    ComponentType, FlexDirection as CoreFlexDirection,
    Node, NodeId, Rect,
};
use std::collections::HashMap;

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn calculate_tree_layout(
        nodes: &mut HashMap<NodeId, Node>,
        root_nodes: &[NodeId],
        viewport_width: f32,
        viewport_height: f32,
    ) {
        let mut root_y = 0.0f32;
        for &root_id in root_nodes {
            let h = Self::layout_node(root_id, nodes, 0.0, root_y, viewport_width, viewport_height);
            root_y += h;
        }
    }

    fn layout_node(
        node_id: NodeId,
        nodes: &mut HashMap<NodeId, Node>,
        rel_x: f32,
        rel_y: f32,
        available_width: f32,
        available_height: f32,
    ) -> f32 {
        let (component_type, style, children, text_content, placeholder) = {
            let n = match nodes.get(&node_id) {
                Some(n) => n,
                None => return 0.0,
            };
            (
                n.component_type.clone(),
                n.style.clone(),
                n.children.clone(),
                n.text_content.clone(),
                n.placeholder.clone(),
            )
        };

        // Determine node dimensions
        let width = style.width.unwrap_or(available_width);
        let mut height = style.height.unwrap_or(0.0);

        // Intrinsic sizing for Text and TextInput
        if (component_type == ComponentType::Text || component_type == ComponentType::TextInput)
            && height == 0.0
        {
            let text_to_measure = text_content.as_deref().or(placeholder.as_deref()).unwrap_or("");
            let font_size = style.font_size;
            let lh = style.line_height.unwrap_or(font_size * 1.3);
            let approx_line_len = (width / (font_size * 0.55)).max(1.0);
            let line_count = (text_to_measure.len() as f32 / approx_line_len).ceil().max(1.0);
            let text_h = line_count * lh;
            height = text_h + style.padding.top + style.padding.bottom;
        }

        // Layout children inside content box
        let content_width = (width - style.padding.left - style.padding.right).max(0.0);
        let is_row = style.flex_direction == CoreFlexDirection::Row
            || style.flex_direction == CoreFlexDirection::RowReverse;

        let mut child_cursor_x = style.padding.left;
        let mut child_cursor_y = style.padding.top;
        let mut max_row_height = 0.0f32;

        for &child_id in &children {
            let child_style = nodes.get(&child_id).map(|c| c.style.clone()).unwrap_or_default();
            let c_x = child_cursor_x + child_style.margin.left;
            let c_y = child_cursor_y + child_style.margin.top;

            let child_h = Self::layout_node(
                child_id,
                nodes,
                c_x,
                c_y,
                content_width - child_style.margin.left - child_style.margin.right,
                available_height,
            );

            let child_w = nodes.get(&child_id).map(|c| c.layout.width).unwrap_or(content_width);

            if is_row {
                child_cursor_x += child_w + child_style.margin.left + child_style.margin.right;
                max_row_height = max_row_height.max(child_h + child_style.margin.top + child_style.margin.bottom);
            } else {
                child_cursor_y += child_h + child_style.margin.top + child_style.margin.bottom;
            }
        }

        if height == 0.0 {
            if is_row {
                height = max_row_height + style.padding.top + style.padding.bottom;
            } else {
                height = child_cursor_y + style.padding.bottom;
            }
        }

        // ScrollView takes full viewport height if flex: 1 or unset
        if component_type == ComponentType::ScrollView {
            height = height.max(available_height);
        }

        let total_h = height;
        if let Some(n) = nodes.get_mut(&node_id) {
            n.layout = Rect::new(rel_x + style.margin.left, rel_y + style.margin.top, width, height);
        }

        total_h
    }
}
