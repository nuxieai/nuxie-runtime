use proc_macro2::{LineColumn, Span, TokenStream, TokenTree};
use quote::ToTokens;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::{self, Read};
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Block, Expr, ExprIf, ExprMatch, ExprMethodCall, ExprPath, File, ImplItem,
    ImplItemFn, Item, ItemFn, ItemImpl, ItemMod, ItemTrait, Local, Macro, Meta, Pat, PatStruct,
    PatTupleStruct, Path, Token, TraitItem, TraitItemFn, Type, TypePath, UseTree, Visibility,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.name() == name)
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
    plain_types: HashMap<String, String>,
    plain_members: HashMap<String, HashSet<String>>,
    plain_values: HashSet<String>,
    modules: HashMap<String, Box<Bindings>>,
    variants: HashSet<String>,
    guarded_values: HashMap<String, HashSet<GuardKind>>,
    nested_animation_glob: bool,
}

#[derive(Clone)]
struct UseEntry {
    path: Vec<String>,
    local: Option<String>,
    glob: bool,
    renamed: bool,
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

fn append_normalized_tokens(tokens: TokenStream, normalized: &mut String) {
    for token in tokens {
        match token {
            TokenTree::Group(group) => {
                let (open, close) = match group.delimiter() {
                    proc_macro2::Delimiter::Parenthesis => ('(', ')'),
                    proc_macro2::Delimiter::Brace => ('{', '}'),
                    proc_macro2::Delimiter::Bracket => ('[', ']'),
                    proc_macro2::Delimiter::None => ('\0', '\0'),
                };
                if open != '\0' {
                    normalized.push(open);
                }
                append_normalized_tokens(group.stream(), normalized);
                if close != '\0' {
                    normalized.push(close);
                }
            }
            TokenTree::Ident(ident) => normalized.push_str(&ident.to_string()),
            TokenTree::Punct(punct) => normalized.push(punct.as_char()),
            TokenTree::Literal(literal) => normalized.push_str(&literal.to_string()),
        }
    }
}

fn normalized_token_hash(value: &impl ToTokens) -> String {
    let mut normalized = String::new();
    append_normalized_tokens(value.to_token_stream(), &mut normalized);
    format!("{:x}", Sha256::digest(normalized.as_bytes()))
}

fn without_relative_prefixes(path: &[String]) -> &[String] {
    let first_absolute = path
        .iter()
        .position(|segment| !matches!(segment.as_str(), "crate" | "self" | "super"))
        .unwrap_or(path.len());
    &path[first_absolute..]
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
                renamed: false,
            });
        }
        UseTree::Rename(rename) => {
            let mut path = prefix.clone();
            path.push(ident_name(&rename.ident));
            entries.push(UseEntry {
                path,
                local: Some(ident_name(&rename.rename)),
                glob: false,
                renamed: true,
            });
        }
        UseTree::Glob(_) => entries.push(UseEntry {
            path: prefix.clone(),
            local: None,
            glob: true,
            renamed: false,
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

fn is_owner_export_visibility(visibility: &Visibility) -> bool {
    match visibility {
        Visibility::Public(_) => true,
        Visibility::Restricted(restricted) => restricted.path.is_ident("crate"),
        Visibility::Inherited => false,
    }
}

fn pattern_is_catch_all(pattern: &Pat) -> bool {
    match pattern {
        Pat::Ident(pattern) => pattern.subpat.is_none(),
        Pat::Wild(_) => true,
        Pat::Or(pattern) => pattern.cases.iter().any(pattern_is_catch_all),
        Pat::Paren(pattern) => pattern_is_catch_all(&pattern.pat),
        Pat::Reference(pattern) => pattern_is_catch_all(&pattern.pat),
        Pat::Type(pattern) => pattern_is_catch_all(&pattern.pat),
        _ => false,
    }
}

#[derive(Default)]
struct PatternPathVisitor {
    paths: Vec<Vec<String>>,
}

impl<'ast> Visit<'ast> for PatternPathVisitor {
    fn visit_path(&mut self, path: &'ast Path) {
        self.paths.push(path_names(path));
        visit::visit_path(self, path);
    }
}

struct MatchesInput {
    _expression: Expr,
    pattern: Pat,
}

impl Parse for MatchesInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let expression = input.parse()?;
        input.parse::<Token![,]>()?;
        let pattern = Pat::parse_multi_with_leading_vert(input)?;
        Ok(Self {
            _expression: expression,
            pattern,
        })
    }
}

