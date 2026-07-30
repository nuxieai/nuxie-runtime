use proc_macro2::{LineColumn, Span, TokenStream, TokenTree};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::{self, Read};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Block, ExprMethodCall, ExprPath, File, ImplItemFn, Item, ItemFn, ItemMod, Macro,
    PatStruct, PatTupleStruct, Path, Type, UseTree,
};

const ALLOW_TAG: &str = "flc5-owner-ratchet-allow:";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum GuardKind {
    Collection,
    Selection,
    Dispatch,
    Audio,
}

impl GuardKind {
    const ALL: [Self; 4] = [
        Self::Collection,
        Self::Selection,
        Self::Dispatch,
        Self::Audio,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Collection => "collection",
            Self::Selection => "selection",
            Self::Dispatch => "dispatch",
            Self::Audio => "audio",
        }
    }

    fn matches_member(self, name: &str) -> bool {
        match self {
            Self::Collection => {
                name == "reported_event_count"
                    || name == "reported_event"
                    || (name.starts_with("take_")
                        && (name.contains("event") || name.contains("report")))
            }
            Self::Selection => name == "StateMachine",
            Self::Dispatch => name == "notify_events" || name.starts_with("notify_events_"),
            Self::Audio => matches!(
                name,
                "flush_deferred_owner_audio_event"
                    | "flush_deferred_owner_audio_events"
                    | "defer_recorded_audio_event_seam"
                    | "reach_recorded_audio_event_seam"
                    | "deliver_recorded_audio_occurrence"
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TypeTarget {
    StateMachineInstance,
    NestedAnimation,
}

#[derive(Clone, Default)]
struct Bindings {
    types: HashMap<String, TypeTarget>,
    variants: HashSet<String>,
    guarded_values: HashMap<String, GuardKind>,
    nested_animation_glob: bool,
}

#[derive(Clone)]
struct UseEntry {
    path: Vec<String>,
    local: Option<String>,
    glob: bool,
}

fn ident_name(ident: &syn::Ident) -> String {
    ident.to_string().trim_start_matches("r#").to_owned()
}

fn path_names(path: &Path) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| ident_name(&segment.ident))
        .collect()
}

fn flatten_use(tree: &UseTree, prefix: &mut Vec<String>, entries: &mut Vec<UseEntry>) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(ident_name(&path.ident));
            flatten_use(&path.tree, prefix, entries);
            prefix.pop();
        }
        UseTree::Name(name) => {
            let imported = ident_name(&name.ident);
            let mut path = prefix.clone();
            path.push(imported.clone());
            entries.push(UseEntry {
                path,
                local: Some(imported),
                glob: false,
            });
        }
        UseTree::Rename(rename) => {
            let mut path = prefix.clone();
            path.push(ident_name(&rename.ident));
            entries.push(UseEntry {
                path,
                local: Some(ident_name(&rename.rename)),
                glob: false,
            });
        }
        UseTree::Glob(_) => entries.push(UseEntry {
            path: prefix.clone(),
            local: None,
            glob: true,
        }),
        UseTree::Group(group) => {
            for item in &group.items {
                flatten_use(item, prefix, entries);
            }
        }
    }
}

fn cfg_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attribute| {
        if !attribute.path().is_ident("cfg") {
            return false;
        }
        let mut has_test = false;
        let _ = attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("test") {
                has_test = true;
            }
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("test") {
                        has_test = true;
                    }
                    Ok(())
                })?;
            }
            Ok(())
        });
        has_test
    })
}

fn path_target(path: &[String], scopes: &[Bindings]) -> Option<TypeTarget> {
    let last = path.last()?;
    match last.as_str() {
        "StateMachineInstance" => return Some(TypeTarget::StateMachineInstance),
        "RuntimeNestedAnimationInstance" => return Some(TypeTarget::NestedAnimation),
        _ => {}
    }
    if path.len() == 1 {
        for scope in scopes.iter().rev() {
            if let Some(target) = scope.types.get(last) {
                return Some(*target);
            }
        }
    }
    None
}

