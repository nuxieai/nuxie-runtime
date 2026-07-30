use proc_macro2::{LineColumn, Span, TokenStream, TokenTree};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::{self, Read};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Block, ExprMethodCall, ExprPath, File, ImplItem, ImplItemFn, Item, ItemFn, ItemMod,
    Macro, Meta, PatStruct, PatTupleStruct, Path, Token, Type, UseTree,
};

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

    fn matches_guarded_type(self, name: &str) -> bool {
        match self {
            Self::Selection => name == "RuntimeNestedAnimationInstance",
            Self::Collection | Self::Dispatch | Self::Audio => name == "StateMachineInstance",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TypeTarget {
    StateMachineInstance,
    NestedAnimation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TypeResolution {
    Target(TypeTarget),
    Unresolved,
}

#[derive(Clone, Default)]
struct Bindings {
    types: HashMap<String, TypeTarget>,
    unresolved_types: HashSet<String>,
    modules: HashMap<String, Box<Bindings>>,
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

fn meta_guarantees_test(meta: &Meta) -> bool {
    match meta {
        Meta::Path(path) => path.is_ident("test"),
        Meta::List(list) if list.path.is_ident("all") => {
            let Ok(arguments) =
                Punctuated::<Meta, Token![,]>::parse_terminated.parse2(list.tokens.clone())
            else {
                return false;
            };
            arguments.iter().any(meta_guarantees_test)
        }
        Meta::List(_) | Meta::NameValue(_) => false,
    }
}

fn cfg_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attribute| {
        if !attribute.path().is_ident("cfg") {
            return false;
        }
        attribute
            .parse_args::<Meta>()
            .is_ok_and(|meta| meta_guarantees_test(&meta))
    })
}

fn resolution_in_bindings(path: &[String], bindings: &Bindings) -> Option<TypeResolution> {
    let path = path.strip_prefix(&["self".to_owned()]).unwrap_or(path);
    let (first, rest) = path.split_first()?;
    if rest.is_empty() {
        if let Some(target) = bindings.types.get(first) {
            return Some(TypeResolution::Target(*target));
        }
        if bindings.unresolved_types.contains(first) {
            return Some(TypeResolution::Unresolved);
        }
        return None;
    }
    bindings
        .modules
        .get(first)
        .and_then(|module| resolution_in_bindings(rest, module))
}

fn path_target(path: &[String], scopes: &[Bindings]) -> Option<TypeResolution> {
    let last = path.last()?;
    match last.as_str() {
        "StateMachineInstance" => {
            return Some(TypeResolution::Target(TypeTarget::StateMachineInstance));
        }
        "RuntimeNestedAnimationInstance" => {
            return Some(TypeResolution::Target(TypeTarget::NestedAnimation));
        }
        _ => {}
    }
    for scope in scopes.iter().rev() {
        if let Some(resolution) = resolution_in_bindings(path, scope) {
            return Some(resolution);
        }
    }
    None
}

#[derive(Clone)]
enum TypeAliasTarget {
    Path(Vec<String>),
    Associated {
        self_type: String,
        trait_name: String,
        member: String,
    },
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
    let mut associated_types = HashMap::new();
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
                    let target = if let Some(qself) = &path.qself {
                        let self_type = match qself.ty.as_ref() {
                            Type::Path(path) => path_names(&path.path).last().cloned(),
                            _ => None,
                        };
                        let names = path_names(&path.path);
                        match (self_type, names.get(qself.position), names.last()) {
                            (Some(self_type), Some(member), Some(last))
                                if member == last && qself.position > 0 =>
                            {
                                TypeAliasTarget::Associated {
                                    self_type,
                                    trait_name: names[qself.position - 1].clone(),
                                    member: member.clone(),
                                }
                            }
                            _ => {
                                bindings
                                    .unresolved_types
                                    .insert(ident_name(&item_type.ident));
                                continue;
                            }
                        }
                    } else {
                        TypeAliasTarget::Path(path_names(&path.path))
                    };
                    type_aliases.push((ident_name(&item_type.ident), target));
                }
            }
            Item::Impl(item_impl) => {
                let Some((_, trait_path, _)) = &item_impl.trait_ else {
                    continue;
                };
                let Type::Path(self_type) = item_impl.self_ty.as_ref() else {
                    continue;
                };
                let Some(self_name) = path_names(&self_type.path).last().cloned() else {
                    continue;
                };
                let Some(trait_name) = path_names(trait_path).last().cloned() else {
                    continue;
                };
                for impl_item in &item_impl.items {
                    let ImplItem::Type(associated) = impl_item else {
                        continue;
                    };
                    let Type::Path(target) = &associated.ty else {
                        continue;
                    };
                    associated_types.insert(
                        (
                            self_name.clone(),
                            trait_name.clone(),
                            ident_name(&associated.ident),
                        ),
                        path_names(&target.path),
                    );
                }
            }
            Item::Mod(item_mod) => {
                if let Some((_, children)) = &item_mod.content {
                    let nested = bindings_for_items(children, scopes, audio_aliases);
                    bindings
                        .modules
                        .insert(ident_name(&item_mod.ident), Box::new(nested));
                }
            }
            _ => {}
        }
    }

    for _ in 0..=(type_aliases.len() + uses.len()) {
        let mut changed = false;
        for (alias, target) in &type_aliases {
            if bindings.types.contains_key(alias) || bindings.unresolved_types.contains(alias) {
                continue;
            }
            let mut visible = scopes.to_vec();
            visible.push(bindings.clone());
            let resolution = match target {
                TypeAliasTarget::Path(path) => path_target(path, &visible),
                TypeAliasTarget::Associated {
                    self_type,
                    trait_name,
                    member,
                } => associated_types
                    .get(&(self_type.clone(), trait_name.clone(), member.clone()))
                    .and_then(|path| path_target(path, &visible))
                    .or(Some(TypeResolution::Unresolved)),
            };
            match resolution {
                Some(TypeResolution::Target(target)) => {
                    bindings.types.insert(alias.clone(), target);
                    changed = true;
                }
                Some(TypeResolution::Unresolved) => {
                    bindings.unresolved_types.insert(alias.clone());
                    changed = true;
                }
                None => {}
            }
        }
        for entry in &uses {
            let mut visible = scopes.to_vec();
            visible.push(bindings.clone());
            if entry.glob {
                if !bindings.nested_animation_glob
                    && path_target(&entry.path, &visible)
                        == Some(TypeResolution::Target(TypeTarget::NestedAnimation))
                {
                    bindings.nested_animation_glob = true;
                    changed = true;
                }
                continue;
            }
            let Some(local) = entry.local.as_ref() else {
                continue;
            };
            match path_target(&entry.path, &visible) {
                Some(TypeResolution::Target(target)) => {
                    if bindings.types.insert(local.clone(), target) != Some(target) {
                        changed = true;
                    }
                    continue;
                }
                Some(TypeResolution::Unresolved) => {
                    changed |= bindings.unresolved_types.insert(local.clone());
                    continue;
                }
                None => {}
            }
            let Some(member) = entry.path.last() else {
                continue;
            };
            let prefix = &entry.path[..entry.path.len().saturating_sub(1)];
            if member == "StateMachine"
                && matches!(
                    path_target(prefix, &visible),
                    Some(
                        TypeResolution::Target(TypeTarget::NestedAnimation)
                            | TypeResolution::Unresolved
                    )
                )
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
    sites: HashMap<GuardKind, BTreeSet<usize>>,
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
            sites: HashMap::new(),
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

    fn record(&mut self, kind: GuardKind, span: Span) {
        let site_offset = self.offset(span.start());
        self.sites.entry(kind).or_default().insert(site_offset);
        let offset = self.function_start.unwrap_or(site_offset);
        self.hits.entry(kind).or_default().insert(offset);
    }

    fn resolve_type(&self, names: &[String]) -> Option<TypeResolution> {
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
        let resolution = self.resolve_type(prefix);
        let resolved = resolution == Some(TypeResolution::Target(TypeTarget::NestedAnimation));
        let unresolved = resolution == Some(TypeResolution::Unresolved);
        if !resolved && !unresolved && !self.direct_selection_is_guarded {
            return;
        }
        let explicitly_canonical = prefix
            .iter()
            .any(|segment| segment == "RuntimeNestedAnimationInstance" || segment == "Self");
        if unresolved || !resolved || !explicitly_canonical || self.direct_selection_is_guarded {
            self.record(GuardKind::Selection, span);
        }
    }

    fn macro_contains_guard(tokens: TokenStream, kind: GuardKind) -> bool {
        tokens.into_iter().any(|token| match token {
            TokenTree::Ident(ident) => {
                let name = ident_name(&ident);
                kind.matches_member(&name) || kind.matches_guarded_type(&name)
            }
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
            let path_guarded = item.path.segments.last().is_some_and(|segment| {
                let name = ident_name(&segment.ident);
                kind.matches_member(&name) || kind.matches_guarded_type(&name)
            });
            if path_guarded || Self::macro_contains_guard(item.tokens.clone(), kind) {
                self.record(kind, item.span());
            }
        }
        visit::visit_macro(self, item);
    }

    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        let tokens = match &attribute.meta {
            Meta::Path(_) | Meta::NameValue(_) => TokenStream::new(),
            Meta::List(list) => list.tokens.clone(),
        };
        for kind in GuardKind::ALL {
            let path_guarded = attribute.path().segments.last().is_some_and(|segment| {
                let name = ident_name(&segment.ident);
                kind.matches_member(&name) || kind.matches_guarded_type(&name)
            });
            if path_guarded || Self::macro_contains_guard(tokens.clone(), kind) {
                self.record(kind, attribute.span());
            }
        }
        visit::visit_attribute(self, attribute);
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
    let mut token_start = None;
    for (offset, character) in normalized
        .char_indices()
        .chain(std::iter::once((normalized.len(), '\0')))
    {
        if character.is_ascii_alphanumeric() || character == '_' {
            token_start.get_or_insert(offset);
            continue;
        }
        let Some(start) = token_start.take() else {
            continue;
        };
        let token = &normalized[start..offset];
        for kind in GuardKind::ALL {
            if kind.matches_member(token) || kind.matches_guarded_type(token) {
                hits.entry(kind).or_insert_with(BTreeSet::new).insert(start);
            }
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
            println!("hit {} 0", kind.name());
            for offset in offsets {
                println!("site {} {offset}", kind.name());
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
        if let Some(offsets) = analyzer.sites.get(&kind) {
            for offset in offsets {
                println!("site {} {offset}", kind.name());
            }
        }
    }
}