fn resolution_in_bindings(path: &[String], bindings: &Bindings) -> Option<TypeResolution> {
    let path = without_relative_prefixes(path);
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

fn plain_resolution_in_bindings(path: &[String], bindings: &Bindings) -> Option<String> {
    let path = without_relative_prefixes(path);
    let (first, rest) = path.split_first()?;
    if rest.is_empty() {
        return bindings.plain_types.get(first).cloned();
    }
    bindings
        .modules
        .get(first)
        .and_then(|module| plain_resolution_in_bindings(rest, module))
}

fn plain_path_target(path: &[String], scopes: &[Bindings]) -> Option<String> {
    for scope in scopes.iter().rev() {
        if let Some(target) = plain_resolution_in_bindings(path, scope) {
            return Some(target);
        }
    }
    None
}

fn module_resolution_in_bindings(path: &[String], bindings: &Bindings) -> Option<Bindings> {
    let path = without_relative_prefixes(path);
    let (first, rest) = path.split_first()?;
    let module = bindings.modules.get(first)?;
    if rest.is_empty() {
        return Some(module.as_ref().clone());
    }
    module_resolution_in_bindings(rest, module)
}

fn module_path_target(path: &[String], scopes: &[Bindings]) -> Option<Bindings> {
    for scope in scopes.iter().rev() {
        if let Some(module) = module_resolution_in_bindings(path, scope) {
            return Some(module);
        }
    }
    None
}

fn bindings_contain_plain_member(bindings: &Bindings, target: &str, member: &str) -> bool {
    bindings
        .plain_members
        .get(target)
        .is_some_and(|members| members.contains(member))
        || bindings
            .modules
            .values()
            .any(|module| bindings_contain_plain_member(module, target, member))
}

fn normalized_type_path(ty: &Type) -> Option<&syn::TypePath> {
    match ty {
        Type::Path(path) => Some(path),
        Type::Group(group) => normalized_type_path(&group.elem),
        Type::Paren(paren) => normalized_type_path(&paren.elem),
        _ => None,
    }
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

fn bindings_for_items(
    items: &[Item],
    scopes: &[Bindings],
    guarded_aliases: &[(GuardKind, String)],
) -> Bindings {
    let mut bindings = Bindings::default();
    for (kind, alias) in guarded_aliases {
        bindings
            .guarded_values
            .entry(alias.clone())
            .or_default()
            .insert(*kind);
    }

    let mut uses = Vec::new();
    let mut type_aliases = Vec::new();
    let mut plain_type_aliases = Vec::new();
    let mut plain_impls = Vec::new();
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
                if let Some(path) = normalized_type_path(&item_type.ty) {
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
                    let alias = ident_name(&item_type.ident);
                    if let TypeAliasTarget::Path(path) = &target {
                        plain_type_aliases.push((alias.clone(), path.clone()));
                    }
                    type_aliases.push((alias, target));
                }
            }
            Item::Enum(item_enum) => {
                let name = ident_name(&item_enum.ident);
                let start = item_enum.span().start();
                let target = format!("{name}@{}:{}", start.line, start.column);
                bindings.plain_types.insert(name, target.clone());
                bindings.plain_members.insert(
                    target,
                    item_enum
                        .variants
                        .iter()
                        .map(|variant| ident_name(&variant.ident))
                        .collect(),
                );
            }
            Item::Struct(item_struct) => {
                let name = ident_name(&item_struct.ident);
                let start = item_struct.span().start();
                let target = format!("{name}@{}:{}", start.line, start.column);
                bindings.plain_types.insert(name, target);
            }
            Item::Union(item_union) => {
                let name = ident_name(&item_union.ident);
                let start = item_union.span().start();
                let target = format!("{name}@{}:{}", start.line, start.column);
                bindings.plain_types.insert(name, target);
            }
            Item::Fn(item_fn) => {
                bindings.plain_values.insert(ident_name(&item_fn.sig.ident));
            }
            Item::Impl(item_impl) => {
                if let Some(self_type) = normalized_type_path(&item_impl.self_ty) {
                    if self_type.qself.is_none() {
                        let members = item_impl
                            .items
                            .iter()
                            .filter_map(|impl_item| match impl_item {
                                ImplItem::Const(item) => Some(ident_name(&item.ident)),
                                ImplItem::Fn(item) => Some(ident_name(&item.sig.ident)),
                                ImplItem::Type(item) => Some(ident_name(&item.ident)),
                                _ => None,
                            })
                            .collect::<HashSet<_>>();
                        plain_impls.push((path_names(&self_type.path), members));
                    }
                }
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
                    let nested = bindings_for_items(children, scopes, guarded_aliases);
                    bindings
                        .modules
                        .insert(ident_name(&item_mod.ident), Box::new(nested));
                }
            }
            _ => {}
        }
    }

    let mut visible = scopes.to_vec();
    visible.push(bindings.clone());
    for (self_path, members) in plain_impls {
        if let Some(target) = plain_path_target(&self_path, &visible) {
            bindings
                .plain_members
                .entry(target)
                .or_default()
                .extend(members);
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
        for (alias, path) in &plain_type_aliases {
            if bindings.plain_types.contains_key(alias) {
                continue;
            }
            let mut visible = scopes.to_vec();
            visible.push(bindings.clone());
            if let Some(target) = plain_path_target(path, &visible) {
                bindings.plain_types.insert(alias.clone(), target);
                changed = true;
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
            if let Some(target) = plain_path_target(&entry.path, &visible) {
                if bindings.plain_types.insert(local.clone(), target.clone()) != Some(target) {
                    changed = true;
                }
                continue;
            }
            if let Some(module) = module_path_target(&entry.path, &visible) {
                if !bindings.modules.contains_key(local) {
                    bindings.modules.insert(local.clone(), Box::new(module));
                    changed = true;
                }
                continue;
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
                    changed |= bindings
                        .guarded_values
                        .entry(local.clone())
                        .or_default()
                        .insert(kind);
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

#[derive(Default)]
struct PatternBindingVisitor {
    names: HashSet<String>,
}

impl<'ast> Visit<'ast> for PatternBindingVisitor {
    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        self.names.insert(ident_name(&pattern.ident));
        visit::visit_pat_ident(self, pattern);
    }
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
    guarded_aliases: Vec<(GuardKind, String)>,
    hits: HashMap<GuardKind, BTreeSet<usize>>,
    sites: HashMap<GuardKind, BTreeSet<usize>>,
    matches: HashMap<GuardKind, BTreeSet<(usize, usize, String, String, String)>>,
    enclosing_item: Option<(usize, String)>,
    item_context: Vec<String>,
    item_hash_context: Vec<String>,
    hash_context: Vec<String>,
    export_stack: Vec<(String, HashSet<GuardKind>)>,
    exports: BTreeSet<(GuardKind, String)>,
    direct_selection_is_guarded: bool,
}

impl<'a> Analyzer<'a> {
    fn new(source: &'a str, guarded_aliases: Vec<(GuardKind, String)>) -> Self {
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
            guarded_aliases,
            hits: HashMap::new(),
            sites: HashMap::new(),
            matches: HashMap::new(),
            enclosing_item: None,
            item_context: Vec::new(),
            item_hash_context: Vec::new(),
            hash_context: Vec::new(),
            export_stack: Vec::new(),
            exports: BTreeSet::new(),
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

    fn record(&mut self, kind: GuardKind, span: Span, guarded_name: &str) {
        let site_offset = self.offset(span.start());
        let site_hash = self
            .hash_context
            .last()
            .or_else(|| self.item_hash_context.last())
            .cloned()
            .unwrap_or_else(|| normalized_token_hash(&guarded_name));
        self.sites.entry(kind).or_default().insert(site_offset);
        let (offset, anchor) = self
            .enclosing_item
            .clone()
            .unwrap_or((site_offset, "file".to_owned()));
        self.hits.entry(kind).or_default().insert(offset);
        self.matches.entry(kind).or_default().insert((
            offset,
            site_offset,
            anchor,
            guarded_name.to_owned(),
            site_hash,
        ));
        for (_, kinds) in &mut self.export_stack {
            kinds.insert(kind);
        }
    }

    fn qualified_anchor(&self, name: &str) -> String {
        self.item_context
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(name))
            .collect::<Vec<_>>()
            .join("::")
    }

    fn item_anchor_name(&self, item: &Item) -> Option<String> {
        let name = match item {
            Item::Const(item) => ident_name(&item.ident),
            Item::Enum(item) => ident_name(&item.ident),
            Item::ExternCrate(item) => ident_name(&item.ident),
            Item::Fn(item) => ident_name(&item.sig.ident),
            Item::Macro(item) => item.ident.as_ref().map(ident_name).unwrap_or_else(|| {
                let start = item.span().start();
                format!("macro_{}_{}", start.line, start.column)
            }),
            Item::Mod(item) => ident_name(&item.ident),
            Item::Static(item) => ident_name(&item.ident),
            Item::Struct(item) => ident_name(&item.ident),
            Item::Trait(item) => ident_name(&item.ident),
            Item::TraitAlias(item) => ident_name(&item.ident),
            Item::Type(item) => ident_name(&item.ident),
            Item::Union(item) => ident_name(&item.ident),
            Item::Use(item) => {
                let start = item.span().start();
                format!("use_{}_{}", start.line, start.column)
            }
            _ => return None,
        };
        Some(self.qualified_anchor(&name))
    }

    fn impl_context_name(item: &ItemImpl) -> String {
        let self_name = normalized_type_path(&item.self_ty)
            .map(|path| path_names(&path.path).join("_"))
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| {
                let start = item.span().start();
                format!("impl_{}_{}", start.line, start.column)
            });
        item.trait_
            .as_ref()
            .map(|(_, trait_path, _)| {
                format!("{}_for_{self_name}", path_names(trait_path).join("_"))
            })
            .unwrap_or(self_name)
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

    fn guarded_value_has(&self, name: &str, kind: GuardKind) -> bool {
        self.scopes.iter().rev().any(|scope| {
            scope
                .guarded_values
                .get(name)
                .is_some_and(|kinds| kinds.contains(&kind))
        })
    }

    fn pattern_mentions_guarded_enum(&self, pattern: &Pat) -> bool {
        let mut visitor = PatternPathVisitor::default();
        visitor.visit_pat(pattern);
        visitor.paths.iter().any(|names| {
            names
                .iter()
                .any(|name| name == "RuntimeNestedAnimationInstance")
                || (names.len() > 1
                    && matches!(
                        self.resolve_type(&names[..names.len() - 1]),
                        Some(
                            TypeResolution::Target(TypeTarget::NestedAnimation)
                                | TypeResolution::Unresolved
                        )
                    ))
                || names.iter().enumerate().any(|(index, _)| {
                    matches!(
                        self.resolve_type(&names[..=index]),
                        Some(TypeResolution::Target(TypeTarget::NestedAnimation))
                    )
                })
        })
    }

    fn export_kinds_for_path(&self, names: &[String]) -> HashSet<GuardKind> {
        let mut kinds = HashSet::new();
        let Some(member) = names.last() else {
            return kinds;
        };
        for kind in GuardKind::ALL {
            if self.guarded_value_has(member, kind)
                || ((kind.matches_member(member) || kind.matches_guarded_type(member))
                    && !self.path_is_known_non_guarded(names))
            {
                kinds.insert(kind);
            }
        }
        if member == "StateMachine"
            || self.resolve_type(names) == Some(TypeResolution::Target(TypeTarget::NestedAnimation))
        {
            kinds.insert(GuardKind::Selection);
        }
        if kinds.is_empty() && !self.path_is_known_non_guarded(names) {
            // A public owner re-export with an unresolved tail must not create
            // a neutral spelling that escapes the non-owner pass.
            kinds.extend(GuardKind::ALL);
        }
        kinds
    }

    fn finish_export(&mut self) {
        let Some((name, kinds)) = self.export_stack.pop() else {
            return;
        };
        self.exports
            .extend(kinds.into_iter().map(|kind| (kind, name.clone())));
    }

    fn path_is_known_non_guarded(&self, names: &[String]) -> bool {
        let names = without_relative_prefixes(names);
        let Some((member, prefix)) = names.split_last() else {
            return false;
        };
        if prefix.is_empty() {
            return self
                .scopes
                .iter()
                .rev()
                .any(|scope| scope.plain_values.contains(member));
        }
        let Some(target) = plain_path_target(prefix, &self.scopes) else {
            return false;
        };
        self.scopes
            .iter()
            .rev()
            .any(|scope| bindings_contain_plain_member(scope, &target, member))
    }

    fn analyze_names(&mut self, names: &[String], span: Span) {
        let Some(member) = names.last() else {
            return;
        };
        let known_non_guarded = self.path_is_known_non_guarded(names);

        for kind in [GuardKind::Collection, GuardKind::Dispatch, GuardKind::Audio] {
            if kind.matches_member(member) && !known_non_guarded {
                // The final-segment rule is intentionally independent of how
                // much of the prefix can be resolved. Only a fully resolved
                // local non-guarded item is exempt.
                self.record(kind, span, member);
            } else if self.guarded_value_has(member, kind) {
                self.record(kind, span, member);
            }
        }

        if self.guarded_value_has(member, GuardKind::Selection) {
            self.record(GuardKind::Selection, span, member);
            return;
        }
        if member == "StateMachine" {
            if !known_non_guarded {
                self.record(GuardKind::Selection, span, member);
            }
            return;
        }
        if names.len() == 1 {
            if self.variant_is_bound(member)
                || self.guarded_value_has(member, GuardKind::Selection)
                || (member == "StateMachine" && self.direct_selection_is_guarded)
            {
                self.record(GuardKind::Selection, span, member);
            }
            return;
        }
        if self.direct_selection_is_guarded
            && self.resolve_type(&names[..names.len() - 1])
                == Some(TypeResolution::Target(TypeTarget::NestedAnimation))
        {
            self.record(GuardKind::Selection, span, member);
        }
    }

    fn analyze_path(&mut self, path: &Path, _qself: Option<&syn::QSelf>) {
        self.analyze_names(&path_names(path), path.span());
    }

    fn append_identifier_fragments(
        tokens: TokenStream,
        normalized: &mut String,
        fragments: &mut Vec<String>,
    ) {
        for token in tokens {
            match token {
                TokenTree::Ident(ident) => {
                    let fragment = ident_name(&ident);
                    normalized.push_str(&fragment);
                    fragments.push(fragment);
                }
                TokenTree::Group(group) => {
                    Self::append_identifier_fragments(group.stream(), normalized, fragments);
                }
                _ => {}
            }
        }
    }

    fn fragments_compose_guarded_name(fragments: &[String], guarded_name: &str) -> bool {
        let mut counts = BTreeMap::<&str, usize>::new();
        for fragment in fragments {
            if !fragment.is_empty()
                && fragment.len() <= guarded_name.len()
                && guarded_name.contains(fragment)
            {
                *counts.entry(fragment).or_default() += 1;
            }
        }
        let fragments = counts.keys().copied().collect::<Vec<_>>();
        let mut available = counts.values().copied().collect::<Vec<_>>();
        let mut failed = HashSet::new();

        fn search(
            guarded_name: &str,
            offset: usize,
            fragments: &[&str],
            available: &mut [usize],
            failed: &mut HashSet<(usize, Vec<usize>)>,
        ) -> bool {
            if offset == guarded_name.len() {
                return true;
            }
            let state = (offset, available.to_vec());
            if failed.contains(&state) {
                return false;
            }
            for (index, fragment) in fragments.iter().enumerate() {
                if available[index] == 0 || !guarded_name[offset..].starts_with(fragment) {
                    continue;
                }
                available[index] -= 1;
                if search(
                    guarded_name,
                    offset + fragment.len(),
                    fragments,
                    available,
                    failed,
                ) {
                    available[index] += 1;
                    return true;
                }
                available[index] += 1;
            }
            failed.insert(state);
            false
        }

        search(guarded_name, 0, &fragments, &mut available, &mut failed)
    }

    fn macro_guarded_name(tokens: TokenStream, kind: GuardKind) -> Option<String> {
        let mut normalized = String::new();
        let mut fragments = Vec::new();
        Self::append_identifier_fragments(tokens, &mut normalized, &mut fragments);
        let guarded_names: &[&str] = match kind {
            GuardKind::Collection => &[
                "reported_event_count",
                "reported_event",
                "StateMachineReportedEvent",
                "reported_events",
            ],
            GuardKind::Selection => &["RuntimeNestedAnimationInstance", "StateMachine"],
            GuardKind::Dispatch => &["notify_events", "StateMachineInstance"],
            GuardKind::Audio => &[
                "flush_deferred_owner_audio_events",
                "flush_deferred_owner_audio_event",
                "defer_recorded_audio_event_seam",
                "reach_recorded_audio_event_seam",
                "deliver_recorded_audio_occurrence",
                "StateMachineInstance",
            ],
        };
        let exact = guarded_names.iter().copied().find(|name| {
            normalized.contains(name) || Self::fragments_compose_guarded_name(&fragments, name)
        });
        if let Some(name) = exact {
            return Some(name.to_owned());
        }
        if kind == GuardKind::Collection {
            for (offset, _) in normalized.match_indices("take_") {
                let suffix = &normalized[offset..];
                if suffix.contains("event") || suffix.contains("report") {
                    return Some("take_event_or_report".to_owned());
                }
            }
        }
        None
    }

    fn push_item_scope(&mut self, items: &[Item]) {
        let bindings = bindings_for_items(items, &self.scopes, &self.guarded_aliases);
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

    fn add_local_pattern_bindings(&mut self, pattern: &syn::Pat) {
        let mut visitor = PatternBindingVisitor::default();
        visitor.visit_pat(pattern);
        if let Some(scope) = self.scopes.last_mut() {
            scope.plain_values.extend(visitor.names);
        }
    }
}

impl<'ast> Visit<'ast> for Analyzer<'_> {
    fn visit_file(&mut self, file: &'ast File) {
        self.push_item_scope(&file.items);
        visit::visit_file(self, file);
        self.scopes.pop();
    }

    fn visit_item(&mut self, item: &'ast Item) {
        if cfg_test(item_attrs(item)) {
            return;
        }
        self.item_hash_context.push(normalized_token_hash(item));
        let previous_item = self.item_anchor_name(item).map(|anchor| {
            self.enclosing_item
                .replace((self.offset(item.span().start()), anchor))
        });
        visit::visit_item(self, item);
        if let Some(previous_item) = previous_item {
            self.enclosing_item = previous_item;
        }
        self.item_hash_context.pop();
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        if cfg_test(&item.attrs) {
            return;
        }
        for attribute in &item.attrs {
            self.visit_attribute(attribute);
        }
        if let Some((_, items)) = &item.content {
            self.item_context.push(ident_name(&item.ident));
            self.push_item_scope(items);
            for child in items {
                self.visit_item(child);
            }
            self.scopes.pop();
            self.item_context.pop();
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        if cfg_test(&item.attrs) {
            return;
        }
        let context = Self::impl_context_name(item);
        let previous_item = self.enclosing_item.replace((
            self.offset(item.span().start()),
            self.qualified_anchor(&context),
        ));
        self.item_context.push(context);
        visit::visit_item_impl(self, item);
        self.item_context.pop();
        self.enclosing_item = previous_item;
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        if cfg_test(&item.attrs) {
            return;
        }
        self.item_context.push(ident_name(&item.ident));
        visit::visit_item_trait(self, item);
        self.item_context.pop();
    }

    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        let name = match item {
            ImplItem::Const(item) => Some(ident_name(&item.ident)),
            ImplItem::Fn(item) => Some(ident_name(&item.sig.ident)),
            ImplItem::Macro(item) => item
                .mac
                .path
                .segments
                .last()
                .map(|segment| format!("macro_{}", ident_name(&segment.ident))),
            ImplItem::Type(item) => Some(ident_name(&item.ident)),
            _ => None,
        };
        let previous_item = name.map(|name| {
            self.enclosing_item.replace((
                self.offset(item.span().start()),
                self.qualified_anchor(&name),
            ))
        });
        visit::visit_impl_item(self, item);
        if let Some(previous_item) = previous_item {
            self.enclosing_item = previous_item;
        }
    }

    fn visit_trait_item(&mut self, item: &'ast TraitItem) {
        let name = match item {
            TraitItem::Const(item) => Some(ident_name(&item.ident)),
            TraitItem::Fn(item) => Some(ident_name(&item.sig.ident)),
            TraitItem::Macro(item) => item
                .mac
                .path
                .segments
                .last()
                .map(|segment| format!("macro_{}", ident_name(&segment.ident))),
            TraitItem::Type(item) => Some(ident_name(&item.ident)),
            _ => None,
        };
        let previous_item = name.map(|name| {
            self.enclosing_item.replace((
                self.offset(item.span().start()),
                self.qualified_anchor(&name),
            ))
        });
        visit::visit_trait_item(self, item);
        if let Some(previous_item) = previous_item {
            self.enclosing_item = previous_item;
        }
    }

    fn visit_trait_item_fn(&mut self, item: &'ast TraitItemFn) {
        if cfg_test(&item.attrs) {
            return;
        }
        for attribute in &item.attrs {
            self.visit_attribute(attribute);
        }
        self.visit_signature(&item.sig);
        let name = ident_name(&item.sig.ident);
        let previous_item = self.enclosing_item.replace((
            self.offset(item.span().start()),
            self.qualified_anchor(&name),
        ));
        if let Some(block) = &item.default {
            let previous_selection = std::mem::replace(
                &mut self.direct_selection_is_guarded,
                block_has_event_mechanic(block),
            );
            self.item_context.push(name);
            self.visit_block(block);
            self.item_context.pop();
            self.direct_selection_is_guarded = previous_selection;
        }
        self.enclosing_item = previous_item;
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        if cfg_test(&item.attrs) {
            return;
        }
        for attribute in &item.attrs {
            self.visit_attribute(attribute);
        }
        self.visit_visibility(&item.vis);
        self.visit_signature(&item.sig);
        let name = ident_name(&item.sig.ident);
        let previous_item = self.enclosing_item.replace((
            self.offset(item.span().start()),
            self.qualified_anchor(&name),
        ));
        let previous_selection = std::mem::replace(
            &mut self.direct_selection_is_guarded,
            block_has_event_mechanic(&item.block),
        );
        if is_owner_export_visibility(&item.vis) {
            self.export_stack.push((name.clone(), HashSet::new()));
        }
        self.item_context.push(name);
        self.visit_block(&item.block);
        self.item_context.pop();
        if is_owner_export_visibility(&item.vis) {
            self.finish_export();
        }
        self.direct_selection_is_guarded = previous_selection;
        self.enclosing_item = previous_item;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        if cfg_test(&item.attrs) {
            return;
        }
        for attribute in &item.attrs {
            self.visit_attribute(attribute);
        }
        self.visit_visibility(&item.vis);
        self.visit_signature(&item.sig);
        let name = ident_name(&item.sig.ident);
        let previous_item = self.enclosing_item.replace((
            self.offset(item.span().start()),
            self.qualified_anchor(&name),
        ));
        let previous_selection = std::mem::replace(
            &mut self.direct_selection_is_guarded,
            block_has_event_mechanic(&item.block),
        );
        if is_owner_export_visibility(&item.vis) {
            self.export_stack.push((name.clone(), HashSet::new()));
        }
        self.item_context.push(name);
        self.visit_block(&item.block);
        self.item_context.pop();
        if is_owner_export_visibility(&item.vis) {
            self.finish_export();
        }
        self.direct_selection_is_guarded = previous_selection;
        self.enclosing_item = previous_item;
    }

    fn visit_block(&mut self, block: &'ast Block) {
        self.push_block_scope(block);
        for statement in &block.stmts {
            self.visit_stmt(statement);
            if let syn::Stmt::Local(local) = statement {
                self.add_local_pattern_bindings(&local.pat);
            }
        }
        self.scopes.pop();
    }

    fn visit_stmt(&mut self, statement: &'ast syn::Stmt) {
        self.hash_context.push(normalized_token_hash(statement));
        visit::visit_stmt(self, statement);
        self.hash_context.pop();
    }

    fn visit_expr_path(&mut self, expression: &'ast ExprPath) {
        self.analyze_path(&expression.path, expression.qself.as_ref());
        visit::visit_expr_path(self, expression);
    }

    fn visit_type_path(&mut self, path: &'ast TypePath) {
        self.analyze_path(&path.path, path.qself.as_ref());
        visit::visit_type_path(self, path);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        let mut entries = Vec::new();
        flatten_use(&item.tree, &mut Vec::new(), &mut entries);
        for entry in &entries {
            if entry.path.last().is_some_and(|member| {
                GuardKind::ALL
                    .iter()
                    .any(|kind| kind.matches_member(member))
            }) {
                self.analyze_names(&entry.path, item.span());
            }
            if is_owner_export_visibility(&item.vis) && entry.renamed {
                let Some(local) = &entry.local else {
                    continue;
                };
                self.exports.extend(
                    self.export_kinds_for_path(&entry.path)
                        .into_iter()
                        .map(|kind| (kind, local.clone())),
                );
            }
        }
        visit::visit_item_use(self, item);
    }

    fn visit_expr_match(&mut self, expression: &'ast ExprMatch) {
        if expression
            .arms
            .iter()
            .any(|arm| self.pattern_mentions_guarded_enum(&arm.pat))
        {
            for arm in &expression.arms {
                if pattern_is_catch_all(&arm.pat) {
                    self.record(
                        GuardKind::Selection,
                        arm.pat.span(),
                        "RuntimeNestedAnimationInstance",
                    );
                }
            }
        }
        visit::visit_expr_match(self, expression);
    }

    fn visit_expr_if(&mut self, expression: &'ast ExprIf) {
        if let Expr::Let(condition) = expression.cond.as_ref()
            && self.pattern_mentions_guarded_enum(&condition.pat)
        {
            // A refutable `if let` has an implicit complement even though syn
            // exposes no wildcard pattern for that path.
            self.record(
                GuardKind::Selection,
                condition.pat.span(),
                "RuntimeNestedAnimationInstance",
            );
        }
        visit::visit_expr_if(self, expression);
    }

    fn visit_local(&mut self, local: &'ast Local) {
        if local
            .init
            .as_ref()
            .is_some_and(|init| init.diverge.is_some())
            && self.pattern_mentions_guarded_enum(&local.pat)
        {
            // `let PAT = value else { ... }` selects PAT's implicit
            // complement in the diverging branch.
            self.record(
                GuardKind::Selection,
                local.pat.span(),
                "RuntimeNestedAnimationInstance",
            );
        }
        visit::visit_local(self, local);
    }

    fn visit_pat_tuple_struct(&mut self, pattern: &'ast PatTupleStruct) {
        self.analyze_path(&pattern.path, pattern.qself.as_ref());
        visit::visit_pat_tuple_struct(self, pattern);
    }

    fn visit_pat_struct(&mut self, pattern: &'ast PatStruct) {
        self.analyze_path(&pattern.path, pattern.qself.as_ref());
        visit::visit_pat_struct(self, pattern);
    }

    fn visit_expr_method_call(&mut self, expression: &'ast ExprMethodCall) {
        let member = ident_name(&expression.method);
        for kind in GuardKind::ALL {
            if kind.matches_member(&member) || self.guarded_value_has(&member, kind) {
                self.record(kind, expression.method.span(), &member);
            }
        }
        visit::visit_expr_method_call(self, expression);
    }

    fn visit_macro(&mut self, item: &'ast Macro) {
        if item.path.is_ident("matches")
            && syn::parse2::<MatchesInput>(item.tokens.clone())
                .is_ok_and(|input| self.pattern_mentions_guarded_enum(&input.pattern))
        {
            // `matches!` has no catch-all arm in its AST surface. Requiring a
            // registry row for every guarded-enum pattern mention closes
            // complement-by-negation without attempting control-flow proof.
            self.record(
                GuardKind::Selection,
                item.span(),
                "RuntimeNestedAnimationInstance",
            );
        }
        for kind in GuardKind::ALL {
            let path_guarded = item.path.segments.last().and_then(|segment| {
                let name = ident_name(&segment.ident);
                (kind.matches_member(&name) || kind.matches_guarded_type(&name)).then_some(name)
            });
            let guarded_name = Self::macro_guarded_name(item.tokens.clone(), kind);
            if path_guarded.is_some() || guarded_name.is_some() {
                let name = guarded_name
                    .or(path_guarded)
                    .unwrap_or_else(|| kind.name().to_owned());
                self.record(kind, item.span(), &name);
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
            let path_guarded = attribute.path().segments.last().and_then(|segment| {
                let name = ident_name(&segment.ident);
                (kind.matches_member(&name) || kind.matches_guarded_type(&name)).then_some(name)
            });
            let guarded_name = Self::macro_guarded_name(tokens.clone(), kind);
            if path_guarded.is_some() || guarded_name.is_some() {
                let name = guarded_name
                    .or(path_guarded)
                    .unwrap_or_else(|| kind.name().to_owned());
                self.record(kind, attribute.span(), &name);
            }
        }
        visit::visit_attribute(self, attribute);
    }
}

fn guarded_value_exports(file: &File) -> BTreeSet<(GuardKind, String)> {
    fn collect(items: &[Item], aliases: &mut BTreeSet<(GuardKind, String)>) {
        for item in items {
            if cfg_test(item_attrs(item)) {
                continue;
            }
            match item {
                Item::Const(item) => {
                    if !is_owner_export_visibility(&item.vis) {
                        continue;
                    }
                    if let syn::Expr::Path(path) = item.expr.as_ref()
                        && let Some(member) = path.path.segments.last()
                    {
                        let member = ident_name(&member.ident);
                        for kind in GuardKind::ALL {
                            if kind.matches_member(&member) {
                                aliases.insert((kind, ident_name(&item.ident)));
                            }
                        }
                    }
                }
                Item::Static(item) => {
                    if !is_owner_export_visibility(&item.vis) {
                        continue;
                    }
                    if let syn::Expr::Path(path) = item.expr.as_ref()
                        && let Some(member) = path.path.segments.last()
                    {
                        let member = ident_name(&member.ident);
                        for kind in GuardKind::ALL {
                            if kind.matches_member(&member) {
                                aliases.insert((kind, ident_name(&item.ident)));
                            }
                        }
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
    let guarded_aliases = std::env::args()
        .skip(1)
        .filter_map(|argument| {
            let (kind, alias) = argument.split_once(':')?;
            Some((GuardKind::parse(kind)?, alias.to_owned()))
        })
        .collect::<Vec<_>>();
    let Ok(file) = syn::parse_file(&source) else {
        for (kind, offsets) in lexical_tripwire(&source) {
            println!("hit {} 0", kind.name());
            for offset in offsets {
                println!("site {} {offset}", kind.name());
            }
        }
        return;
    };

    let mut analyzer = Analyzer::new(&source, guarded_aliases);
    analyzer.visit_file(&file);
    analyzer.exports.extend(guarded_value_exports(&file));
    for (kind, alias) in &analyzer.exports {
        println!("export {} {alias}", kind.name());
    }
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
        if let Some(records) = analyzer.matches.get(&kind) {
            for (anchor_offset, site_offset, anchor, guarded_name, site_hash) in records {
                println!(
                    "match {} {anchor_offset} {site_offset} {anchor} {guarded_name} {site_hash}",
                    kind.name()
                );
            }
        }
    }
}