fn bindings_for_items(items: &[Item], scopes: &[Bindings], audio_aliases: &[String]) -> Bindings {
    let mut bindings = Bindings::default();
    for alias in audio_aliases {
        bindings
            .guarded_values
            .insert(alias.clone(), GuardKind::Audio);
    }

    let mut uses = Vec::new();
    let mut type_aliases = Vec::new();
    for item in items {
        if cfg_test(item_attrs(item)) {
            continue;
        }
        match item {
            Item::Use(item_use) => {
                flatten_use(&item_use.tree, &mut Vec::new(), &mut uses);
            }
            Item::Type(item_type) => {
                if let Type::Path(path) = item_type.ty.as_ref() {
                    type_aliases.push((ident_name(&item_type.ident), path_names(&path.path)));
                }
            }
            _ => {}
        }
    }

    for _ in 0..=(type_aliases.len() + uses.len()) {
        let mut changed = false;
        for (alias, path) in &type_aliases {
            if bindings.types.contains_key(alias) {
                continue;
            }
            let mut visible = scopes.to_vec();
            visible.push(bindings.clone());
            if let Some(target) = path_target(path, &visible) {
                bindings.types.insert(alias.clone(), target);
                changed = true;
            }
        }
        for entry in &uses {
            let mut visible = scopes.to_vec();
            visible.push(bindings.clone());
            if entry.glob {
                if !bindings.nested_animation_glob
                    && path_target(&entry.path, &visible) == Some(TypeTarget::NestedAnimation)
                {
                    bindings.nested_animation_glob = true;
                    changed = true;
                }
                continue;
            }
            let Some(local) = entry.local.as_ref() else {
                continue;
            };
            if let Some(target) = path_target(&entry.path, &visible) {
                if bindings.types.insert(local.clone(), target) != Some(target) {
                    changed = true;
                }
                continue;
            }
            let Some(member) = entry.path.last() else {
                continue;
            };
            let prefix = &entry.path[..entry.path.len().saturating_sub(1)];
            if member == "StateMachine"
                && path_target(prefix, &visible) == Some(TypeTarget::NestedAnimation)
            {
                changed |= bindings.variants.insert(local.clone());
                continue;
            }
            for kind in [GuardKind::Collection, GuardKind::Dispatch, GuardKind::Audio] {
                if kind.matches_member(member) {
                    if bindings.guarded_values.insert(local.clone(), kind) != Some(kind) {
                        changed = true;
                    }
                    break;
                }
            }
        }
        if !changed {
            break;
        }
    }
    bindings
}

fn item_attrs(item: &Item) -> &[Attribute] {
    match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::ExternCrate(item) => &item.attrs,
        Item::Fn(item) => &item.attrs,
        Item::ForeignMod(item) => &item.attrs,
        Item::Impl(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::TraitAlias(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        Item::Verbatim(_) => &[],
        _ => &[],
    }
}

struct MechanicVisitor {
    found: bool,
}

impl<'ast> Visit<'ast> for MechanicVisitor {
    fn visit_ident(&mut self, ident: &'ast syn::Ident) {
        let name = ident_name(ident);
        if name == "reported_events"
            || name == "StateMachineReportedEvent"
            || GuardKind::Collection.matches_member(&name)
            || GuardKind::Dispatch.matches_member(&name)
            || GuardKind::Audio.matches_member(&name)
        {
            self.found = true;
        }
    }
}

fn block_has_event_mechanic(block: &Block) -> bool {
    let mut visitor = MechanicVisitor { found: false };
    visitor.visit_block(block);
    visitor.found
}

struct Analyzer<'a> {
    source: &'a str,
    line_offsets: Vec<usize>,
    scopes: Vec<Bindings>,
    audio_aliases: Vec<String>,
    hits: HashMap<GuardKind, BTreeSet<usize>>,
    function_start: Option<usize>,
    direct_selection_is_guarded: bool,
}

impl<'a> Analyzer<'a> {
    fn new(source: &'a str, audio_aliases: Vec<String>) -> Self {
        let mut line_offsets = vec![0];
        line_offsets.extend(
            source
                .bytes()
                .enumerate()
                .filter_map(|(index, byte)| (byte == b'\n').then_some(index + 1)),
        );
        Self {
            source,
            line_offsets,
            scopes: Vec::new(),
            audio_aliases,
            hits: HashMap::new(),
            function_start: None,
            direct_selection_is_guarded: false,
        }
    }

    fn offset(&self, location: LineColumn) -> usize {
        self.line_offsets
            .get(location.line.saturating_sub(1))
            .copied()
            .unwrap_or(0)
            .saturating_add(location.column)
            .min(self.source.len())
    }

