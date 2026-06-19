use std::rc::Rc;

use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    overlay::{Menu, MenuItem, overlay},
    theme::{ActiveTheme, radius, space},
};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type SelectHandler = Box<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;
type ActionHandler = Box<dyn Fn(BranchActionKind, &mut Window, &mut App) + 'static>;

/// A selectable Git branch row in [`BranchSelector`].
pub struct BranchOption {
    key: &'static str,
    label: String,
    detail: Option<String>,
}

impl BranchOption {
    pub fn new(key: &'static str, label: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
            detail: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// A non-destructive helper action shown at the bottom of a branch picker.
pub struct BranchPickerAction {
    key: &'static str,
    label: String,
    icon: IconName,
}

impl BranchPickerAction {
    pub fn new(key: &'static str, label: impl Into<String>, icon: IconName) -> Self {
        Self {
            key,
            label: label.into(),
            icon,
        }
    }
}

/// Compact branch selector for title bars and pane toolbars.
#[derive(IntoElement)]
pub struct BranchSelector {
    id: ElementId,
    selected_key: &'static str,
    branches: Vec<BranchOption>,
    actions: Vec<BranchPickerAction>,
    open: bool,
    on_toggle: Option<ClickHandler>,
    on_select: Option<SelectHandler>,
    on_action: Option<SelectHandler>,
}

impl BranchSelector {
    pub fn new(
        id: impl Into<ElementId>,
        selected_key: &'static str,
        branches: Vec<BranchOption>,
    ) -> Self {
        Self {
            id: id.into(),
            selected_key,
            branches,
            actions: default_picker_actions(),
            open: false,
            on_toggle: None,
            on_select: None,
            on_action: None,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn actions(mut self, actions: Vec<BranchPickerAction>) -> Self {
        self.actions = actions;
        self
    }

    pub fn on_toggle(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }

    pub fn on_action(
        mut self,
        handler: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_action = Some(Box::new(handler));
        self
    }

    pub fn selected_label(&self) -> &str {
        self.branches
            .iter()
            .find(|branch| branch.key == self.selected_key)
            .map_or(self.selected_key, |branch| branch.label.as_str())
    }
}

impl RenderOnce for BranchSelector {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let selected_label = self.selected_label().to_string();
        let selected_key = self.selected_key;
        let select_handler = self.on_select.map(Rc::new);
        let action_handler = self.on_action.map(Rc::new);
        let trigger_handler = self.on_toggle;
        let mut root = div().id(self.id).relative().flex().items_center().child(
            div()
                .id("branch-selector-trigger")
                .h(px(28.0))
                .max_w(px(260.0))
                .px_2()
                .flex()
                .items_center()
                .gap_1()
                .rounded(px(radius::MD))
                .border_1()
                .border_color(if self.open {
                    theme.border_strong
                } else {
                    theme.border
                })
                .bg(if self.open {
                    theme.panel_alt
                } else {
                    theme.panel
                })
                .text_color(theme.text_secondary)
                .cursor_pointer()
                .hover(move |style| style.bg(theme.hover).border_color(theme.border_strong))
                .child(
                    Icon::new(IconName::GitBranch)
                        .size(IconSize::Small)
                        .color(theme.text_muted),
                )
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .child(selected_label),
                )
                .child(
                    Icon::new(IconName::ChevronDown)
                        .size(IconSize::XSmall)
                        .color(theme.text_muted),
                )
                .when_some(trigger_handler, |this, handler| {
                    this.on_click(move |event, window, cx| {
                        handler(event, window, cx);
                        cx.stop_propagation();
                    })
                }),
        );

        if self.open {
            root = root.child(branch_picker_panel(
                selected_key,
                self.branches,
                self.actions,
                select_handler,
                action_handler,
            ));
        }

        root
    }
}

fn branch_picker_panel(
    selected_key: &'static str,
    branches: Vec<BranchOption>,
    actions: Vec<BranchPickerAction>,
    select_handler: Option<Rc<SelectHandler>>,
    action_handler: Option<Rc<SelectHandler>>,
) -> impl IntoElement {
    overlay(
        BranchPickerPanel {
            selected_key,
            branches,
            actions,
            select_handler,
            action_handler,
        }
        .into_any_element(),
    )
    .offset(0.0, 34.0)
}

#[derive(IntoElement)]
struct BranchPickerPanel {
    selected_key: &'static str,
    branches: Vec<BranchOption>,
    actions: Vec<BranchPickerAction>,
    select_handler: Option<Rc<SelectHandler>>,
    action_handler: Option<Rc<SelectHandler>>,
}

impl RenderOnce for BranchPickerPanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let mut panel = div()
            .id("branch-picker-panel")
            .w(px(320.0))
            .p(px(space::XS))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .rounded(px(radius::LG))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border_strong)
            .shadow_lg()
            .occlude()
            .child(
                div()
                    .h(px(26.0))
                    .px_2()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text_muted)
                            .child("SWITCH BRANCH"),
                    )
                    .child(
                        Icon::new(IconName::GitBranch)
                            .size(IconSize::Small)
                            .color(theme.text_muted),
                    ),
            );

        for (index, branch) in self.branches.into_iter().enumerate() {
            let handler = self.select_handler.clone();
            let selected = branch.key == self.selected_key;
            let key = branch.key;
            let row_fg = if selected {
                theme.text
            } else {
                theme.text_secondary
            };

            panel = panel.child(
                div()
                    .id(("branch-option", index))
                    .min_h(px(34.0))
                    .px_2()
                    .py_1()
                    .flex()
                    .items_center()
                    .gap_2()
                    .rounded(px(radius::MD))
                    .text_color(row_fg)
                    .cursor_pointer()
                    .when(selected, |this| this.bg(theme.selection))
                    .when(!selected, |this| {
                        this.hover(move |style| style.bg(theme.hover))
                    })
                    .child(
                        div()
                            .w(px(16.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(if selected {
                                Icon::new(IconName::Check)
                                    .size(IconSize::Small)
                                    .color(theme.accent)
                                    .into_any_element()
                            } else {
                                Icon::new(IconName::GitBranch)
                                    .size(IconSize::Small)
                                    .color(theme.text_muted)
                                    .into_any_element()
                            }),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(1.0))
                            .child(
                                div()
                                    .truncate()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(branch.label),
                            )
                            .when_some(branch.detail, |this, detail| {
                                this.child(
                                    div()
                                        .truncate()
                                        .text_size(px(11.0))
                                        .text_color(theme.text_muted)
                                        .child(detail),
                                )
                            }),
                    )
                    .when_some(handler, |this, handler| {
                        this.on_click(move |_event: &ClickEvent, window, cx| {
                            handler(key, window, cx);
                            cx.stop_propagation();
                        })
                    }),
            );
        }

        if !self.actions.is_empty() {
            panel = panel.child(div().my(px(space::XS)).h(px(1.0)).w_full().bg(theme.border));
        }

        for (index, action) in self.actions.into_iter().enumerate() {
            let handler = self.action_handler.clone();
            let key = action.key;
            panel = panel.child(
                div()
                    .id(("branch-picker-action", index))
                    .h(px(30.0))
                    .px_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .rounded(px(radius::MD))
                    .text_sm()
                    .text_color(theme.text_secondary)
                    .cursor_pointer()
                    .hover(move |style| style.bg(theme.hover).text_color(theme.text))
                    .child(
                        Icon::new(action.icon)
                            .size(IconSize::Small)
                            .color(theme.text_muted),
                    )
                    .child(div().flex_1().min_w_0().truncate().child(action.label))
                    .when_some(handler, |this, handler| {
                        this.on_click(move |_event: &ClickEvent, window, cx| {
                            handler(key, window, cx);
                            cx.stop_propagation();
                        })
                    }),
            );
        }

        panel
    }
}

