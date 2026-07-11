use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let args = Args::parse()?;
    let schema = Schema::load(&args.defs)?;
    let generated = schema.render()?;

    if let Some(parent) = args.out.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating output directory {}", parent.display()))?;
    }
    fs::write(&args.out, generated)
        .with_context(|| format!("writing generated schema {}", args.out.display()))?;

    eprintln!(
        "generated {} runtime definitions with {} runtime properties into {}",
        schema.runtime_definition_count(),
        schema.runtime_property_count(),
        args.out.display()
    );

    Ok(())
}

#[derive(Debug)]
struct Args {
    defs: PathBuf,
    out: PathBuf,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut defs = None;
        let mut out = None;
        let mut args = std::env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--defs" => {
                    defs = Some(PathBuf::from(
                        args.next().context("--defs requires a path argument")?,
                    ));
                }
                "--out" => {
                    out = Some(PathBuf::from(
                        args.next().context("--out requires a path argument")?,
                    ));
                }
                "--help" | "-h" => {
                    println!("usage: nuxie-codegen --defs <defs-dir> --out <schema.rs>");
                    std::process::exit(0);
                }
                _ => bail!("unknown argument {arg:?}"),
            }
        }

        Ok(Self {
            defs: defs.context("missing --defs <defs-dir>")?,
            out: out.context("missing --out <schema.rs>")?,
        })
    }
}

#[derive(Debug)]
struct Schema {
    defs: BTreeMap<String, RawDefinition>,
}

impl Schema {
    fn load(defs_dir: &Path) -> Result<Self> {
        let mut files = Vec::new();
        collect_json_files(defs_dir, defs_dir, &mut files)?;

        let mut defs = BTreeMap::new();
        for relative in files {
            let path = defs_dir.join(&relative);
            let contents =
                fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
            let mut def: RawDefinition = serde_json::from_str(&contents)
                .with_context(|| format!("parsing {}", path.display()))?;
            def.file = relative.clone();
            if defs.insert(relative.clone(), def).is_some() {
                bail!("duplicate definition file {relative}");
            }
        }

        let schema = Self { defs };
        schema.validate_extends()?;
        schema.validate_definition_keys()?;
        schema.validate_property_keys()?;
        schema.validate_bitmask_passthroughs()?;
        Ok(schema)
    }

    fn runtime_definition_count(&self) -> usize {
        self.runtime_definitions().count()
    }

    fn runtime_property_count(&self) -> usize {
        self.runtime_definitions()
            .flat_map(|(_, def)| def.runtime_properties())
            .count()
    }

    fn runtime_definitions(&self) -> impl Iterator<Item = (&String, &RawDefinition)> {
        self.defs
            .iter()
            .filter(|(_, def)| def.runtime && def.type_key().is_some())
    }