    fn is_allowlisted(&self, kind: GuardKind, span: Span) -> bool {
        let offset = self.offset(span.start());
        let line_start = self.source[..offset]
            .rfind('\n')
            .map_or(0, |index| index.saturating_add(1));
        let line_end = self.source[offset..]
            .find('\n')
            .map_or(self.source.len(), |index| offset.saturating_add(index));
        let line = &self.source[line_start..line_end];
        line.contains(&format!("{ALLOW_TAG} {}", kind.name()))
            || line.contains(&format!("{ALLOW_TAG} all"))
    }

    fn record(&mut self, kind: GuardKind, span: Span) {
        if self.is_allowlisted(kind, span) {
            return;
        }
        let offset = self
            .function_start
            .unwrap_or_else(|| self.offset(span.start()));
        self.hits.entry(kind).or_default().insert(offset);
    }

    fn resolve_type(&self, names: &[String]) -> Option<TypeTarget> {
        path_target(names, &self.scopes)
    }

    fn variant_is_bound(&self, name: &str) -> bool {
        self.scopes
            .iter()
            .rev()
            .any(|scope| scope.variants.contains(name) || scope.nested_animation_glob)
    }

    fn guarded_value(&self, name: &str) -> Option<GuardKind> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.guarded_values.get(name).copied())
    }

    fn analyze_path(&mut self, path: &Path, qself: Option<&syn::QSelf>) {
        let names = path_names(path);
        let Some(member) = names.last() else {
            return;
        };
        let span = path.span();

        for kind in [GuardKind::Collection, GuardKind::Dispatch, GuardKind::Audio] {
            if self.guarded_value(member) == Some(kind) {
                self.record(kind, span);
            } else if qself.is_some() && kind.matches_member(member) {
                self.record(kind, span);
            } else if names.len() > 1 && kind.matches_member(member) {
                // Qualified paths, including spaced tokens and UFCS, are
                // deliberately fail-closed on the guarded ownership names.
                self.record(kind, span);
            }
        }

        if names.len() == 1 {
            if self.variant_is_bound(member)
                || (member == "StateMachine" && self.direct_selection_is_guarded)
            {
                self.record(GuardKind::Selection, span);
            }
            return;
        }
        if member != "StateMachine" {
            return;
        }
        let prefix = &names[..names.len() - 1];
        let resolved = self.resolve_type(prefix) == Some(TypeTarget::NestedAnimation);
        if !resolved && !self.direct_selection_is_guarded {
            return;
        }
        let explicitly_canonical = prefix
            .iter()
            .any(|segment| segment == "RuntimeNestedAnimationInstance" || segment == "Self");
        if !resolved || !explicitly_canonical || self.direct_selection_is_guarded {
            self.record(GuardKind::Selection, span);
        }
    }

    fn macro_contains_guard(tokens: TokenStream, kind: GuardKind) -> bool {
        tokens.into_iter().any(|token| match token {
            TokenTree::Ident(ident) => kind.matches_member(&ident_name(&ident)),
            TokenTree::Group(group) => Self::macro_contains_guard(group.stream(), kind),
            _ => false,
        })
    }

    fn push_item_scope(&mut self, items: &[Item]) {
        let bindings = bindings_for_items(items, &self.scopes, &self.audio_aliases);
        self.scopes.push(bindings);
    }

    fn push_block_scope(&mut self, block: &Block) {
        let items = block
            .stmts
            .iter()
            .filter_map(|statement| match statement {
                syn::Stmt::Item(item) => Some(item.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        self.push_item_scope(&items);
    }
}

impl<'ast> Visit<'ast> for Analyzer<'_> {
    fn visit_file(&mut self, file: &'ast File) {
        self.push_item_scope(&file.items);
        visit::visit_file(self, file);
        self.scopes.pop();
    }

    fn visit_item(&mut self, item: &'ast Item) {
        if !cfg_test(item_attrs(item)) {
            visit::visit_item(self, item);
        }
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        if cfg_test(&item.attrs) {
            return;
        }
        if let Some((_, items)) = &item.content {
            self.push_item_scope(items);
            for child in items {
                self.visit_item(child);
            }
            self.scopes.pop();
        }
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        if cfg_test(&item.attrs) {
            return;
        }
        let previous_start = self
            .function_start
            .replace(self.offset(item.span().start()));
        let previous_selection = std::mem::replace(
            &mut self.direct_selection_is_guarded,
            block_has_event_mechanic(&item.block),
        );
        visit::visit_item_fn(self, item);
        self.direct_selection_is_guarded = previous_selection;
        self.function_start = previous_start;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        if cfg_test(&item.attrs) {
            return;
        }
        let previous_start = self
            .function_start
            .replace(self.offset(item.span().start()));
        let previous_selection = std::mem::replace(
            &mut self.direct_selection_is_guarded,
            block_has_event_mechanic(&item.block),
        );
        visit::visit_impl_item_fn(self, item);
        self.direct_selection_is_guarded = previous_selection;
        self.function_start = previous_start;
    }

    fn visit_block(&mut self, block: &'ast Block) {
        self.push_block_scope(block);
        visit::visit_block(self, block);
        self.scopes.pop();
    }

    fn visit_expr_path(&mut self, expression: &'ast ExprPath) {
        self.analyze_path(&expression.path, expression.qself.as_ref());
        visit::visit_expr_path(self, expression);
    }

    fn visit_pat_tuple_struct(&mut self, pattern: &'ast PatTupleStruct) {
        self.analyze_path(&pattern.path, None);
        visit::visit_pat_tuple_struct(self, pattern);
    }

    fn visit_pat_struct(&mut self, pattern: &'ast PatStruct) {
        self.analyze_path(&pattern.path, None);
        visit::visit_pat_struct(self, pattern);
    }

    fn visit_expr_method_call(&mut self, expression: &'ast ExprMethodCall) {
        let member = ident_name(&expression.method);
        for kind in [GuardKind::Collection, GuardKind::Dispatch, GuardKind::Audio] {
            if kind.matches_member(&member) {
                self.record(kind, expression.method.span());
            }
        }
        visit::visit_expr_method_call(self, expression);
    }

    fn visit_macro(&mut self, item: &'ast Macro) {
        for kind in GuardKind::ALL {
            let path_guarded = item
                .path
                .segments
                .last()
                .is_some_and(|segment| kind.matches_member(&ident_name(&segment.ident)));
            if path_guarded || Self::macro_contains_guard(item.tokens.clone(), kind) {
                self.record(kind, item.span());
            }
        }
        visit::visit_macro(self, item);
    }
}

fn audio_exports(file: &File) -> BTreeSet<String> {
    fn collect(items: &[Item], aliases: &mut BTreeSet<String>) {
        for item in items {
            if cfg_test(item_attrs(item)) {
                continue;
            }
            match item {
                Item::Const(item) => {
                    if let syn::Expr::Path(path) = item.expr.as_ref()
                        && path.path.segments.last().is_some_and(|segment| {
                            GuardKind::Audio.matches_member(&ident_name(&segment.ident))
                        })
                    {
                        aliases.insert(ident_name(&item.ident));
                    }
                }
                Item::Static(item) => {
                    if let syn::Expr::Path(path) = item.expr.as_ref()
                        && path.path.segments.last().is_some_and(|segment| {
                            GuardKind::Audio.matches_member(&ident_name(&segment.ident))
                        })
                    {
                        aliases.insert(ident_name(&item.ident));
                    }
                }
                Item::Mod(item) => {
                    if let Some((_, children)) = &item.content {
                        collect(children, aliases);
                    }
                }
                _ => {}
            }
        }
    }

    let mut aliases = BTreeSet::new();
    collect(&file.items, &mut aliases);
    aliases
}

fn lexical_tripwire(source: &str) -> HashMap<GuardKind, BTreeSet<usize>> {
    let mut hits = HashMap::new();
    let normalized = source.replace("r#", "");
    for kind in GuardKind::ALL {
        if normalized
            .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
            .any(|token| kind.matches_member(token))
        {
            hits.entry(kind).or_insert_with(BTreeSet::new).insert(0);
        }
    }
    hits
}

fn main() {
    let mut source = String::new();
    if let Err(error) = io::stdin().read_to_string(&mut source) {
        eprintln!("cannot read Rust source: {error}");
        std::process::exit(2);
    }
    let audio_aliases = std::env::args().skip(1).collect::<Vec<_>>();
    let Ok(file) = syn::parse_file(&source) else {
        for (kind, offsets) in lexical_tripwire(&source) {
            for offset in offsets {
                println!("hit {} {offset}", kind.name());
            }
        }
        return;
    };

    for alias in audio_exports(&file) {
        println!("alias {alias}");
    }
    let mut analyzer = Analyzer::new(&source, audio_aliases);
    analyzer.visit_file(&file);
    for kind in GuardKind::ALL {
        if let Some(offsets) = analyzer.hits.get(&kind) {
            for offset in offsets {
                println!("hit {} {offset}", kind.name());
            }
        }
    }
}