/// Branch operations shown from a separate "more" menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchActionKind {
    Checkout,
    NewWorktree,
    Rename,
    Delete,
}

impl BranchActionKind {
    pub fn label(self) -> &'static str {
        match self {
            BranchActionKind::Checkout => "Checkout branch",
            BranchActionKind::NewWorktree => "New worktree from branch",
            BranchActionKind::Rename => "Rename branch",
            BranchActionKind::Delete => "Delete branch",
        }
    }

    pub fn icon(self) -> IconName {
        match self {
            BranchActionKind::Checkout => IconName::GitBranch,
            BranchActionKind::NewWorktree => IconName::FolderPlus,
            BranchActionKind::Rename => IconName::Settings,
            BranchActionKind::Delete => IconName::Archive,
        }
    }

    pub fn is_dangerous(self) -> bool {
        matches!(self, BranchActionKind::Delete)
    }
}

/// Context menu for branch management actions.
#[derive(IntoElement)]
pub struct BranchActionsMenu {
    id: ElementId,
    actions: Vec<BranchActionKind>,
    on_select: Option<ActionHandler>,
}

impl BranchActionsMenu {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            actions: vec![
                BranchActionKind::Checkout,
                BranchActionKind::NewWorktree,
                BranchActionKind::Rename,
                BranchActionKind::Delete,
            ],
            on_select: None,
        }
    }

    pub fn actions(mut self, actions: Vec<BranchActionKind>) -> Self {
        self.actions = actions;
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(BranchActionKind, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for BranchActionsMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let handler = self.on_select.map(Rc::new);
        let mut items = Vec::with_capacity(self.actions.len() + 1);

        for action in self.actions {
            if action == BranchActionKind::Delete && !items.is_empty() {
                items.push(MenuItem::separator());
            }

            let action_handler = handler.clone();
            let mut item = MenuItem::new(action.label()).icon(action.icon());
            if action.is_dangerous() {
                item = item.danger();
            }
            if let Some(action_handler) = action_handler {
                item = item.on_click(move |_event, window, cx| {
                    action_handler(action, window, cx);
                });
            }
            items.push(item);
        }

        Menu::new(self.id, items)
            .min_width(232.0)
            .render(window, cx)
    }
}

fn default_picker_actions() -> Vec<BranchPickerAction> {
    vec![
        BranchPickerAction::new("branch:create", "Create branch", IconName::Plus),
        BranchPickerAction::new("worktree:create", "New worktree", IconName::FolderPlus),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_selector_uses_branch_label_for_selected_key() {
        let selector = BranchSelector::new(
            "branch-selector",
            "feat-ui",
            vec![BranchOption::new("feat-ui", "feature/ui-kit")],
        );

        assert_eq!(selector.selected_label(), "feature/ui-kit");
    }

    #[test]
    fn branch_selector_falls_back_to_selected_key() {
        let selector = BranchSelector::new("branch-selector", "main", vec![]);

        assert_eq!(selector.selected_label(), "main");
    }

    #[test]
    fn delete_branch_action_is_dangerous() {
        assert!(BranchActionKind::Delete.is_dangerous());
    }
}