    fn render(&self) -> Result<String> {
        let mut defs = self
            .runtime_definitions()
            .map(|(file, def)| {
                let type_key = def
                    .type_key()
                    .with_context(|| format!("{file} is missing a numeric type key"))?;
                Ok(RuntimeDefinition {
                    file: file.clone(),
                    name: def
                        .name
                        .clone()
                        .with_context(|| format!("{file} is missing a name"))?,
                    variant: rust_variant(
                        def.name
                            .as_deref()
                            .with_context(|| format!("{file} is missing a name"))?,
                    )?,
                    key: type_key,
                    key_name: def
                        .key
                        .as_ref()
                        .and_then(|key| key.string.clone())
                        .unwrap_or_else(|| def.name.clone().unwrap_or_default().to_lowercase()),
                    raw_parent_file: def.extends.clone(),
                    runtime_parent_file: self.runtime_parent_file(file)?,
                    mixins: def.mixins.clone(),
                    generic: def.generic.clone(),
                    generic_pass_through: def.generic_pass_through.clone(),
                    exports_with_context: def.exports_with_context,
                    abstract_: def.abstract_,
                    properties: def.runtime_properties().collect(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        defs.sort_by_key(|def| def.key);
        validate_variants(&defs)?;

        let mut out = String::new();
        out.push_str("// @generated by `nuxie-codegen`; do not edit by hand.\n");
        out.push_str("// Source: Rive runtime `dev/defs` JSON.\n\n");
        out.push_str(
            "use crate::{BitmaskPassthrough, CoreRegistryFieldKind, Definition, FieldKind, Key, Property};\n\n",
        );

        out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
        out.push_str("pub enum ObjectKind {\n");
        for def in &defs {
            out.push_str(&format!("    {},\n", def.variant));
        }
        out.push_str("}\n\n");

        out.push_str("impl ObjectKind {\n");
        out.push_str("    pub fn definition(self) -> &'static Definition {\n");
        out.push_str("        match self {\n");
        for (index, def) in defs.iter().enumerate() {
            out.push_str(&format!(
                "            Self::{} => &DEFINITIONS[{}],\n",
                def.variant, index
            ));
        }
        out.push_str("        }\n");
        out.push_str("    }\n\n");
        out.push_str("    pub fn type_key(self) -> u16 {\n");
        out.push_str("        self.definition().type_key.int\n");
        out.push_str("    }\n\n");
        out.push_str("    pub fn name(self) -> &'static str {\n");
        out.push_str("        self.definition().name\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");

        out.push_str(&self.render_definition_lookup_tables(&defs));

        let callback_keys = self.callback_property_keys(&defs)?;
        out.push_str("pub fn is_callback_property_key(key: u16) -> bool {\n");
        if callback_keys.is_empty() {
            out.push_str("    let _ = key;\n");
            out.push_str("    false\n");
        } else {
            let patterns = callback_keys
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join(" | ");
            out.push_str(&format!("    matches!(key, {patterns})\n"));
        }
        out.push_str("}\n\n");

        out.push_str(&self.render_core_registry_lookup_tables(&defs)?);

        for (index, def) in defs.iter().enumerate() {
            let ancestors = self.ancestor_names(&def.file)?;
            out.push_str(&format!("static DEF_{index}_ANCESTORS: &[&str] = &[\n"));
            for ancestor in ancestors {
                out.push_str("    ");
                out.push_str(&rust_string(&ancestor));
                out.push_str(",\n");
            }
            out.push_str("];\n\n");

            out.push_str(&format!(
                "static DEF_{index}_PROPERTIES: &[Property] = &[\n"
            ));
            for property in sorted_runtime_properties(def) {
                out.push_str(&self.render_property(&property, &def.file)?);
            }
            out.push_str("];\n\n");
        }

        out.push_str(&self.render_hierarchy_property_lookup_table(&defs)?);
        out.push_str(&self.render_bitmask_passthrough_lookup_table(&defs)?);

        out.push_str("pub static DEFINITIONS: &[Definition] = &[\n");
        for (index, def) in defs.iter().enumerate() {
            let runtime_parent = match &def.runtime_parent_file {
                Some(file) => Some(
                    self.defs
                        .get(file)
                        .and_then(|parent| parent.name.as_deref())
                        .with_context(|| format!("runtime parent {file} is missing a name"))?,
                ),
                None => None,
            };

            out.push_str("    Definition {\n");
            out.push_str(&format!("        name: {},\n", rust_string(&def.name)));
            out.push_str(&format!(
                "        rust_variant: {},\n",
                rust_string(&def.variant)
            ));
            out.push_str(&format!("        file: {},\n", rust_string(&def.file)));
            out.push_str(&format!(
                "        type_key: Key {{ int: {}, name: {} }},\n",
                def.key,
                rust_string(&def.key_name)
            ));
            out.push_str(&format!(
                "        runtime_parent: {},\n",
                option_string(runtime_parent)
            ));
            out.push_str(&format!(
                "        raw_parent_file: {},\n",
                option_string(def.raw_parent_file.as_deref())
            ));
            out.push_str(&format!("        mixins: {},\n", string_slice(&def.mixins)));
            out.push_str(&format!(
                "        generic: {},\n",
                option_string(def.generic.as_deref())
            ));
            out.push_str(&format!(
                "        generic_pass_through: {},\n",
                option_string(def.generic_pass_through.as_deref())
            ));
            out.push_str(&format!(
                "        exports_with_context: {},\n",
                def.exports_with_context
            ));
            out.push_str(&format!("        abstract_: {},\n", def.abstract_));
            out.push_str(&format!("        cloneable: {},\n", !def.abstract_));
            out.push_str(&format!("        properties: DEF_{index}_PROPERTIES,\n"));
            out.push_str(&format!("        ancestors: DEF_{index}_ANCESTORS,\n"));
            out.push_str("    },\n");
        }
        out.push_str("];\n");

        Ok(out)
    }

    fn render_definition_lookup_tables(&self, defs: &[RuntimeDefinition<'_>]) -> String {
        let mut out = String::new();

        out.push_str("pub fn object_kind_by_type_key(key: u16) -> Option<ObjectKind> {\n");
        out.push_str("    match key {\n");
        for def in defs {
            out.push_str(&format!(
                "        {} => Some(ObjectKind::{}),\n",
                def.key, def.variant
            ));
        }
        out.push_str("        _ => None,\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");

        out.push_str("pub fn definition_by_type_key(key: u16) -> Option<&'static Definition> {\n");
        out.push_str("    object_kind_by_type_key(key).map(ObjectKind::definition)\n");
        out.push_str("}\n\n");

        out.push_str("pub fn definition_by_name(name: &str) -> Option<&'static Definition> {\n");
        out.push_str("    match name {\n");
        for (index, def) in defs.iter().enumerate() {
            out.push_str(&format!(
                "        {} => Some(&DEFINITIONS[{}]),\n",
                rust_string(&def.name),
                index
            ));
        }
        out.push_str("        _ => None,\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");

        out
    }

    fn render_core_registry_lookup_tables(&self, defs: &[RuntimeDefinition<'_>]) -> Result<String> {
        let first_properties = first_properties_by_key(defs);
        let mut out = String::new();

        out.push_str(
            "pub fn core_registry_field_kind_by_property_key(key: u16) -> Option<CoreRegistryFieldKind> {\n",
        );
        out.push_str("    match key {\n");
        for (key, property) in &first_properties {
            let Some(kind) = core_registry_field_kind_variant(*property)? else {
                continue;
            };
            out.push_str(&format!(
                "        {key} => Some(CoreRegistryFieldKind::{kind}),\n"
            ));
        }
        out.push_str("        _ => None,\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");

        out.push_str(
            "pub fn core_registry_setter_field_kind_by_property_key(key: u16) -> Option<FieldKind> {\n",
        );
        out.push_str("    match key {\n");
        for (key, property) in &first_properties {
            let Some(kind) = core_registry_setter_field_kind(*property)? else {
                continue;
            };
            out.push_str(&format!(
                "        {key} => Some(FieldKind::{}),\n",
                kind.variant()
            ));
        }
        out.push_str("        _ => None,\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");

        out.push_str(
            "pub fn core_registry_getter_field_kind_by_property_key(key: u16) -> Option<FieldKind> {\n",
        );
        out.push_str("    match key {\n");
        for (key, property) in &first_properties {
            let Some(kind) = core_registry_getter_field_kind(*property)? else {
                continue;
            };
            out.push_str(&format!(
                "        {key} => Some(FieldKind::{}),\n",
                kind.variant()
            ));
        }
        out.push_str("        _ => None,\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");

        Ok(out)
    }

    fn render_hierarchy_property_lookup_table(
        &self,
        defs: &[RuntimeDefinition<'_>],
    ) -> Result<String> {
        let mut index_by_name = BTreeMap::new();
        for (index, def) in defs.iter().enumerate() {
            index_by_name.insert(def.name.as_str(), index);
        }

        let properties_by_definition = defs
            .iter()
            .map(sorted_runtime_properties)
            .collect::<Vec<_>>();
        let mut entries = BTreeMap::<(u16, u16), (usize, usize)>::new();

        for (definition_index, def) in defs.iter().enumerate() {
            let mut owner_indices = vec![definition_index];
            for ancestor in self.ancestor_names(&def.file)? {
                let ancestor_index = *index_by_name
                    .get(ancestor.as_str())
                    .with_context(|| format!("ancestor {ancestor} missing from runtime defs"))?;
                owner_indices.push(ancestor_index);
            }

            for owner_index in owner_indices {
                for (property_index, property) in properties_by_definition[owner_index]
                    .iter()
                    .copied()
                    .enumerate()
                {
                    for property_key in generated_property_keys(property) {
                        entries
                            .entry((def.key, property_key))
                            .or_insert((owner_index, property_index));
                    }
                }
            }
        }

        let mut out = String::new();
        out.push_str("pub fn property_by_key_in_hierarchy(\n");
        out.push_str("    type_key: u16,\n");
        out.push_str("    property_key: u16,\n");
        out.push_str(") -> Option<(&'static str, &'static Property)> {\n");
        out.push_str("    match (type_key, property_key) {\n");
        for ((type_key, property_key), (owner_index, property_index)) in entries {
            out.push_str(&format!(
                "        ({type_key}, {property_key}) => Some(({}, &DEF_{}_PROPERTIES[{}])),\n",
                rust_string(&defs[owner_index].name),
                owner_index,
                property_index
            ));
        }
        out.push_str("        _ => None,\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");

        Ok(out)
    }

    fn render_bitmask_passthrough_lookup_table(
        &self,
        defs: &[RuntimeDefinition<'_>],
    ) -> Result<String> {
        let mut index_by_name = BTreeMap::new();
        for (index, def) in defs.iter().enumerate() {
            index_by_name.insert(def.name.as_str(), index);
        }

        let properties_by_definition = defs
            .iter()
            .map(sorted_runtime_properties)
            .collect::<Vec<_>>();
        let mut entries = BTreeMap::<(u16, u16), RenderedBitmask<'_>>::new();

        for def in defs {
            let mut owner_indices = vec![
                *index_by_name
                    .get(def.name.as_str())
                    .expect("definition index was inserted"),
            ];
            for ancestor in self.ancestor_names(&def.file)? {
                let ancestor_index = *index_by_name
                    .get(ancestor.as_str())
                    .with_context(|| format!("ancestor {ancestor} missing from runtime defs"))?;
                owner_indices.push(ancestor_index);
            }

            for owner_index in owner_indices {
                for property in properties_by_definition[owner_index].iter().copied() {
                    let Some(bitmask) = property.raw.bitmask_passthrough() else {
                        continue;
                    };
                    for property_key in generated_property_keys(property) {
                        entries.entry((def.key, property_key)).or_insert(bitmask);
                    }
                }
            }
        }

        let mut out = String::new();
        out.push_str("pub fn bitmask_passthrough_by_key_in_hierarchy(\n");
        out.push_str("    type_key: u16,\n");
        out.push_str("    property_key: u16,\n");
        out.push_str(") -> Option<BitmaskPassthrough> {\n");
        out.push_str("    match (type_key, property_key) {\n");
        for ((type_key, property_key), bitmask) in entries {
            out.push_str(&format!(
                "        ({type_key}, {property_key}) => Some(BitmaskPassthrough {{ target: {}, bit: {}, width: {} }}),\n",
                rust_string(bitmask.target),
                bitmask.bit,
                bitmask.width,
            ));
        }
        out.push_str("        _ => None,\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");

        Ok(out)
    }

    fn callback_property_keys(&self, defs: &[RuntimeDefinition<'_>]) -> Result<Vec<u16>> {
        let mut keys = BTreeSet::new();
        for def in defs {
            for property in &def.properties {
                let field = FieldKind::from_property(property.raw).with_context(|| {
                    format!(
                        "{}:{} has unsupported runtime type {:?}",
                        def.file,
                        property.name,
                        property.raw.runtime_type()
                    )
                })?;
                if matches!(field, FieldKind::Callback) {
                    keys.insert(property.key_int());
                }
            }
        }
        Ok(keys.into_iter().collect())
    }

    fn render_property(&self, property: &RawPropertyEntry<'_>, def_file: &str) -> Result<String> {
        let key = property
            .raw
            .key
            .as_ref()
            .with_context(|| format!("{def_file}:{} is missing a key", property.name))?;
        let key_int = key
            .int
            .with_context(|| format!("{def_file}:{} is missing a numeric key", property.name))?;
        let key_name = key.string.as_deref().unwrap_or(property.name).to_string();
        let field = FieldKind::from_property(property.raw).with_context(|| {
            format!(
                "{def_file}:{} has unsupported runtime type {:?}",
                property.name,
                property.raw.runtime_type()
            )
        })?;
        let stores_data = field.stores_data();
        let bitmask = property.raw.bitmask_passthrough();
        let passthrough = property.raw.passthrough;
        let deserializes = stores_data && !passthrough && bitmask.is_none();
        let stores_field = deserializes && !property.raw.encoded;

        let mut out = String::new();
        out.push_str("    Property {\n");
        out.push_str(&format!("        name: {},\n", rust_string(property.name)));
        out.push_str(&format!(
            "        key: Key {{ int: {}, name: {} }},\n",
            key_int,
            rust_string(&key_name)
        ));
        out.push_str("        alternates: &[\n");
        for alternate in &key.alternates {
            if let (Some(int), Some(name)) = (alternate.int, alternate.string.as_deref()) {
                out.push_str(&format!(
                    "            Key {{ int: {}, name: {} }},\n",
                    int,
                    rust_string(name)
                ));
            }
        }
        out.push_str("        ],\n");
        out.push_str(&format!(
            "        declared_type: {},\n",
            rust_string(property.raw.declared_type.as_deref().unwrap_or(""))
        ));
        out.push_str(&format!(
            "        runtime_type: FieldKind::{},\n",
            field.variant()
        ));
        out.push_str(&format!(
            "        description: {},\n",
            option_string(property.raw.description.as_deref())
        ));
        out.push_str(&format!(
            "        initial_value: {},\n",
            option_string(property.raw.initial_value.as_deref())
        ));
        out.push_str(&format!(
            "        initial_value_runtime: {},\n",
            option_string(property.raw.initial_value_runtime.as_deref())
        ));
        out.push_str(&format!(
            "        group: {},\n",
            option_string(property.raw.group.as_deref())
        ));
        out.push_str(&format!("        nullable: {},\n", property.raw.nullable));
        out.push_str(&format!(
            "        override_set: {},\n",
            property.raw.override_set
        ));
        out.push_str(&format!(
            "        override_get: {},\n",
            property.raw.override_get
        ));
        out.push_str(&format!("        virtual_: {},\n", property.raw.virtual_));
        out.push_str(&format!(
            "        editor_only: {},\n",
            property.raw.editor_only
        ));
        out.push_str(&format!(
            "        coop: {},\n",
            property.raw.effective_coop()
        ));
        out.push_str(&format!(
            "        with_rive_tools_only: {},\n",
            property.raw.with_rive_tools_only
        ));
        out.push_str(&format!("        stores_data: {stores_data},\n"));
        out.push_str(&format!("        deserializes: {deserializes},\n"));
        out.push_str(&format!("        stores_field: {stores_field},\n"));
        out.push_str(&format!("        encoded: {},\n", property.raw.encoded));
        out.push_str(&format!("        bindable: {},\n", property.raw.bindable));
        out.push_str(&format!("        animates: {},\n", property.raw.animates));
        out.push_str(&format!("        computed: {},\n", property.raw.computed));
        out.push_str(&format!(
            "        journal: {},\n",
            option_bool(property.raw.journal)
        ));
        out.push_str(&format!(
            "        parentable: {},\n",
            option_u64(property.raw.parentable)
        ));
        out.push_str(&format!(
            "        records: {},\n",
            option_bool(property.raw.records)
        ));
        out.push_str(&format!(
            "        exports_to_runtime_conditionally: {},\n",
            property.raw.exports_to_runtime_conditionally
        ));
        out.push_str(&format!(
            "        pure_virtual: {},\n",
            property.raw.pure_virtual
        ));
        out.push_str(&format!("        passthrough: {passthrough},\n"));
        out.push_str("        bitmask_passthrough: ");
        match bitmask {
            Some(bitmask) => {
                out.push_str(&format!(
                    "Some(BitmaskPassthrough {{ target: {}, bit: {}, width: {} }}),\n",
                    rust_string(bitmask.target),
                    bitmask.bit,
                    bitmask.width
                ));
            }
            None => out.push_str("None,\n"),
        }
        out.push_str("    },\n");
        Ok(out)
    }

    fn runtime_parent_file(&self, file: &str) -> Result<Option<String>> {
        let Some(parent_file) = self.defs.get(file).and_then(|def| def.extends.as_deref()) else {
            return Ok(None);
        };
        let parent = self
            .defs
            .get(parent_file)
            .with_context(|| format!("{file} extends missing definition {parent_file}"))?;

        if parent.runtime {
            Ok(Some(parent_file.to_string()))
        } else {
            self.runtime_parent_file(parent_file)
        }
    }

    fn ancestor_names(&self, file: &str) -> Result<Vec<String>> {
        let mut ancestors = Vec::new();
        let mut current = self.runtime_parent_file(file)?;
        while let Some(parent_file) = current {
            let parent = self
                .defs
                .get(&parent_file)
                .with_context(|| format!("missing ancestor {parent_file}"))?;
            ancestors.push(
                parent
                    .name
                    .clone()
                    .with_context(|| format!("{parent_file} is missing a name"))?,
            );
            current = self.runtime_parent_file(&parent_file)?;
        }
        Ok(ancestors)
    }

    fn validate_extends(&self) -> Result<()> {
        for (file, def) in &self.defs {
            if let Some(parent) = &def.extends {
                if !self.defs.contains_key(parent) {
                    bail!("{file} extends missing definition {parent}");
                }
            }
        }
        Ok(())
    }

    fn validate_definition_keys(&self) -> Result<()> {
        let mut seen = BTreeMap::new();
        for (file, def) in &self.defs {
            let Some(key) = def.key.as_ref().and_then(|key| key.int) else {
                continue;
            };
            if let Some(previous) = seen.insert(key, file) {
                bail!("duplicate definition type key {key}: {previous} and {file}");
            }
            if def.runtime && fit_u16(key).is_none() {
                bail!("{file} runtime type key {key} does not fit in uint16_t");
            }
        }
        Ok(())
    }

    fn validate_property_keys(&self) -> Result<()> {
        const MIN_PROPERTY_ID: u64 = 3;

        let mut seen = BTreeMap::<u64, (String, String)>::new();
        for (file, def) in &self.defs {
            for property in def.runtime_properties() {
                let Some(key) = property.raw.key.as_ref().and_then(|key| key.int) else {
                    continue;
                };
                if key < MIN_PROPERTY_ID {
                    bail!(
                        "{file}:{} property key {key} is reserved; ids less than {MIN_PROPERTY_ID} are reserved",
                        property.name
                    );
                }
                if fit_u16(key).is_none() {
                    bail!(
                        "{file}:{} property key {key} does not fit in uint16_t",
                        property.name
                    );
                }

                if let Some((previous_file, previous_name)) =
                    seen.insert(key, (file.clone(), property.name.to_owned()))
                {
                    bail!(
                        "duplicate property key {key}: {file}:{} and {previous_file}:{previous_name}",
                        property.name
                    );
                }
            }
        }

        Ok(())
    }

    fn validate_bitmask_passthroughs(&self) -> Result<()> {
        for (file, def) in self.runtime_definitions() {
            let properties = def.runtime_properties().collect::<Vec<_>>();
            let mut ranges_by_mask = BTreeMap::<&str, Vec<(&str, u16, u16)>>::new();

            for property in &properties {
                let Some(target_name) = property.raw.passthrough_for_bitmask.as_deref() else {
                    continue;
                };
                let bit = property.raw.passthrough_bit.with_context(|| {
                    format!(
                        "{file}:{} passthroughForBitmask requires passthroughBit",
                        property.name
                    )
                })?;
                let width = property.raw.passthrough_bit_width.unwrap_or(1);

                if property.raw.passthrough {
                    bail!(
                        "{file}:{} cannot use passthrough with passthroughForBitmask",
                        property.name
                    );
                }

                let property_type = property.raw.runtime_type().with_context(|| {
                    format!(
                        "{file}:{} passthroughForBitmask requires a runtime type",
                        property.name
                    )
                })?;
                if !matches!(property_type, "bool" | "uint") {
                    bail!(
                        "{file}:{} passthroughForBitmask requires bool or uint, got {property_type}",
                        property.name
                    );
                }
                if property_type == "bool" && property.raw.passthrough_bit_width.unwrap_or(1) != 1 {
                    bail!(
                        "{file}:{} bool passthroughForBitmask must have width 1",
                        property.name
                    );
                }
                if property_type == "uint"
                    && property
                        .raw
                        .passthrough_bit_width
                        .is_none_or(|width| width < 1)
                {
                    bail!(
                        "{file}:{} uint passthroughForBitmask requires passthroughBitWidth >= 1",
                        property.name
                    );
                }

                let target = properties
                    .iter()
                    .find(|candidate| candidate.name == target_name)
                    .with_context(|| {
                        format!(
                            "{file}:{} passthroughForBitmask target {target_name:?} not found",
                            property.name
                        )
                    })?;
                let target_type = target.raw.runtime_type().with_context(|| {
                    format!("{file}:{} bitmask target has no runtime type", target.name)
                })?;
                if target_type != "uint" {
                    bail!(
                        "{file}:{} passthroughForBitmask target {target_name:?} must be uint, got {target_type}",
                        property.name
                    );
                }
                if target.raw.encoded || target.raw.passthrough {
                    bail!(
                        "{file}:{} passthroughForBitmask target {target_name:?} cannot be encoded or passthrough",
                        property.name
                    );
                }

                let end = u16::from(bit) + u16::from(width);
                if width < 1 || end > 32 {
                    bail!(
                        "{file}:{} passthroughBit/passthroughBitWidth must fit in 0..32",
                        property.name
                    );
                }

                let ranges = ranges_by_mask.entry(target_name).or_default();
                let start = u16::from(bit);
                let end = start + u16::from(width);
                for (other_name, other_start, other_end) in ranges.iter().copied() {
                    if start < other_end && other_start < end {
                        bail!(
                            "{file}:{} and {other_name} have overlapping passthrough bit ranges on {target_name}",
                            property.name
                        );
                    }
                }
                ranges.push((property.name, start, end));
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct RuntimeDefinition<'a> {
    file: String,
    name: String,
    variant: String,
    key: u16,
    key_name: String,
    raw_parent_file: Option<String>,
    runtime_parent_file: Option<String>,
    mixins: Vec<String>,
    generic: Option<String>,
    generic_pass_through: Option<String>,
    exports_with_context: bool,
    abstract_: bool,
    properties: Vec<RawPropertyEntry<'a>>,
}

#[derive(Debug, Deserialize)]
struct RawDefinition {
    #[serde(skip)]
    file: String,
    name: Option<String>,
    key: Option<RawKey>,
    extends: Option<String>,
    #[serde(default, rename = "abstract")]
    abstract_: bool,
    #[serde(default = "default_true")]
    runtime: bool,
    #[serde(default)]
    mixins: Vec<String>,
    generic: Option<String>,
    #[serde(rename = "genericPassThrough")]
    generic_pass_through: Option<String>,
    #[serde(default, rename = "exportsWithContext")]
    exports_with_context: bool,
    #[serde(default)]
    properties: BTreeMap<String, RawProperty>,
}

impl RawDefinition {
    fn type_key(&self) -> Option<u16> {
        self.key.as_ref()?.int.and_then(fit_u16)
    }

    fn runtime_properties(&self) -> impl Iterator<Item = RawPropertyEntry<'_>> + '_ {
        self.properties
            .iter()
            .filter(|(_, property)| property.runtime)
            .map(|(name, raw)| RawPropertyEntry { name, raw })
    }
}

#[derive(Debug, Clone, Copy)]
struct RawPropertyEntry<'a> {
    name: &'a str,
    raw: &'a RawProperty,
}

impl RawPropertyEntry<'_> {
    fn key_int(self) -> u16 {
        self.raw
            .key
            .as_ref()
            .and_then(|key| key.int)
            .and_then(fit_u16)
            .unwrap_or(0)
    }
}

fn sorted_runtime_properties<'a>(def: &RuntimeDefinition<'a>) -> Vec<RawPropertyEntry<'a>> {
    let mut properties = def.properties.clone();
    properties.sort_by_key(|property| property.key_int());
    properties
}

fn generated_property_keys(property: RawPropertyEntry<'_>) -> Vec<u16> {
    let mut keys = Vec::new();
    keys.push(property.key_int());

    if let Some(raw_key) = property.raw.key.as_ref() {
        for alternate in &raw_key.alternates {
            if alternate.string.is_none() {
                continue;
            }
            if let Some(int) = alternate.int.and_then(fit_u16) {
                keys.push(int);
            }
        }
    }

    keys
}

fn first_properties_by_key<'a>(
    defs: &'a [RuntimeDefinition<'a>],
) -> BTreeMap<u16, RawPropertyEntry<'a>> {
    let mut properties_by_key = BTreeMap::new();
    for def in defs {
        for property in sorted_runtime_properties(def) {
            for key in generated_property_keys(property) {
                properties_by_key.entry(key).or_insert(property);
            }
        }
    }
    properties_by_key
}

fn field_kind_for_entry(property: RawPropertyEntry<'_>) -> Result<FieldKind> {
    FieldKind::from_property(property.raw).with_context(|| {
        format!(
            "{} has unsupported runtime type {:?}",
            property.name,
            property.raw.runtime_type()
        )
    })
}

fn core_registry_field_kind_variant(
    property: RawPropertyEntry<'_>,
) -> Result<Option<&'static str>> {
    if property.raw.bitmask_passthrough().is_some() {
        return Ok(None);
    }

    Ok(match field_kind_for_entry(property)? {
        FieldKind::Uint => Some("Uint"),
        FieldKind::String | FieldKind::Bytes => Some("StringOrBytes"),
        FieldKind::Double => Some("Double"),
        FieldKind::Color => Some("Color"),
        FieldKind::Bool => Some("Bool"),
        FieldKind::Callback => None,
    })
}

fn core_registry_setter_field_kind(property: RawPropertyEntry<'_>) -> Result<Option<FieldKind>> {
    let field = field_kind_for_entry(property)?;
    if property.raw.encoded || matches!(field, FieldKind::Bytes) {
        Ok(None)
    } else {
        Ok(Some(field))
    }
}

fn core_registry_getter_field_kind(property: RawPropertyEntry<'_>) -> Result<Option<FieldKind>> {
    let field = field_kind_for_entry(property)?;
    if property.raw.encoded
        || matches!(field, FieldKind::Bytes | FieldKind::Callback)
        || (matches!(field, FieldKind::Bool) && property.raw.bitmask_passthrough().is_some())
    {
        Ok(None)
    } else {
        Ok(Some(field))
    }
}

#[derive(Debug, Deserialize)]
struct RawProperty {
    #[serde(rename = "type")]
    declared_type: Option<String>,
    #[serde(rename = "typeRuntime")]
    type_runtime: Option<String>,
    key: Option<RawKey>,
    description: Option<String>,
    #[serde(rename = "initialValue")]
    initial_value: Option<String>,
    #[serde(rename = "initialValueRuntime")]
    initial_value_runtime: Option<String>,
    group: Option<String>,
    #[serde(default)]
    nullable: bool,
    #[serde(default, rename = "overrideSet")]
    override_set: bool,
    #[serde(default, rename = "overrideGet")]
    override_get: bool,
    #[serde(default, rename = "virtual")]
    virtual_: bool,
    #[serde(default, rename = "editorOnly")]
    editor_only: bool,
    #[serde(default = "default_true")]
    runtime: bool,
    #[serde(default)]
    encoded: bool,
    coop: Option<bool>,
    #[serde(default, rename = "withRiveToolsOnly")]
    with_rive_tools_only: bool,
    #[serde(default)]
    bindable: bool,
    #[serde(default)]
    animates: bool,
    #[serde(default)]
    computed: bool,
    journal: Option<bool>,
    parentable: Option<u64>,
    records: Option<bool>,
    #[serde(default, rename = "exportsToRuntimeConditionally")]
    exports_to_runtime_conditionally: bool,
    #[serde(default, rename = "pureVirtual")]
    pure_virtual: bool,
    #[serde(default)]
    passthrough: bool,
    #[serde(rename = "passthroughForBitmask")]
    passthrough_for_bitmask: Option<String>,
    #[serde(rename = "passthroughBit")]
    passthrough_bit: Option<u8>,
    #[serde(rename = "passthroughBitWidth")]
    passthrough_bit_width: Option<u8>,
}

impl RawProperty {
    fn runtime_type(&self) -> Option<&str> {
        self.type_runtime
            .as_deref()
            .or(self.declared_type.as_deref())
    }

    fn bitmask_passthrough(&self) -> Option<RenderedBitmask<'_>> {
        Some(RenderedBitmask {
            target: self.passthrough_for_bitmask.as_deref()?,
            bit: self.passthrough_bit?,
            width: self.passthrough_bit_width.unwrap_or(1),
        })
    }

    fn effective_coop(&self) -> bool {
        self.coop.unwrap_or(!self.editor_only)
    }
}

#[derive(Debug, Deserialize)]
struct RawKey {
    int: Option<u64>,
    string: Option<String>,
    #[serde(default)]
    alternates: Vec<RawKey>,
}

#[derive(Debug, Clone, Copy)]
struct RenderedBitmask<'a> {
    target: &'a str,
    bit: u8,
    width: u8,
}

#[derive(Debug, Clone, Copy)]
enum FieldKind {
    Bool,
    Bytes,
    Callback,
    Color,
    Double,
    String,
    Uint,
}

impl FieldKind {
    fn from_property(property: &RawProperty) -> Option<Self> {
        match property.runtime_type()? {
            "bool" => Some(Self::Bool),
            "Bytes" => Some(Self::Bytes),
            "callback" => Some(Self::Callback),
            "Color" => Some(Self::Color),
            "double" => Some(Self::Double),
            "String" => Some(Self::String),
            "uint" => Some(Self::Uint),
            _ => None,
        }
    }

    fn stores_data(self) -> bool {
        !matches!(self, Self::Callback)
    }

    fn variant(self) -> &'static str {
        match self {
            Self::Bool => "Bool",
            Self::Bytes => "Bytes",
            Self::Callback => "Callback",
            Self::Color => "Color",
            Self::Double => "Double",
            Self::String => "String",
            Self::Uint => "Uint",
        }
    }
}

fn collect_json_files(base: &Path, current: &Path, out: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(current).with_context(|| format!("reading {}", current.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(base, &path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "json") {
            let relative = path
                .strip_prefix(base)
                .with_context(|| format!("stripping prefix {}", base.display()))?
                .to_string_lossy()
                .replace('\\', "/");
            out.push(relative);
        }
    }
    out.sort();
    Ok(())
}

fn rust_string(value: &str) -> String {
    let escaped = value
        .chars()
        .flat_map(char::escape_default)
        .collect::<String>();
    format!("\"{escaped}\"")
}

fn option_string(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("Some({})", rust_string(value)),
        None => "None".to_string(),
    }
}

fn option_bool(value: Option<bool>) -> String {
    match value {
        Some(value) => format!("Some({value})"),
        None => "None".to_string(),
    }
}

fn option_u64(value: Option<u64>) -> String {
    match value {
        Some(value) => format!("Some({value})"),
        None => "None".to_string(),
    }
}

fn string_slice(values: &[String]) -> String {
    if values.is_empty() {
        "&[]".to_string()
    } else {
        let values = values
            .iter()
            .map(|value| rust_string(value))
            .collect::<Vec<_>>()
            .join(", ");
        format!("&[{values}]")
    }
}

fn rust_variant(name: &str) -> Result<String> {
    let mut variant = String::new();
    let mut uppercase_next = true;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            if variant.is_empty() && ch.is_ascii_digit() {
                variant.push('_');
            }
            if uppercase_next {
                variant.push(ch.to_ascii_uppercase());
                uppercase_next = false;
            } else {
                variant.push(ch);
            }
        } else {
            uppercase_next = true;
        }
    }

    if variant.is_empty() {
        bail!("cannot make Rust variant from definition name {name:?}");
    }

    Ok(variant)
}

fn validate_variants(defs: &[RuntimeDefinition<'_>]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for def in defs {
        if !seen.insert(def.variant.as_str()) {
            bail!("duplicate generated Rust object variant {}", def.variant);
        }
    }
    Ok(())
}

fn fit_u16(value: u64) -> Option<u16> {
    u16::try_from(value).ok()
}

fn default_true() -> bool {
    true
}
