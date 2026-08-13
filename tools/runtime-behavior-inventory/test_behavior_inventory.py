#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import json
import pathlib
import subprocess
import tempfile
import unicodedata
import unittest
from unittest import mock

TOOL = pathlib.Path(__file__).with_name("behavior_inventory.py")
SPEC = importlib.util.spec_from_file_location("behavior_inventory", TOOL)
assert SPEC and SPEC.loader
behavior_inventory = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(behavior_inventory)


class CppDiscoveryTests(unittest.TestCase):
    def test_continued_macro_does_not_hide_following_function(self) -> None:
        source = r"""
        #define WRAPPED(value) \
            (static_cast<unsigned>(value))
        static float halfToFloat(uint16_t value) { return value; }
        """
        members = behavior_inventory.cpp_members("src/float.cpp", source)
        self.assertEqual(["halfToFloat"], [member["name"] for member in members])

    def test_conditional_continued_macros_are_not_members_or_boundaries(self) -> None:
        source = r"""
        #if A
        #define UNREACHABLE() \
            assert(false); \
            builtin_unreachable()
        #else
        #define UNREACHABLE() \
            do \
            { \
                assert(false); \
            } while (0)
        #endif
        void shipped() { production(); }
        """
        members = behavior_inventory.cpp_members("include/rive/control.hpp", source)
        self.assertEqual(["shipped"], [member["name"] for member in members])

    def test_uppercase_constructor_is_not_discarded_as_a_macro(self) -> None:
        source = "class AABB { AABB() : minX(0), minY(0) {} };\n"
        members = behavior_inventory.cpp_members("include/rive/aabb.hpp", source)
        self.assertEqual(["AABB::AABB"], [member["name"] for member in members])

    def test_annotation_macro_does_not_replace_uppercase_constructor(self) -> None:
        source = """
        #define API()
        class AABB {
            API()
            AABB() : x(0) {}
            int x;
        };
        """
        members = behavior_inventory.cpp_members("include/rive/aabb.hpp", source)
        self.assertEqual(["AABB::AABB"], [member["name"] for member in members])

        for annotations in ("API()", "API() EXPORT()"):
            with self.subTest(annotations=annotations):
                same_line = (
                    "#define API()\n#define EXPORT()\n"
                    f"class AABB {{ {annotations} AABB() : x(0) {{}} int x; }};\n"
                )
                members = behavior_inventory.cpp_members(
                    "include/rive/aabb.hpp", same_line
                )
                self.assertEqual(["AABB::AABB"], [member["name"] for member in members])

        for arguments in (
            "std::array<int, 1>{}",
            "Foo{1}",
            "[] { return 1; }",
        ):
            with self.subTest(arguments=arguments):
                balanced = (
                    "#define API(...)\n"
                    f"class AABB {{ API({arguments}) AABB() : x(0) {{}} int x; }};\n"
                )
                members = behavior_inventory.cpp_members(
                    "include/rive/aabb.hpp", balanced
                )
                self.assertEqual(["AABB::AABB"], [member["name"] for member in members])

    def test_loop_is_a_valid_cpp_member_name(self) -> None:
        source = "class Animation { Loop loop() const { return Loop::oneShot; } };\n"
        members = behavior_inventory.cpp_members("include/rive/animation.hpp", source)
        self.assertEqual(["Animation::loop"], [member["name"] for member in members])

    def test_cpp_raw_string_payload_does_not_hide_following_member(self) -> None:
        source = r"""
        const char* fixture = R"tag("; } void fake() {)tag";
        void shipped() { publishDirt(); }
        """
        members = behavior_inventory.cpp_members("src/shipped.cpp", source)
        self.assertEqual(["shipped"], [member["name"] for member in members])

    def test_cpp_block_comment_inner_marker_does_not_hide_shipped_member(self) -> None:
        source = """
        /* document the /* token */
        void shipped() { publishDirt(); }
        """
        members = behavior_inventory.cpp_members("src/shipped.cpp", source)
        self.assertEqual(1, len(members))
        self.assertIn(":shipped@", members[0]["id"])

    def test_comment_prefixed_continued_directive_is_masked(self) -> None:
        source = r"""
        /* lead */ #define WRAPPED(value) \
            before(); \
            fake(value)
        static float real(int value) { return value; }
        """
        members = behavior_inventory.cpp_members("src/real.cpp", source)
        self.assertEqual(["real"], [member["name"] for member in members])

    def test_macro_only_behavioral_header_has_explicit_macro_evidence(self) -> None:
        source = r"""
        #if defined(__GNUC__)
        #define RIVE_UNREACHABLE \
            assert(false); \
            __builtin_unreachable()
        #else
        #define RIVE_UNREACHABLE() \
            do { assert(false); } while (0)
        #endif
        """
        macros = behavior_inventory.cpp_behavioral_macros(source)
        self.assertEqual(2, len(macros))
        self.assertEqual({"RIVE_UNREACHABLE"}, {macro["name"] for macro in macros})
        self.assertEqual(
            "behavioral-header",
            behavior_inventory.cpp_file_classification(
                "include/rive/rive_types.hpp", [], macros
            ),
        )

    def test_behavioral_macros_cover_conditions_calls_and_mutations(self) -> None:
        source = r"""
        /* lead */ #if FEATURE_A
        #define ACTION(x)do { use_a(x); } while (0)
        /* lead */ #else
        #define ACTION(x) callback(x)
        /* lead */ #endif
        #define ASSIGN(target, value) ((target) = (value))
        #define INCREMENT(target) ((target) += 1)
        #define ALLOCATE(value) new Widget(value)
        """
        macros = behavior_inventory.cpp_behavioral_macros(source)
        self.assertEqual(5, len(macros))
        action_ids = [record["id"] for record in macros if record["name"] == "ACTION"]
        self.assertEqual(2, len(set(action_ids)))
        self.assertEqual(
            {"ACTION", "ALLOCATE", "ASSIGN", "INCREMENT"},
            {record["name"] for record in macros},
        )
        self.assertEqual(
            "behavioral-header",
            behavior_inventory.cpp_file_classification(
                "include/rive/macros.hpp", [], macros
            ),
        )

    def test_logical_directives_preserve_condition_and_token_splices(self) -> None:
        source = r"""
        #if defined(A) && \
            defined(B)
        #define ACTION(x) callback_a(x)
        #endif
        #if defined(A) && \
            !defined(B)
        #define ACTION(x) callback_b(x)
        #endif
        #define IS_OUT\
(x) ((x) > 0)
        """
        macros = behavior_inventory.cpp_behavioral_macros(source)
        self.assertEqual(3, len(macros))
        actions = [record for record in macros if record["name"] == "ACTION"]
        self.assertEqual(2, len({record["id"] for record in actions}))
        self.assertEqual({"ACTION", "IS_OUT"}, {record["name"] for record in macros})

        branches = """
        #if FIRST
        #elif COMMON
        #define BRANCH(x) branch_a(x)
        #endif
        #if SECOND
        #elif COMMON
        #define BRANCH(x) branch_b(x)
        #endif
        """
        macros = behavior_inventory.cpp_behavioral_macros(branches)
        self.assertEqual(2, len({record["id"] for record in macros}))

        literal_spaces = r"""
        #if CHECK("a  b")
        #define LITERAL(x) literal_a(x)
        #endif
        #if CHECK("a b")
        #define LITERAL(x) literal_b(x)
        #endif
        """
        macros = behavior_inventory.cpp_behavioral_macros(literal_spaces)
        self.assertEqual(2, len({record["id"] for record in macros}))

        raw_directive = '#define RAW(x) consume(R"tag(first\nsecond)tag")\n'
        before = behavior_inventory.cpp_behavioral_macros(raw_directive)[0]
        after = behavior_inventory.cpp_behavioral_macros(
            raw_directive.replace("second", "changed")
        )[0]
        self.assertEqual(2, before["end_line"])
        self.assertNotEqual(before["content_sha256"], after["content_sha256"])

        raw_conditions = (
            '#if CHECK(R"tag(a \\\nb)tag")\n'
            "#define RAW_CONDITION(x) raw_a(x)\n"
            "#endif\n"
            '#if CHECK(R"tag(a b)tag")\n'
            "#define RAW_CONDITION(x) raw_b(x)\n"
            "#endif\n"
        )
        macros = behavior_inventory.cpp_behavioral_macros(raw_conditions)
        self.assertEqual(2, len({record["id"] for record in macros}))

        comment_raw = (
            '#define FIRST(x) callback(x) // docs R"tag(not raw\n'
            "#define SECOND(x) callback2(x)\n"
        )
        macros = behavior_inventory.cpp_behavioral_macros(comment_raw)
        self.assertEqual(["FIRST", "SECOND"], [record["name"] for record in macros])

    def test_cpp_splices_apply_before_comment_recognition(self) -> None:
        swallowed = "// swallowed \\\n#define HIDDEN(x) callback(x)\n"
        self.assertEqual([], behavior_inventory.cpp_behavioral_macros(swallowed))

        created_comment = "/\\\n* lead */ #define LIVE(x) callback(x)\n"
        macros = behavior_inventory.cpp_behavioral_macros(created_comment)
        self.assertEqual(["LIVE"], [record["name"] for record in macros])

        closed_comment = "/* lead *\\\n/ #define ALSO_LIVE(x) callback(x)\n"
        macros = behavior_inventory.cpp_behavioral_macros(closed_comment)
        self.assertEqual(["ALSO_LIVE"], [record["name"] for record in macros])

    def test_condition_literals_and_spliced_raw_strings_remain_structural(self) -> None:
        source = r"""
        #if __has_include("vector")
        #define ACTION(x) vector_action(x)
        #endif
        #if __has_include("definitely_missing_codex_header")
        #define ACTION(x) fallback_action(x)
        #endif
        """
        actions = behavior_inventory.cpp_behavioral_macros(source)
        self.assertEqual(2, len({record["id"] for record in actions}))

        raw = 'const char* text = R\\\n"tag(" } int fake() { return 0; })tag";\nint live(){return 1;}\n'
        members = behavior_inventory.cpp_members("src/raw.cpp", raw)
        self.assertEqual(["live"], [record["name"] for record in members])

        raw_payload_splice = (
            'const char* text = R"tag(payload )ta\\\n'
            'g"; int fake(){return 0;} )tag";\nint live(){return 1;}\n'
        )
        members = behavior_inventory.cpp_members("src/raw.cpp", raw_payload_splice)
        self.assertEqual(["live"], [record["name"] for record in members])

        raw_directive_payload = (
            'const char* text = R"tag(payload\n#if GHOST\n)tag";\n'
            "int live(){return 1;}\n"
        )
        members = behavior_inventory.cpp_members("src/raw.cpp", raw_directive_payload)
        self.assertEqual(["live"], [record["name"] for record in members])
        plain = behavior_inventory.cpp_members("src/raw.cpp", "int live(){return 1;}\n")
        self.assertEqual(plain[0]["id"], members[0]["id"])

    def test_cpp_member_identity_uses_phase_two_spliced_tokens(self) -> None:
        source = (
            "namespace na\\\nme {\n"
            "class Th\\\ning {\n"
            "int f\\\noo(){return 1;}\n"
            "};\n}\n"
        )
        members = behavior_inventory.cpp_members("src/spliced.cpp", source)
        self.assertEqual(["name::Thing::foo"], [record["name"] for record in members])

        for comment in ('// docs R"tag(not raw\n', '/* docs R"tag(not raw */\n'):
            with self.subTest(comment=comment):
                members = behavior_inventory.cpp_members(
                    "src/spliced.cpp", comment + "int fo\\\no(){return 1;}\n"
                )
                self.assertEqual(["foo"], [record["name"] for record in members])

        member = behavior_inventory.cpp_members(
            "src/spliced.cpp", "int foo() \\\n{ return 1; }\n"
        )[0]
        self.assertEqual((2, 2), (member["start_line"], member["end_line"]))

    def test_object_like_control_and_increment_macros_are_behavioral(self) -> None:
        source = """
        #define PRE_INCREMENT ++cursor
        #define POST_INCREMENT cursor++
        #define STOP break
        #define NEXT continue
        #define COMPLETE co_return
        #define YIELD co_yield value
        #define BAIL goto cleanup
        #define AWAIT co_await task
        #define SELECT switch
        #define BRANCH case 1:
        #define FALLBACK default:
        #define OTHERWISE else
        #define HANDLE try
        """
        macros = behavior_inventory.cpp_behavioral_macros(source)
        self.assertEqual(13, len(macros))

    def test_header_only_behavior_and_declaration_only_headers_are_distinct(
        self,
    ) -> None:
        behavioral = """
        class Shape {
        public:
            void opacity(float value) {
                if (m_opacity == value) return;
                m_opacity = value;
                addDirt(ComponentDirt::Paint);
            }
        };
        """
        declaration = "class Shape { public: void opacity(float value); };\n"

        members = behavior_inventory.cpp_members("include/rive/shape.hpp", behavioral)
        self.assertEqual(1, len(members))
        self.assertIn(":Shape::opacity@", members[0]["id"])
        self.assertIn("setter", members[0]["behavior_kinds"])
        self.assertIn("mutation-guard", members[0]["behavior_kinds"])
        self.assertIn("dirt-publication", members[0]["behavior_kinds"])
        self.assertEqual(
            "behavioral-header",
            behavior_inventory.cpp_file_classification(
                "include/rive/shape.hpp", members
            ),
        )
        self.assertEqual(
            "declaration-only",
            behavior_inventory.cpp_file_classification(
                "include/rive/shape_decl.hpp",
                behavior_inventory.cpp_members(
                    "include/rive/shape_decl.hpp", declaration
                ),
            ),
        )

    def test_generated_header_is_explicit_even_when_it_has_behavior(self) -> None:
        source = "class A { public: int value() const { return 1; } };\n"
        members = behavior_inventory.cpp_members(
            "include/rive/generated/a_base.hpp", source
        )
        self.assertEqual(1, len(members))
        self.assertEqual(
            "generated",
            behavior_inventory.cpp_file_classification(
                "include/rive/generated/a_base.hpp", members
            ),
        )

    def test_behavior_tags_cover_callbacks_lifecycle_dependencies_and_edges(
        self,
    ) -> None:
        source = """
        void Node::onRemoved() override {
            removeDependent(m_owner);
            auto clone = std::make_unique<Node>(*this);
            for (auto item : m_items) { item->dispose(); }
            m_value = static_cast<uint32_t>(std::clamp(value, 0, 255));
            notifyChanged();
        }
        """
        member = behavior_inventory.cpp_members("src/node.cpp", source)[0]
        self.assertEqual(
            {
                "callback",
                "dependency-operation",
                "lifecycle",
                "ordering-loop",
                "ownership",
                "scalar-edge",
                "setter",
                "virtual-override",
            },
            set(member["behavior_kinds"]),
        )

    def test_braced_constructor_initializer_does_not_replace_body(self) -> None:
        source = """
        AudioSourceDecoder::AudioSourceDecoder() : m_decoder({}) {
            publishDirt();
            initializeDecoder();
        }
        """
        members = behavior_inventory.cpp_members("src/audio/source.cpp", source)
        self.assertEqual(1, len(members))
        self.assertEqual(3, members[0]["end_line"] - members[0]["start_line"])
        self.assertIn("dirt-publication", members[0]["behavior_kinds"])
        changed = source.replace("m_decoder({})", "m_decoder({1})")
        changed_member = behavior_inventory.cpp_members(
            "src/audio/source.cpp", changed
        )[0]
        self.assertNotEqual(
            members[0]["content_sha256"], changed_member["content_sha256"]
        )

    def test_parenthesized_constructor_initializers_are_member_behavior(self) -> None:
        source = """
        AudioSound::AudioSound() : m_isDisposed(false), m_owner(std::move(owner))
        {}
        """
        member = behavior_inventory.cpp_members("src/audio/sound.cpp", source)[0]
        changed = source.replace("m_isDisposed(false)", "m_isDisposed(true)")
        changed_member = behavior_inventory.cpp_members("src/audio/sound.cpp", changed)[
            0
        ]
        self.assertEqual(1, member["end_line"] - member["start_line"])
        self.assertEqual(member["id"], changed_member["id"])
        self.assertNotEqual(member["content_sha256"], changed_member["content_sha256"])

    def test_braced_default_argument_does_not_replace_inline_constructor(self) -> None:
        source = """
        class ScriptedPathCommand {
        public:
            ScriptedPathCommand(
                std::string type,
                std::vector<Vec2D> points = {}) :
                m_type(type), m_points(std::move(points)) {}
        };
        """
        members = behavior_inventory.cpp_members(
            "include/rive/lua/rive_lua_libs.hpp", source
        )
        self.assertEqual(1, len(members))
        self.assertEqual("ScriptedPathCommand::ScriptedPathCommand", members[0]["name"])
        self.assertNotIn("::m_type@", members[0]["id"])

    def test_default_argument_lambda_does_not_replace_outer_function(self) -> None:
        for lambda_head in ("[]", "[]()"):
            with self.subTest(lambda_head=lambda_head):
                source = (
                    "void shipped(std::function<void()> cb = "
                    f"{lambda_head} {{ fixture(); }}) {{ production(); }}"
                )
                members = behavior_inventory.cpp_members("src/shipped.cpp", source)
                self.assertEqual(["shipped"], [member["name"] for member in members])
                self.assertIn("production", source)

    def test_field_initializer_lambda_is_not_a_false_member(self) -> None:
        source = """
        class Node {
            std::function<void(Vec2D&)> map = [](Vec2D& value) {};
            void shipped() { production(); }
        };
        """
        members = behavior_inventory.cpp_members("include/rive/node.hpp", source)
        self.assertEqual(["Node::shipped"], [member["name"] for member in members])

    def test_array_return_declarator_is_not_a_lambda_capture(self) -> None:
        source = """
        auto reference_to_array() -> int(&)[3] {
            static int values[3];
            return values;
        }
        auto pointer_to_array() -> int(*)[3] { return nullptr; }
        """
        members = behavior_inventory.cpp_members("src/arrays.cpp", source)
        self.assertEqual(
            ["reference_to_array", "pointer_to_array"],
            [member["name"] for member in members],
        )

    def test_header_override_declaration_marks_out_of_line_definition(self) -> None:
        header = "class Drawable { void draw(Renderer*) override; };"
        source = "void Drawable::draw(Renderer* renderer) { renderer->draw(); }"
        declarations = behavior_inventory.cpp_virtual_declarations(header)
        member = behavior_inventory.cpp_members("src/drawable.cpp", source)[0]
        if behavior_inventory.cpp_virtual_key(member["name"]) in declarations:
            member["behavior_kinds"] = sorted(
                set(member["behavior_kinds"]) | {"virtual-override"}
            )
        self.assertIn("virtual-override", member["behavior_kinds"])

    def test_cpp_signature_identity_is_stable_when_overloads_reorder(self) -> None:
        first = "void Node::set(uint8_t value) {}\nvoid Node::set(uint16_t value) {}\n"
        second = "void Node::set(uint16_t value) {}\nvoid Node::set(uint8_t value) {}\n"
        self.assertEqual(
            {
                item["id"]
                for item in behavior_inventory.cpp_members("src/node.cpp", first)
            },
            {
                item["id"]
                for item in behavior_inventory.cpp_members("src/node.cpp", second)
            },
        )

    def test_cpp_namespace_contexts_distinguish_free_functions(self) -> None:
        source = """
        namespace audio { void init() {} }
        namespace renderer { void init() {} }
        """
        members = behavior_inventory.cpp_members("src/runtime.cpp", source)
        self.assertEqual(2, len(members))
        self.assertEqual(2, len({member["id"] for member in members}))
        self.assertEqual(
            {"audio::init", "renderer::init"},
            {member["name"] for member in members},
        )

    def test_cpp_type_macro_does_not_consume_inline_member_bodies(self) -> None:
        source = """
        class RenderBuffer : public RefCnt<RenderBuffer>,
                             public ENABLE_LITE_RTTI(RenderBuffer)
        {
        public:
            bool checkAndResetDirty() {
                if (!m_dirty) return false;
                m_dirty = false;
                return true;
            }
        };
        """
        members = behavior_inventory.cpp_members("include/rive/renderer.hpp", source)
        self.assertEqual(
            ["RenderBuffer::checkAndResetDirty"], [m["name"] for m in members]
        )

    def test_cpp_elaborated_return_type_is_still_a_member(self) -> None:
        source = """
        class SemanticNode {
            class SemanticData* semanticData() const { return m_semanticData; }
        };
        """
        members = behavior_inventory.cpp_members(
            "include/rive/semantic/semantic_node.hpp", source
        )
        self.assertEqual(["SemanticNode::semanticData"], [m["name"] for m in members])

    def test_cpp_aligned_class_scope_does_not_hide_operators(self) -> None:
        source = """
        class alignas(32) VectorXform {
            VectorXform& operator=(const Matrix& value) { return *this; }
            float operator[](size_t index) const { return values[index]; }
            float operator()(float value) const { return value; }
            operator gvec<float, 4>() const { return {}; }
        };
        """
        members = behavior_inventory.cpp_members("include/rive/wangs.hpp", source)
        self.assertEqual(
            [
                "VectorXform::operator=",
                "VectorXform::operator[]",
                "VectorXform::operator()",
                "VectorXform::operator gvec<float, 4>",
            ],
            [m["name"] for m in members],
        )

    def test_cpp_template_class_context_stops_before_base_list(self) -> None:
        source = """
        template <typename T, int N>
        struct Buffer<T, N> : public BufferStorage<T, N> {
            T& operator[](int index) { return values[index]; }
        };
        """
        members = behavior_inventory.cpp_members("include/rive/buffer.hpp", source)
        self.assertEqual(["Buffer<T, N>::operator[]"], [m["name"] for m in members])

    def test_cpp_explicit_specialization_retains_class_context(self) -> None:
        source = """
        namespace std {
        template <> struct hash<rive::Vec2D> {
            size_t operator()(const rive::Vec2D& value) const { return 1; }
        };
        }
        """
        members = behavior_inventory.cpp_members("include/rive/vec2d.hpp", source)
        self.assertEqual(
            ["std::hash<rive::Vec2D>::operator()"], [m["name"] for m in members]
        )

    def test_cpp_partial_specialization_retains_nested_class_context(self) -> None:
        source = """
        namespace std {
        template <typename T> struct hash<rive::rcp<T>> {
            size_t operator()(const rive::rcp<T>& value) const { return 1; }
        };
        }
        """
        members = behavior_inventory.cpp_members("include/rive/refcnt.hpp", source)
        self.assertEqual(
            ["std::hash<rive::rcp<T>>::operator()"], [m["name"] for m in members]
        )

    def test_cpp_preprocessor_macro_does_not_consume_namespace_body(self) -> None:
        source = """
        #define DISABLE_WARNING() _Pragma("clang diagnostic push")
        DISABLE_WARNING()
        namespace rive
        {
        bool any() { return true; }
        }
        """
        members = behavior_inventory.cpp_members("include/rive/simd.hpp", source)
        self.assertEqual(["rive::any"], [member["name"] for member in members])

    def test_cpp_standalone_macro_does_not_replace_following_function_name(
        self,
    ) -> None:
        source = """
        PUSH_DISABLE_WARNING()
        template <typename T> T join(T a, T b) { return a + b; }
        """
        members = behavior_inventory.cpp_members("include/rive/simd.hpp", source)
        self.assertEqual(["join"], [member["name"] for member in members])


class RustDiscoveryTests(unittest.TestCase):
    def test_raw_identifier_function_has_normalized_item_identity(self) -> None:
        keyword_items = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", "fn r#match() { production(); }\n"
        )
        self.assertEqual(["match"], [item["name"] for item in keyword_items])
        self.assertIn("::match@", keyword_items[0]["id"])
        ordinary = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", "fn shipped() { production(); }\n"
        )[0]
        raw = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", "fn r#shipped() { production(); }\n"
        )[0]
        self.assertEqual(ordinary["id"], raw["id"])
        self.assertEqual(ordinary["signature_sha256"], raw["signature_sha256"])
        cfg_raw_text = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs",
            '#[cfg(custom = "fn r#variant")]\nfn shipped() { production(); }\n',
        )[0]
        cfg_plain_text = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs",
            '#[cfg(custom = "fn variant")]\nfn shipped() { production(); }\n',
        )[0]
        self.assertNotEqual(cfg_raw_text["id"], cfg_plain_text["id"])

    def test_long_raw_string_payload_does_not_hide_following_item(self) -> None:
        hashes = "#" * 255
        source = (
            f'const FIXTURE: &str = r{hashes}""; }} fn fake() {{"{hashes};\n'
            "fn shipped() { production(); }\n"
        )
        items = behavior_inventory.rust_items("crates/demo/src/lib.rs", source)
        self.assertEqual(["shipped"], [item["name"] for item in items])

    def test_extern_function_identity_retains_cfg_attributes_and_abi(self) -> None:
        source = """
        #[cfg(target_os = "macos")]
        #[unsafe(no_mangle)]
        pub unsafe extern /* outer /* ABI */ tail */ "C" fn exported() { macos_behavior(); }
        #[cfg(target_os = "linux")]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn exported() { linux_behavior(); }
        #[cfg(test)]
        pub unsafe extern "C" fn test_export() { test_only(); }
        pub extern r#"C"# fn raw_export() { raw_behavior(); }
        """
        items = behavior_inventory.rust_items("crates/demo/src/lib.rs", source)
        self.assertEqual(
            ["exported", "exported", "raw_export"],
            [item["name"] for item in items],
        )
        self.assertEqual(3, len({item["id"] for item in items}))
        abi_changed = source.replace('"C" fn exported', '"C-unwind" fn exported', 1)
        changed_items = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", abi_changed
        )
        self.assertNotEqual(items[0]["id"], changed_items[0]["id"])
        raw_abi_changed = source.replace('r#"C"#', 'r##"Rust"##')
        raw_changed_items = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", raw_abi_changed
        )
        self.assertNotEqual(items[2]["id"], raw_changed_items[2]["id"])
        continued_c = 'pub unsafe extern "C\\\n" fn continued() { c_behavior(); }'
        continued_unwind = (
            'pub unsafe extern "C-\\\nunwind" fn continued() { unwind_behavior(); }'
        )
        continued_c_item = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", continued_c
        )[0]
        continued_unwind_item = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", continued_unwind
        )[0]
        self.assertNotEqual(continued_c_item["id"], continued_unwind_item["id"])

    def test_extern_text_in_comments_and_docs_does_not_change_identity(self) -> None:
        before = """
        #[doc = r#"uses extern \"C\" ABI"#]
        fn shipped(/* extern "C" */ value: u8) { behavior(value); }
        """
        after = before.replace('extern \\"C\\"', 'extern \\"Rust\\"').replace(
            'extern "C"', 'extern "Rust"'
        )
        before_item = behavior_inventory.rust_items("crates/demo/src/lib.rs", before)[0]
        after_item = behavior_inventory.rust_items("crates/demo/src/lib.rs", after)[0]
        self.assertEqual(before_item["id"], after_item["id"])
        self.assertEqual(
            before_item["signature_sha256"], after_item["signature_sha256"]
        )

    def test_cfg_text_in_attribute_payload_does_not_hide_shipped_item(self) -> None:
        before = """
        #[doc = "example: #[cfg(test)]"]
        fn shipped() { production_one(); }
        """
        after = before.replace("production_one", "production_two")
        before_items = behavior_inventory.rust_items("crates/demo/src/lib.rs", before)
        after_items = behavior_inventory.rust_items("crates/demo/src/lib.rs", after)
        self.assertEqual(["shipped"], [item["name"] for item in before_items])
        self.assertEqual(before_items[0]["id"], after_items[0]["id"])
        self.assertNotEqual(
            before_items[0]["content_sha256"], after_items[0]["content_sha256"]
        )
        self.assertNotEqual(
            behavior_inventory.rust_shipped_source(before),
            behavior_inventory.rust_shipped_source(after),
        )

    def test_cfg_attr_applied_cfg_test_is_not_shipped(self) -> None:
        before = """
        #[cfg_attr(not(test), cfg(test))]
        fn only_test() { fixture_one(); }
        fn shipped() { production(); }
        """
        after = before.replace("fixture_one", "fixture_two")
        self.assertEqual(
            ["shipped"],
            [
                item["name"]
                for item in behavior_inventory.rust_items(
                    "crates/demo/src/lib.rs", before
                )
            ],
        )
        self.assertEqual(
            behavior_inventory.rust_shipped_source(before),
            behavior_inventory.rust_shipped_source(after),
        )

    def test_adding_test_only_statement_does_not_change_shipped_proof(self) -> None:
        before = "fn shipped() { baseline(); }\n"
        after = "fn shipped() { baseline(); #[cfg(test)] probe(); }\n"
        before_item = behavior_inventory.rust_items("crates/demo/src/lib.rs", before)[0]
        after_item = behavior_inventory.rust_items("crates/demo/src/lib.rs", after)[0]
        self.assertEqual(
            behavior_inventory.rust_shipped_source(before),
            behavior_inventory.rust_shipped_source(after),
        )
        self.assertEqual(before_item["id"], after_item["id"])
        self.assertEqual(before_item["content_sha256"], after_item["content_sha256"])
        for without_test, with_test, shipped_name in (
            (
                "fn shipped() {}\n",
                "#[cfg(test)]\nfn fixture() {}\nfn shipped() {}\n",
                "shipped",
            ),
            (
                "struct Config { value: u8, }\n",
                "struct Config { value: u8, #[cfg(test)] probe: u8 }\n",
                None,
            ),
            (
                "enum Event { Value, }\n",
                "enum Event { Value, #[cfg(test)] Probe }\n",
                None,
            ),
        ):
            with self.subTest(with_test=with_test):
                self.assertEqual(
                    behavior_inventory.rust_shipped_source(without_test),
                    behavior_inventory.rust_shipped_source(with_test),
                )
                if shipped_name is not None:
                    without_item = behavior_inventory.rust_items(
                        "crates/demo/src/lib.rs", without_test
                    )[0]
                    with_item = behavior_inventory.rust_items(
                        "crates/demo/src/lib.rs", with_test
                    )[0]
                    self.assertEqual(shipped_name, without_item["name"])
                    self.assertEqual(without_item, with_item)

    def test_cfg_comment_trivia_does_not_hide_test_requirement(self) -> None:
        before = """
        #[cfg(/* direct */ test)]
        fn direct_only_test() { fixture_one(); }
        #[cfg_attr(not(/* flag */ test), cfg(/* nested */ test))]
        fn nested_only_test() { fixture_two(); }
        fn shipped() { production(); }
        """
        after = before.replace("fixture_one", "changed_one").replace(
            "fixture_two", "changed_two"
        )
        self.assertEqual(
            ["shipped"],
            [
                item["name"]
                for item in behavior_inventory.rust_items(
                    "crates/demo/src/lib.rs", before
                )
            ],
        )
        self.assertEqual(
            behavior_inventory.rust_shipped_source(before),
            behavior_inventory.rust_shipped_source(after),
        )

    def test_cfg_atom_whitespace_is_semantically_consistent(self) -> None:
        source = """
        #[cfg(any(test, all(feature="tools", not(feature = "tools"))))]
        fn only_test() { fixture(); }
        fn shipped() { production(); }
        """
        self.assertTrue(
            behavior_inventory.cfg_requires_test(
                'any(test, all(feature="tools", not(feature = "tools")))'
            )
        )
        self.assertTrue(
            behavior_inventory.cfg_requires_test(
                'any(test, all(custom="a = b", not(custom = "a = b")))'
            )
        )
        self.assertFalse(
            behavior_inventory.cfg_requires_test(
                'any(test, all(custom="a = b", not(custom = "a=b")))'
            )
        )
        for left, right in ((r'"a,b"', r'"a,c"'), (r'r#"a,b"#', r'r#"a,c"#')):
            with self.subTest(left=left):
                self.assertFalse(
                    behavior_inventory.cfg_requires_test(
                        f"any(test, all(custom={left}, not(custom={right})))"
                    )
                )
        for equivalent in (r'"\x61"', r'r#"a"#'):
            with self.subTest(equivalent=equivalent):
                self.assertTrue(
                    behavior_inventory.cfg_requires_test(
                        f'any(test, all(custom="a", not(custom={equivalent})))'
                    )
                )
        self.assertEqual(
            ["shipped"],
            [
                item["name"]
                for item in behavior_inventory.rust_items(
                    "crates/demo/src/lib.rs", source
                )
            ],
        )

    def test_indexed_projected_lines_match_projection(self) -> None:
        source = "first\n#[cfg(test)]\ntest_one();\ntest_two();\nlast\n"
        first = source.index("#[cfg(test)]")
        last = source.index("test_two();") + len("test_two();") - 1
        ranges = [(first, last)]
        newlines, removed = behavior_inventory.source_line_indexes(source, ranges)
        for index in (0, first, last + 1, source.index("last"), len(source)):
            original, projected = behavior_inventory.indexed_line_number(
                newlines, removed, index
            )
            self.assertEqual(behavior_inventory.line_number(source, index), original)
            self.assertEqual(
                behavior_inventory.project_source_ranges(
                    source, ranges, 0, index
                ).count("\n")
                + 1,
                projected,
            )

    def test_function_discovery_balances_const_generic_bounds(self) -> None:
        source = """
        trait Bound<T> {}
        struct Flag<const VALUE: bool>;
        fn array_bound<T: Bound<[u8; 1]>>() { array_behavior(); }
        fn const_bound<T: Bound<{1}>>() { const_behavior(); }
        fn comparison_return() -> Flag<{ 1 < 2 }> { comparison_behavior(); Flag }
        """
        items = behavior_inventory.rust_items("crates/demo/src/lib.rs", source)
        self.assertEqual(
            ["array_bound", "const_bound", "comparison_return"],
            [item["name"] for item in items],
        )
        self.assertIn("array_behavior", behavior_inventory.rust_shipped_source(source))
        self.assertIn("const_behavior", behavior_inventory.rust_shipped_source(source))
        self.assertIn(
            "comparison_behavior", behavior_inventory.rust_shipped_source(source)
        )

    def test_test_only_type_generic_does_not_suppress_production_body(self) -> None:
        before = """
        struct Wrapper<T: Bound<{1}>, #[cfg(test)] ProbeOne> {
            value: T,
            #[cfg(test)] probe: PhantomData<ProbeOne>
        }
        struct ArrayWrapper<T = [u8; 1], #[cfg(test)] ProbeOne = ()> {
            value: T,
            #[cfg(test)] probe: PhantomData<ProbeOne>
        }
        trait Support<F: Fn() -> u8, #[cfg(test)] ProbeOne> {
            fn shipped(&self) { baseline(); }
            #[cfg(test)] fn probe(&self) {}
        }
        """
        test_changed = before.replace("ProbeOne", "ProbeTwo")
        production_changed = before.replace("value: T", "value: Option<T>").replace(
            "baseline()", "changed_baseline()"
        )
        self.assertEqual(
            behavior_inventory.rust_shipped_source(before),
            behavior_inventory.rust_shipped_source(test_changed),
        )
        before_items = behavior_inventory.rust_items("crates/demo/src/lib.rs", before)
        changed_items = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", production_changed
        )
        self.assertEqual(["shipped"], [item["name"] for item in before_items])
        self.assertNotEqual(
            before_items[0]["content_sha256"], changed_items[0]["content_sha256"]
        )
        self.assertNotEqual(
            behavior_inventory.rust_shipped_source(before),
            behavior_inventory.rust_shipped_source(production_changed),
        )

    def test_test_only_closure_parameter_does_not_suppress_closure_body(self) -> None:
        before = """
        fn shipped() -> usize {
            let compute = |value: Bound<{1}>, #[cfg(test)] probe: u8| production_one();
            compute()
        }
        """
        test_changed = before.replace("probe: u8", "probe: u16")
        production_changed = before.replace("production_one", "production_two")
        before_item = behavior_inventory.rust_items("crates/demo/src/lib.rs", before)[0]
        test_item = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", test_changed
        )[0]
        production_item = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", production_changed
        )[0]
        self.assertEqual(before_item["content_sha256"], test_item["content_sha256"])
        self.assertNotEqual(
            before_item["content_sha256"], production_item["content_sha256"]
        )
        projected = behavior_inventory.rust_shipped_source(before)
        self.assertIn("production_one", projected)
        self.assertNotEqual(
            projected, behavior_inventory.rust_shipped_source(production_changed)
        )

    def test_final_test_only_array_element_preserves_production_postfix(self) -> None:
        before = """
        fn shipped() -> usize {
            [production(), #[cfg(test)] test_only()].len()
        }
        """
        test_changed = before.replace("test_only", "changed_test_only")
        production_changed = before.replace(".len()", ".iter().count()")
        before_item = behavior_inventory.rust_items("crates/demo/src/lib.rs", before)[0]
        test_item = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", test_changed
        )[0]
        production_item = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", production_changed
        )[0]
        self.assertEqual(before_item["content_sha256"], test_item["content_sha256"])
        self.assertNotEqual(
            before_item["content_sha256"], production_item["content_sha256"]
        )
        projected = behavior_inventory.rust_shipped_source(before)
        self.assertIn(".len()", projected)
        self.assertNotEqual(
            projected, behavior_inventory.rust_shipped_source(production_changed)
        )

    def test_test_only_generic_parameter_does_not_suppress_shipped_function(
        self,
    ) -> None:
        before = """
        fn shipped<T: Fn() -> T, #[cfg(test)] ProbeOne>(value: T) -> T {
            fn local() {}
            baseline(value)
        }
        """
        test_changed = before.replace("ProbeOne", "ProbeTwo")
        before_items = behavior_inventory.rust_items("crates/demo/src/lib.rs", before)
        changed_items = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", test_changed
        )
        self.assertEqual(["shipped", "local"], [item["name"] for item in before_items])
        self.assertEqual(
            [
                (item["id"], item["signature_sha256"], item["content_sha256"])
                for item in before_items
            ],
            [
                (item["id"], item["signature_sha256"], item["content_sha256"])
                for item in changed_items
            ],
        )
        production_changed = before.replace("value: T", "value: &T").replace(
            "-> T", "-> Option<&T>"
        )
        production_items = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", production_changed
        )
        self.assertNotEqual(before_items[0]["id"], production_items[0]["id"])
        self.assertIn("baseline", behavior_inventory.rust_shipped_source(before))

    def test_final_test_only_field_does_not_consume_following_production(self) -> None:
        before = """
        struct Config {
            shipped: u8,
            #[cfg(test)] probe: ProbeOne
        }
        enum Choice {
            Shipped,
            #[cfg(test)] Probe
        }
        const BASELINE: usize = production_one();
        fn anchor() {}
        """
        test_changed = before.replace("ProbeOne", "ProbeTwo")
        production_changed = before.replace("production_one", "production_two")
        self.assertEqual(
            behavior_inventory.rust_shipped_source(before),
            behavior_inventory.rust_shipped_source(test_changed),
        )
        projected = behavior_inventory.rust_shipped_source(before)
        self.assertIn("const BASELINE", projected)
        self.assertIn("production_one", projected)
        self.assertNotEqual(
            projected, behavior_inventory.rust_shipped_source(production_changed)
        )

    def test_test_only_signature_components_do_not_change_item_identity(self) -> None:
        before = """
        fn shipped<T>(
            value: T,
            #[cfg(test)] probe: ProbeOne
        ) where #[cfg(test)] T: Iterator<Item = BoundOne> {
            fn local() {}
            baseline(value);
        }
        """
        after = before.replace("ProbeOne", "ProbeTwo").replace("BoundOne", "BoundTwo")
        before_items = behavior_inventory.rust_items("crates/demo/src/lib.rs", before)
        after_items = behavior_inventory.rust_items("crates/demo/src/lib.rs", after)
        self.assertEqual(
            [(item["id"], item["signature_sha256"]) for item in before_items],
            [(item["id"], item["signature_sha256"]) for item in after_items],
        )
        self.assertEqual(["shipped", "local"], [item["name"] for item in before_items])
        changed_body = before.replace("baseline(value)", "changed_baseline(value)")
        changed_items = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", changed_body
        )
        self.assertNotEqual(
            before_items[0]["content_sha256"], changed_items[0]["content_sha256"]
        )
        self.assertNotEqual(
            behavior_inventory.rust_shipped_source(before),
            behavior_inventory.rust_shipped_source(changed_body),
        )

    def test_complete_test_only_typed_items_exclude_their_bodies(self) -> None:
        source = """
        #[cfg(test)]
        fn test_helper(value: u8) { fn leaked() {} test_only(value); }
        #[cfg(test)]
        impl<T: Bound> Subject<T> { fn also_leaked(value: T) {} }
        fn shipped() { production(); }
        """
        self.assertEqual(
            ["shipped"],
            [
                item["name"]
                for item in behavior_inventory.rust_items(
                    "crates/demo/src/lib.rs", source
                )
            ],
        )
        projected = behavior_inventory.rust_shipped_source(source)
        self.assertNotIn("leaked", projected)
        self.assertIn("production", projected)

    def test_visible_inline_test_module_is_not_shipped(self) -> None:
        source = """
        #[cfg(test)]
        pub(crate) mod tests { fn fixture_only() {} }
        fn shipped() {}
        """
        self.assertEqual(
            ["shipped"],
            [
                item["name"]
                for item in behavior_inventory.rust_items(
                    "crates/demo/src/lib.rs", source
                )
            ],
        )

    def test_nested_test_instrumentation_is_projected_from_item_hash(self) -> None:
        before = """
        fn shipped() {
            baseline_behavior();
            #[cfg(test)]
            if 1 < 2 { for value in values { record_probe(value); } };
            #[cfg(test)]
            async { record_async_probe(); }.await_one();
            #[cfg(test)]
            { record_block_probe(); }.method_one();
            #[cfg(test)]
            'probe: loop { record_label_probe(); break 'probe; }
        }
        """
        after = (
            before.replace(
                "if 1 < 2 { for value in values { record_probe(value); } };",
                "assert!(true);",
            )
            .replace("await_one", "await_two")
            .replace("method_one", "method_two")
            .replace("record_label_probe", "changed_label_probe")
        )
        before_item = behavior_inventory.rust_items("crates/demo/src/lib.rs", before)[0]
        after_item = behavior_inventory.rust_items("crates/demo/src/lib.rs", after)[0]
        self.assertEqual(before_item, after_item)
        self.assertEqual(
            behavior_inventory.rust_shipped_source(before),
            behavior_inventory.rust_shipped_source(after),
        )

    def test_local_helpers_include_the_enclosing_function_in_their_identity(
        self,
    ) -> None:
        source = """
        fn first() { fn helper() {} helper(); }
        fn second() { fn helper() {} helper(); }
        """
        helpers = [
            item
            for item in behavior_inventory.rust_items("crates/demo/src/lib.rs", source)
            if item["name"] == "helper"
        ]
        self.assertEqual(2, len(helpers))
        self.assertEqual(2, len({item["id"] for item in helpers}))
        self.assertTrue(any("first" in item["context"] for item in helpers))
        self.assertTrue(any("second" in item["context"] for item in helpers))

    def test_cfg_satisfiability_is_bounded_fail_closed(self) -> None:
        expression = (
            "all(test," + ",".join(f'feature = "f{index}"' for index in range(20)) + ")"
        )
        with self.assertRaisesRegex(ValueError, "too many distinct atoms"):
            behavior_inventory.cfg_requires_test(expression)

    def test_nested_block_comments_do_not_hide_shipped_siblings(self) -> None:
        source = """
        mod outer {
            #[cfg(test)]
            mod test_support {
                /* outer /* inner */ { */
            }
            fn shipped() { publish_dirt(); }
        }
        """
        self.assertEqual(
            ["shipped"],
            [
                item["name"]
                for item in behavior_inventory.rust_items(
                    "crates/demo/src/lib.rs", source
                )
            ],
        )
        self.assertIn("publish_dirt", behavior_inventory.rust_shipped_source(source))

    def test_mixed_generated_file_keeps_handwritten_item_visible(self) -> None:
        source = """
        #[cfg(test)]
        fn fixture_only() {
            record_fixture();
            record_more_fixture();
        }
        // @generated-region begin schema
        pub fn generated_getter() -> u32 { 7 }
        // @generated-region end schema

        pub fn handwritten_setter(value: u32) {
            publish_dirt(value);
        }
        """
        items = behavior_inventory.rust_items("crates/demo/src/mixed.rs", source)
        by_name = {item["name"]: item for item in items}
        self.assertEqual("generated", by_name["generated_getter"]["region"])
        self.assertEqual("handwritten", by_name["handwritten_setter"]["region"])
        self.assertIn(
            "dirt-publication", by_name["handwritten_setter"]["behavior_kinds"]
        )
        legacy_codegen = {
            "owners": set(),
            "adapted": False,
            "evidence": {"rust-additions.toml:codegen"},
            "addition_category": "codegen",
        }
        for item in items:
            behavior_inventory.enrich_rust_item(item, legacy_codegen)
        self.assertEqual("generated", by_name["generated_getter"]["provenance"])
        self.assertEqual("host-support", by_name["handwritten_setter"]["provenance"])

    def test_rust_items_include_trait_defaults_and_impl_methods(self) -> None:
        source = """
        trait Wake { fn wake(&mut self) { self.mark_dirty(); } }
        impl Wake for Node {
            fn wake(&mut self) { self.mark_dirty(); }
        }
        """
        items = behavior_inventory.rust_items("crates/demo/src/lib.rs", source)
        self.assertEqual(["wake", "wake"], [item["name"] for item in items])
        self.assertEqual(2, len({item["id"] for item in items}))
        self.assertIn("::trait Wake::wake@", items[0]["id"])
        self.assertIn("::impl Wake for Node::wake@", items[1]["id"])

    def test_inline_module_contexts_distinguish_sibling_items(self) -> None:
        source = """
        mod unix { fn init() {} }
        mod windows { fn init() {} }
        """
        items = behavior_inventory.rust_items("crates/demo/src/lib.rs", source)
        self.assertEqual(2, len(items))
        self.assertEqual(2, len({item["id"] for item in items}))
        raw_source = source.replace("mod unix", "mod r#unix").replace(
            "mod windows", "mod r#windows"
        )
        raw_items = behavior_inventory.rust_items("crates/demo/src/lib.rs", raw_source)
        self.assertEqual(
            [item["id"] for item in items], [item["id"] for item in raw_items]
        )

    def test_unicode_function_and_module_identifiers_are_itemized(self) -> None:
        composed = "mod café { fn résumé() { publish_dirt(); } }"
        decomposed = unicodedata.normalize("NFD", composed)
        composed_item = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", composed
        )[0]
        decomposed_item = behavior_inventory.rust_items(
            "crates/demo/src/lib.rs", decomposed
        )[0]

        self.assertEqual("résumé", composed_item["name"])
        self.assertIn("::mod café::résumé@", composed_item["id"])
        self.assertEqual(composed_item["id"], decomposed_item["id"])

    def test_rust_lifetimes_do_not_mask_following_items(self) -> None:
        source = """
        pub fn borrow<'a>(value: &'a Value) -> &'a Value { value }
        pub fn after_lifetime() { publish_dirt(); }
        """
        self.assertEqual(
            ["borrow", "after_lifetime"],
            [
                item["name"]
                for item in behavior_inventory.rust_items(
                    "crates/demo/src/lib.rs", source
                )
            ],
        )

    def test_array_return_semicolon_does_not_look_like_a_trait_declaration(
        self,
    ) -> None:
        source = """
        fn digest() -> [u8; 32] { [0; 32] }
        trait Digest { fn declaration_only() -> [u8; 32]; }
        """
        self.assertEqual(
            ["digest"],
            [
                item["name"]
                for item in behavior_inventory.rust_items(
                    "crates/demo/src/lib.rs", source
                )
            ],
        )

    def test_generated_banner_does_not_hide_a_mixed_file(self) -> None:
        source = """
        // @generated by a tool; do not edit.
        pub fn generated_looking() {}
        pub fn handwritten_setter() { publish_dirt(); }
        """
        items = behavior_inventory.rust_items("crates/demo/src/mixed.rs", source)
        self.assertEqual({"handwritten"}, {item["region"] for item in items})

    def test_generated_region_markers_must_be_dedicated_comments(self) -> None:
        for source in (
            r"""
            const BEGIN: &str = "@generated-region begin schema";
            pub fn handwritten_setter(value: u32) { publish_dirt(value); }
            const END: &str = "@generated-region end schema";
            """,
            """
            /// Documentation mentions @generated-region begin schema.
            pub fn handwritten_setter(value: u32) { publish_dirt(value); }
            // Prose mentions @generated-region end schema without marking it.
            """,
            r"""
            const TEXT: &str = r###"
            // @generated-region begin schema
            "###;
            pub fn handwritten_setter(value: u32) { publish_dirt(value); }
            const MORE_TEXT: &str = r###"
            // @generated-region end schema
            "###;
            """,
            """
            /*
            // @generated-region begin schema
            */
            pub fn handwritten_setter(value: u32) { publish_dirt(value); }
            /*
            // @generated-region end schema
            */
            """,
        ):
            with self.subTest(source=source):
                items = behavior_inventory.rust_items(
                    "crates/demo/src/mixed.rs", source
                )
                self.assertEqual(
                    {"handwritten"},
                    {item["region"] for item in items},
                )

    def test_partial_generated_region_overlap_is_rejected(self) -> None:
        source = """
        // @generated-region begin schema
        pub fn generated_shell() {
        // @generated-region end schema
            handwritten_side_effect();
        }
        """
        with self.assertRaisesRegex(ValueError, "partially overlaps"):
            behavior_inventory.rust_items("crates/demo/src/mixed.rs", source)

    def test_inline_cfg_test_items_are_not_shipped_inventory(self) -> None:
        source = """
        pub fn shipped() {}
        /// Fixture documentation one.
        #[cfg(test)]
        mod tests { fn fixture_only() {} }
        /** Inline fixture documentation one. */
        #[test]
        fn inline_test() { fn local_fixture() {} }
        """
        self.assertEqual(
            ["shipped"],
            [
                item["name"]
                for item in behavior_inventory.rust_items(
                    "crates/demo/src/lib.rs", source
                )
            ],
        )
        changed_docs = source.replace("documentation one", "documentation two")
        self.assertEqual(
            behavior_inventory.rust_shipped_source(source),
            behavior_inventory.rust_shipped_source(changed_docs),
        )
        self.assertNotIn(
            "Fixture documentation", behavior_inventory.rust_shipped_source(source)
        )
        self.assertNotIn(
            "Inline fixture documentation",
            behavior_inventory.rust_shipped_source(source),
        )

    def test_inner_cfg_test_excludes_the_enclosing_source_or_module(self) -> None:
        crate_source = "#![cfg(test)]\nfn fixture_only() { probe_one(); }\n"
        changed_crate_source = crate_source.replace("probe_one", "probe_two")
        self.assertEqual(
            [],
            behavior_inventory.rust_items("crates/demo/src/fixture.rs", crate_source),
        )
        self.assertEqual(
            behavior_inventory.rust_shipped_source(crate_source),
            behavior_inventory.rust_shipped_source(changed_crate_source),
        )

        inline_source = """
        mod fixtures {
            #![cfg(test)]
            fn fixture_only() { probe(); }
        }
        fn shipped() { production(); }
        """
        self.assertEqual(
            ["shipped"],
            [
                item["name"]
                for item in behavior_inventory.rust_items(
                    "crates/demo/src/lib.rs", inline_source
                )
            ],
        )
        self.assertNotIn("probe", behavior_inventory.rust_shipped_source(inline_source))

        macro_source = (
            "macro_rules! tokens {\n"
            "    () => { mod fixtures { #![cfg(test)] fn fixture() {} } };\n"
            "}\n"
            "fn shipped() { production(); }\n"
        )
        self.assertIn("fixture", behavior_inventory.rust_shipped_source(macro_source))

    def test_cfg_alternatives_and_not_test_remain_shipped(self) -> None:
        source = """
        #[cfg(any(test, target_arch = "wasm32"))]
        fn wasm_runtime_behavior() {}
        #[cfg(not(test))]
        fn production_only() {}
        #[cfg(all(test, feature = "tools"))]
        fn test_required() {}
        #[cfg(not(not(test)))]
        fn double_negated_test_required() {}
        #[cfg(test)]
        impl Scratch { fn test_helper() {} }
        #[cfg(test)]
        trait TestFixture { fn trait_helper() {} }
        """
        self.assertEqual(
            ["wasm_runtime_behavior", "production_only"],
            [
                item["name"]
                for item in behavior_inventory.rust_items(
                    "crates/demo/src/lib.rs", source
                )
            ],
        )

    def test_cfg_literals_distinguish_platform_item_identities(self) -> None:
        source = """
        #[cfg(target_os = "macos")]
        impl Backend { fn run(&self) {} }
        #[cfg(target_os = "linux")]
        impl Backend { fn run(&self) {} }
        """
        items = behavior_inventory.rust_items("crates/demo/src/lib.rs", source)
        self.assertEqual(2, len(items))
        self.assertEqual(2, len({item["id"] for item in items}))

    def test_multiline_cfg_literals_distinguish_context_identities(self) -> None:
        source = """
        #[cfg(
            target_os = "macos"
        )]
        impl Backend { fn run(&self) {} }
        #[cfg(
            target_os = "linux"
        )]
        impl Backend { fn run(&self) {} }
        """
        items = behavior_inventory.rust_items("crates/demo/src/lib.rs", source)
        self.assertEqual(2, len(items))
        self.assertEqual(2, len({item["id"] for item in items}))

    def test_cfg_atoms_have_consistent_values_within_a_predicate(self) -> None:
        source = """
        #[cfg(any(test, all(feature = "tools", not(feature = "tools"))))]
        fn contradictory_test_only() {}
        """
        self.assertEqual(
            [], behavior_inventory.rust_items("crates/demo/src/lib.rs", source)
        )

    def test_cfg_atoms_are_consistent_across_conjoined_attributes(self) -> None:
        source = """
        #[cfg(any(test, feature="tools"))]
        #[cfg(not(feature = "tools"))]
        fn cross_attribute_test_required() {}
        """
        self.assertEqual(
            [], behavior_inventory.rust_items("crates/demo/src/lib.rs", source)
        )

    def test_single_value_builtin_cfg_atoms_are_mutually_exclusive(self) -> None:
        source = """
        #[cfg(any(test, all(target_os = "linux", target_os = "macos")))]
        fn same_attribute_test_only() {}
        #[cfg(any(test, target_arch = "x86_64"))]
        #[cfg(target_arch = "aarch64")]
        fn cross_attribute_test_only() {}
        #[cfg(any(test, all(feature = "one", feature = "two")))]
        fn multiple_features_remain_production_capable() {}
        """
        self.assertEqual(
            ["multiple_features_remain_production_capable"],
            [
                item["name"]
                for item in behavior_inventory.rust_items(
                    "crates/demo/src/lib.rs", source
                )
            ],
        )

    def test_distinct_cfg_feature_atoms_survive_structural_masking(self) -> None:
        source = """
        #[cfg(any(test, all(feature = "a", not(feature = "b"))))]
        fn production_capable() { fn shipped_local() {} }
        """
        self.assertEqual(
            ["production_capable", "shipped_local"],
            [
                item["name"]
                for item in behavior_inventory.rust_items(
                    "crates/demo/src/lib.rs", source
                )
            ],
        )
        self.assertIn("shipped_local", behavior_inventory.rust_shipped_source(source))

    def test_multiline_direct_cfg_test_attribute_is_excluded(self) -> None:
        source = """
        #[cfg(all(
            test,
            feature = "tools"
        ))]
        fn direct_test_only() {}
        #[cfg(any(
            test,
            target_arch = "wasm32"
        ))]
        fn wasm_runtime_behavior() {}
        """
        self.assertEqual(
            ["wasm_runtime_behavior"],
            [
                item["name"]
                for item in behavior_inventory.rust_items(
                    "crates/demo/src/lib.rs", source
                )
            ],
        )

    def test_signature_identity_is_stable_when_overloads_reorder(self) -> None:
        first = "fn value(input: u8) {}\nfn value(input: u16) {}\n"
        second = "fn value(input: u16) {}\nfn value(input: u8) {}\n"
        self.assertEqual(
            {
                item["id"]
                for item in behavior_inventory.rust_items(
                    "crates/demo/src/lib.rs", first
                )
            },
            {
                item["id"]
                for item in behavior_inventory.rust_items(
                    "crates/demo/src/lib.rs", second
                )
            },
        )


class GateTests(unittest.TestCase):
    def test_runtime_generator_build_scripts_are_inventoried(self) -> None:
        repo_root = TOOL.parents[2]
        self.assertEqual(
            [
                "rustc-env:NUX_RUNTIME_SOURCE_REVISION",
                "rustc-env:NUX_CAPI_BUILD_PROVENANCE",
                "OUT_DIR/nux_capi.generated.h",
                "crates/nux-capi/include/nux_capi.generated.h",
            ],
            behavior_inventory.RUST_GENERATOR_OUTPUTS["crates/nux-capi/build.rs"],
        )
        self.assertEqual(
            ["native-link:nuxie_renderer_ffi"],
            behavior_inventory.RUST_GENERATOR_OUTPUTS[
                "crates/nuxie-renderer-ffi/build.rs"
            ],
        )
        capi_build = (repo_root / "crates/nux-capi/build.rs").read_text()
        for directive in (
            "cargo:rustc-env=NUX_RUNTIME_SOURCE_REVISION=",
            "cargo:rustc-env=NUX_CAPI_BUILD_PROVENANCE=",
            'join("nux_capi.generated.h")',
            'join("include/nux_capi.generated.h")',
        ):
            self.assertIn(directive, capi_build)
        renderer_build = (repo_root / "crates/nuxie-renderer-ffi/build.rs").read_text()
        self.assertIn('compile("nuxie_renderer_ffi")', renderer_build)
        self.assertIn("rustc-link-lib=static=nuxie_renderer_ffi", renderer_build)
        candidates = {
            path.relative_to(repo_root).as_posix()
            for path in behavior_inventory.rust_source_candidates(repo_root)
        }
        self.assertEqual(
            set(behavior_inventory.RUST_GENERATOR_OUTPUTS),
            {path for path in candidates if path.endswith("/build.rs")},
        )
        roots = {
            path.relative_to(repo_root).as_posix()
            for path in behavior_inventory.rust_crate_roots(
                repo_root, behavior_inventory.rust_source_candidates(repo_root)
            )
        }
        self.assertTrue(set(behavior_inventory.RUST_GENERATOR_OUTPUTS) <= roots)
        for path, outputs in behavior_inventory.RUST_GENERATOR_OUTPUTS.items():
            with self.subTest(path=path):
                self.assertIn(path, candidates)
                self.assertTrue(outputs)
                self.assertTrue(
                    behavior_inventory.rust_items(path, (repo_root / path).read_text())
                )

    def test_runtime_generator_roots_follow_cargo_manifest_configuration(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            custom_crate = repo_root / "crates/demo"
            custom_crate.mkdir(parents=True)
            (custom_crate / "Cargo.toml").write_text(
                '[package]\nname = "demo"\nversion = "0.1.0"\n'
                'build = "codegen/runtime_generator.rs"\n'
            )
            (custom_crate / "build.rs").write_text("fn inert() {}\n")
            custom_generator = custom_crate / "codegen/runtime_generator.rs"
            custom_generator.parent.mkdir()
            custom_generator.write_text("fn main() {}\n")

            disabled_crate = repo_root / "crates/disabled"
            disabled_crate.mkdir()
            (disabled_crate / "Cargo.toml").write_text(
                '[package]\nname = "disabled"\nversion = "0.1.0"\nbuild = false\n'
            )
            disabled_generator = disabled_crate / "build.rs"
            disabled_generator.write_text("fn inert() {}\n")

            generators = behavior_inventory.rust_generator_paths(
                repo_root, crate_names=("demo", "disabled")
            )
            self.assertEqual({custom_generator.resolve()}, generators)

    def test_rust_candidates_cover_shipped_sources_outside_src(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            (crate / "src").mkdir(parents=True)
            (crate / "tests").mkdir()
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
                '[lib]\npath = "runtime.rs"\n'
            )
            runtime = crate / "runtime.rs"
            shared = crate / "shared.rs"
            included = crate / "shipped.inc"
            build = crate / "build.rs"
            build_helper = crate / "build_helper.rs"
            unreachable = crate / "src/unreachable.rs"
            test_only = crate / "tests/oracle.rs"
            runtime.write_text(
                '#[path = "shared.rs"]\nmod shared;\n'
                'include!("shipped.inc");\n'
                'myinclude!("not-source.inc");\n'
                'r#include!("not-source-raw.inc");\n'
                "macro_rules! invoke { ($include:ident) => { "
                '$include!("not-source-metavariable.inc"); } }\n'
                'éinclude!("not-source-unicode.inc");\n'
                'áinclude!("not-source-combining.inc");\n'
            )
            shared.write_text("fn shipped() {}\n")
            included.write_text("fn included_behavior() {}\n")
            build.write_text(
                "mod build_helper;\nfn main() { build_helper::generate(); }\n"
            )
            build_helper.write_text("pub fn generate() {}\n")
            unreachable.write_text("fn not_compiled() {}\n")
            test_only.write_text("fn test_only() {}\n")

            candidates = behavior_inventory.rust_source_candidates(
                repo_root, crate_names=("nuxie",)
            )
            self.assertEqual(
                {
                    runtime.resolve(),
                    build.resolve(),
                    unreachable.resolve(),
                },
                {path.resolve() for path in candidates},
            )
            excluded = behavior_inventory.external_test_module_paths(
                repo_root, candidates
            )
            self.assertEqual(
                {
                    runtime.resolve(),
                    shared.resolve(),
                    included.resolve(),
                    build.resolve(),
                    build_helper.resolve(),
                    unreachable.resolve(),
                },
                {path.resolve() for path in candidates},
            )
            self.assertNotIn(runtime.resolve(), excluded)
            self.assertNotIn(shared.resolve(), excluded)
            self.assertNotIn(included.resolve(), excluded)
            self.assertNotIn(build.resolve(), excluded)
            self.assertNotIn(build_helper.resolve(), excluded)
            self.assertIn(unreachable.resolve(), excluded)
            self.assertNotIn(test_only.resolve(), candidates)

    def test_macro_expanded_production_module_is_reachable(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            behavior = source / "behavior.rs"
            unicode_behavior = source / "béhavior.rs"
            raw_behavior = source / "raw_behavior.rs"
            test_only = source / "test_only.rs"
            lib.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
                "macro_rules! chargé { ($module:ident) => { mod $module; } }\n"
                "macro_rules! r#type { ($module:ident) => { mod $module; } }\n"
                "load!(behavior);\n"
                "chargé!(béhavior);\n"
                "r#type!(raw_behavior);\n"
                "#[cfg(test)]\nload!(test_only);\n"
            )
            behavior.write_text("fn shipped() {}\n")
            unicode_behavior.write_text("fn unicode_shipped() {}\n")
            raw_behavior.write_text("fn raw_shipped() {}\n")
            test_only.write_text("fn fixture() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root,
                [lib, behavior, unicode_behavior, raw_behavior, test_only],
            )
            self.assertNotIn(behavior.resolve(), excluded)
            self.assertNotIn(unicode_behavior.resolve(), excluded)
            self.assertNotIn(raw_behavior.resolve(), excluded)
            self.assertIn(test_only.resolve(), excluded)

            for variable in ("módulo", "mo\u0301dulo"):
                with self.subTest(unicode_metavariable=variable):
                    lib.write_text(
                        f"macro_rules! load {{ (${variable}:ident) => "
                        f"{{ mod ${variable}; }} }}\n"
                        f"macro_rules! wrapper {{ (${variable}:ident) => "
                        f"{{ load!(${variable}); }} }}\n"
                        "wrapper!(behavior);\n"
                    )
                    excluded = behavior_inventory.external_test_module_paths(
                        repo_root, [lib, behavior]
                    )
                    self.assertNotIn(behavior.resolve(), excluded)

            lib.write_text(
                "macro_rules! load { () => {\n"
                '    #[path = "behavior.rs"] mod béhavior;\n'
                "} }\n"
                "load!();\n"
            )
            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

            for literal in ('"x"', "'x'", 'r#"x"#', '"/*"', '"//"'):
                with self.subTest(literal_pattern=literal):
                    lib.write_text(
                        "macro_rules! load {\n"
                        "    () => {};\n"
                        f"    ({literal}) => {{ mod behavior; }};\n"
                        "}\n"
                        f"load!({literal});\n"
                    )
                    excluded = behavior_inventory.external_test_module_paths(
                        repo_root, [lib, behavior]
                    )
                    self.assertNotIn(behavior.resolve(), excluded)

            lib.write_text(
                "macro_rules! load {\n"
                '    ("x" $($value:ident),*) => {};\n'
                '    ("x" foo) => { mod behavior; };\n'
                "}\n"
                'load!("x" foo);\n'
            )
            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertIn(behavior.resolve(), excluded)

            for pattern, invocation in (
                ('"$(" +', '"$(" +'),
                ("foo /* $( */ +", "foo +"),
            ):
                with self.subTest(non_repetition_payload=pattern):
                    lib.write_text(
                        "macro_rules! load {\n"
                        f"    ({pattern}) => {{ mod behavior; }};\n"
                        "    ($token:tt) => {};\n"
                        "}\n"
                        f"load!({invocation});\n"
                    )
                    excluded = behavior_inventory.external_test_module_paths(
                        repo_root, [lib, behavior]
                    )
                    self.assertNotIn(behavior.resolve(), excluded)

            for pattern in (
                "$ /* trivia */ ($module:ident),*",
                "$ // trivia\n ($module:ident),*",
                "$($module:ident) /* trivia */ ,*",
            ):
                with self.subTest(commented_repetition=pattern):
                    lib.write_text(
                        "macro_rules! load {\n"
                        f"    ({pattern}) => {{}};\n"
                        "    ($module:ident) => { mod behavior; };\n"
                        "}\n"
                        "load!(item);\n"
                    )
                    excluded = behavior_inventory.external_test_module_paths(
                        repo_root, [lib, behavior]
                    )
                    self.assertIn(behavior.resolve(), excluded)

            for suffix, invocation in (
                ('"$("', 'foo "$("'),
                ("/* $( */ end", "foo end"),
            ):
                with self.subTest(repetition_suffix_payload=suffix):
                    lib.write_text(
                        "macro_rules! load {\n"
                        f"    ($($module:ident),* {suffix}) => {{}};\n"
                        f"    ({invocation}) => {{ mod behavior; }};\n"
                        "}\n"
                        f"load!({invocation});\n"
                    )
                    excluded = behavior_inventory.external_test_module_paths(
                        repo_root, [lib, behavior]
                    )
                    self.assertIn(behavior.resolve(), excluded)

            for separator in ("*=", "+=", '"x"', 'r#"x"#', "123", "'x'", "'a"):
                with self.subTest(joint_repetition_separator=separator):
                    invocation = f"foo {separator} bar"
                    lib.write_text(
                        "macro_rules! load {\n"
                        f"    ($($module:ident){separator}*) => {{}};\n"
                        f"    ({invocation}) => {{ mod behavior; }};\n"
                        "}\n"
                        f"load!({invocation});\n"
                    )
                    excluded = behavior_inventory.external_test_module_paths(
                        repo_root, [lib, behavior]
                    )
                    self.assertIn(behavior.resolve(), excluded)

            for attached, separated in (
                ("123foo", "123 foo"),
                ('"x"foo', '"x" foo'),
                ('"é"', '"é"'),
                ("=>", "= >"),
                ("..", ". ."),
            ):
                with self.subTest(attached_literal=attached, separated=separated):
                    lib.write_text(
                        "macro_rules! load {\n"
                        f"    ({separated}) => {{}};\n"
                        f"    ({attached}) => {{ mod behavior; }};\n"
                        "}\n"
                        f"load!({attached});\n"
                    )
                    excluded = behavior_inventory.external_test_module_paths(
                        repo_root, [lib, behavior]
                    )
                    self.assertNotIn(behavior.resolve(), excluded)

            lib.write_text(
                "macro_rules! load {\n"
                "    (1 .. 2) => { mod behavior; };\n"
                "    ($token:tt) => {};\n"
                "}\n"
                "load!(1..2);\n"
            )
            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

            lib.write_text(
                "macro_rules! inner {\n"
                "    (behavior) => { mod shipped; };\n"
                "    ($other:ident) => {};\n"
                "}\n"
                "macro_rules! outer {\n"
                "    ($módulo:ident) => { inner!($módulo); }\n"
                "}\n"
                "outer!(behavior);\n"
            )
            shipped = source / "shipped.rs"
            shipped.write_text("fn shipped() {}\n")
            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior, shipped]
            )
            self.assertNotIn(shipped.resolve(), excluded)

            for literal in ('"$module"', 'r#"$module"#', 'b"$module"'):
                with self.subTest(delegated_literal=literal):
                    lib.write_text(
                        "macro_rules! inner {\n"
                        f"    ({literal}, behavior) => {{ mod shipped; }};\n"
                        "    ($literal:tt, $other:ident) => {};\n"
                        "}\n"
                        "macro_rules! outer {\n"
                        f"    ($module:ident) => {{ inner!({literal}, $module); }}\n"
                        "}\n"
                        "outer!(behavior);\n"
                    )
                    excluded = behavior_inventory.external_test_module_paths(
                        repo_root, [lib, behavior, shipped]
                    )
                    self.assertNotIn(shipped.resolve(), excluded)

            lib.write_text(
                "macro_rules! load {\n"
                '    ("x" /* matcher /* nested */ trivia */) => { mod behavior; };\n'
                "    () => {};\n"
                "}\n"
                'load!(/* invocation trivia */ "x");\n'
            )
            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

            lib.write_text(
                "macro_rules! load {\n"
                "    ({ /* matcher trivia */ value }) => { mod behavior; };\n"
                "    () => {};\n"
                "}\n"
                "load!({ value });\n"
            )
            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

            lib.write_text(
                "macro_rules! load {\n"
                "    ($module /* matcher trivia */ : ident) => { mod $module; };\n"
                "    ($module:ident) => {};\n"
                "}\n"
                "load!(behavior);\n"
            )
            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

            for invocation in (
                "/* leading trivia */ behavior",
                '/* leading trivia */ "x"',
                '"x" /* trailing trivia */',
                '/* leading trivia */ b"x"',
                '/* leading trivia */ br#"x"#',
                "/* leading trivia */ { value }",
            ):
                with self.subTest(tt_invocation=invocation):
                    lib.write_text(
                        "macro_rules! load {\n"
                        "    ($token:tt) => { mod behavior; };\n"
                        "    () => {};\n"
                        "}\n"
                        f"load!({invocation});\n"
                    )
                    excluded = behavior_inventory.external_test_module_paths(
                        repo_root, [lib, behavior]
                    )
                    self.assertNotIn(behavior.resolve(), excluded)

            lib.write_text(
                "macro_rules! load { ($token:tt) => { mod $token; }; }\n"
                "load!(/* leading trivia */ behavior /* trailing trivia */);\n"
            )
            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_cross_file_macro_expanded_module_inherits_test_state(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            macros = source / "macros.rs"
            behavior = source / "behavior.rs"
            test_only = source / "test_only.rs"
            lib.write_text(
                "#[macro_use]\nmod macros;\n"
                "load!(behavior);\n"
                "#[cfg(test)]\nload!(test_only);\n"
            )
            macros.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
            )
            behavior.write_text("fn shipped() {}\n")
            test_only.write_text("fn fixture() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, macros, behavior, test_only]
            )
            self.assertNotIn(macros.resolve(), excluded)
            self.assertNotIn(behavior.resolve(), excluded)
            self.assertIn(test_only.resolve(), excluded)

    def test_private_macro_does_not_redefine_root_invocation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            private = source / "private.rs"
            ghost = source / "ghost.rs"
            lib.write_text(
                "mod private;\n"
                "macro_rules! load { ($module:ident) => {}; }\n"
                "load!(ghost);\n"
            )
            private.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
            )
            ghost.write_text('compile_error!("not shipped");\n')

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, private, ghost]
            )
            self.assertNotIn(private.resolve(), excluded)
            self.assertIn(ghost.resolve(), excluded)

    def test_imported_macro_is_visible_until_locally_shadowed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            macros = source / "macros.rs"
            behavior = source / "behavior.rs"
            ghost = source / "ghost.rs"
            lib.write_text(
                "#[macro_use]\nmod macros;\n"
                "load!(behavior);\n"
                "macro_rules! load { ($module:ident) => {}; }\n"
                "load!(ghost);\n"
            )
            macros.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
            )
            behavior.write_text("fn shipped() {}\n")
            ghost.write_text('compile_error!("shadowed");\n')

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, macros, behavior, ghost]
            )
            self.assertNotIn(behavior.resolve(), excluded)
            self.assertIn(ghost.resolve(), excluded)

    def test_test_only_macro_does_not_shadow_production_import(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            macros = source / "macros.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "#[macro_use]\nmod macros;\n"
                "#[cfg(test)]\n"
                "macro_rules! load { ($module:ident) => {}; }\n"
                "load!(behavior);\n"
            )
            macros.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
            )
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, macros, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_block_local_macro_does_not_shadow_outer_scope_after_block(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            macros = source / "macros.rs"
            inner = source / "inner.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "#[macro_use]\nmod macros;\n"
                "fn scope() {\n"
                "    macro_rules! load { ($module:ident) => {}; }\n"
                "    load!(inner);\n"
                "}\n"
                "load!(behavior);\n"
            )
            macros.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
            )
            inner.write_text('compile_error!("shadowed in block");\n')
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, macros, inner, behavior]
            )
            self.assertIn(inner.resolve(), excluded)
            self.assertNotIn(behavior.resolve(), excluded)

    def test_unexpanded_macro_body_does_not_reach_literal_metavariable(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            macros = source / "macros.rs"
            module = source / "module.rs"
            lib.write_text(
                "#[macro_use]\nmod macros;\n"
                "macro_rules! wrapper {\n"
                "    ($module:ident) => { load!($module); }\n"
                "}\n"
            )
            macros.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
            )
            module.write_text('compile_error!("wrapper was not invoked");\n')

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, macros, module]
            )
            self.assertIn(module.resolve(), excluded)

    def test_unexpanded_macro_literal_module_is_not_reachable(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            behavior = source / "behavior.rs"
            lib.write_text("macro_rules! hidden { () => { mod behavior; } }\n")
            behavior.write_text('compile_error!("macro was not invoked");\n')

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertIn(behavior.resolve(), excluded)

            lib.write_text(
                "macro_rules! hidden { () => { mod behavior; } }\n" "hidden!();\n"
            )
            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

            alternate = source / "feature_behavior.rs"
            alternate.write_text("fn feature_shipped() {}\n")
            default = source / "behavior.rs"
            default.write_text("fn default_shipped() {}\n")
            lib.write_text(
                "macro_rules! load {\n"
                "    () => {\n"
                '        #[cfg_attr(feature = "x", path = "feature_behavior.rs")]\n'
                "        mod behavior;\n"
                "    };\n"
                "}\n"
                "load!();\n"
            )
            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, default, alternate]
            )
            self.assertNotIn(default.resolve(), excluded)
            self.assertNotIn(alternate.resolve(), excluded)

    def test_include_inside_macro_is_reached_only_when_invoked(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            literal = source / "literal.rs"
            variable = source / "variable.rs"
            literal.write_text("fn literal() {}\n")
            variable.write_text("fn variable() {}\n")
            definitions = (
                "macro_rules! literal_include {\n"
                '    () => { include!("literal.rs"); };\n'
                "}\n"
                "macro_rules! variable_include {\n"
                "    ($path:tt) => { include!($path); };\n"
                "}\n"
            )

            lib.write_text(definitions)
            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, literal, variable]
            )
            self.assertIn(literal.resolve(), excluded)
            self.assertIn(variable.resolve(), excluded)

            lib.write_text(
                definitions
                + "literal_include!();\n"
                + 'variable_include!("variable.rs");\n'
            )
            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, literal, variable]
            )
            self.assertNotIn(literal.resolve(), excluded)
            self.assertNotIn(variable.resolve(), excluded)

    def test_invoked_wrapper_macro_reaches_concrete_module(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            macros = source / "macros.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "#[macro_use]\nmod macros;\n"
                "macro_rules! wrapper {\n"
                "    ($module:ident) => { load!($module); }\n"
                "}\n"
                "wrapper!(behavior);\n"
            )
            macros.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
            )
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, macros, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_wrapper_selects_only_the_invoked_macro_arm(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            macros = source / "macros.rs"
            ghost = source / "ghost.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "#[macro_use]\nmod macros;\n"
                "macro_rules! wrapper {\n"
                "    ($module:ident) => {};\n"
                "    (generate $module:ident) => { load!($module); };\n"
                "}\n"
                "wrapper!(ghost);\n"
                "wrapper!(generate behavior);\n"
            )
            macros.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
            )
            ghost.write_text('compile_error!("no-op arm");\n')
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, macros, ghost, behavior]
            )
            self.assertIn(ghost.resolve(), excluded)
            self.assertNotIn(behavior.resolve(), excluded)

    def test_wrapper_resolves_callee_at_invocation_position(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "macro_rules! wrapper {\n"
                "    ($module:ident) => { load!($module); }\n"
                "}\n"
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
                "wrapper!(behavior);\n"
            )
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_imported_wrapper_retains_arm_behavior(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            macros = source / "macros.rs"
            ghost = source / "ghost.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "#[macro_use]\nmod macros;\n"
                "wrapper!(ghost);\n"
                "wrapper!(generate behavior);\n"
            )
            macros.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
                "macro_rules! wrapper {\n"
                "    ($module:ident) => {};\n"
                "    (generate $module:ident) => { load!($module); };\n"
                "}\n"
            )
            ghost.write_text('compile_error!("no-op imported arm");\n')
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, macros, ghost, behavior]
            )
            self.assertIn(ghost.resolve(), excluded)
            self.assertNotIn(behavior.resolve(), excluded)

    def test_nested_imported_wrapper_retains_delegated_behavior(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            outer = source / "outer.rs"
            inner = source / "outer/inner.rs"
            behavior = source / "behavior.rs"
            inner.parent.mkdir()
            lib.write_text("#[macro_use]\nmod outer;\nwrapper!(behavior);\n")
            outer.write_text("#[macro_use]\nmod inner;\n")
            inner.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
                "macro_rules! wrapper { ($module:ident) => { load!($module); } }\n"
            )
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, outer, inner, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_transitive_local_and_imported_wrappers_reach_module(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            macros = source / "macros.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "#[macro_use]\nmod macros;\n"
                "macro_rules! outer { ($module:ident) => { inner!($module); } }\n"
                "outer!(behavior);\n"
            )
            macros.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
                "macro_rules! inner { ($module:ident) => { load!($module); } }\n"
            )
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, macros, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_delegated_wrapper_binds_literal_and_variable_arguments(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "macro_rules! load {\n"
                "    (generate $module:ident) => { mod $module; }\n"
                "}\n"
                "macro_rules! wrapper {\n"
                "    ($module:ident) => { load!(generate $module); }\n"
                "}\n"
                "wrapper!(behavior);\n"
            )
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_multiple_delegated_calls_have_independent_arguments(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            first = source / "first.rs"
            second = source / "second.rs"
            unrelated = source / "load.rs"
            lib.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
                "macro_rules! wrapper {\n"
                "    ($a:ident, $b:ident) => { load!($a); load!($b); }\n"
                "}\n"
                "wrapper!(first, second);\n"
            )
            first.write_text("fn first_shipped() {}\n")
            second.write_text("fn second_shipped() {}\n")
            unrelated.write_text('compile_error!("not an argument");\n')

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, first, second, unrelated]
            )
            self.assertNotIn(first.resolve(), excluded)
            self.assertNotIn(second.resolve(), excluded)
            self.assertIn(unrelated.resolve(), excluded)

    def test_nested_invocation_tokens_are_owned_by_outer_macro(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            ghost = source / "ghost.rs"
            lib.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
                "macro_rules! noop { ($($tokens:tt)*) => {}; }\n"
                "macro_rules! wrapper {\n"
                "    ($module:ident) => { noop!(load!($module)); }\n"
                "}\n"
                "wrapper!(ghost);\n"
            )
            ghost.write_text('compile_error!("nested load is inert");\n')

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, ghost]
            )
            self.assertIn(ghost.resolve(), excluded)

    def test_nested_literal_mod_tokens_are_owned_by_outer_macro(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            ghost = source / "ghost.rs"
            lib.write_text(
                "macro_rules! noop { ($($tokens:tt)*) => {}; }\n"
                "macro_rules! wrapper { () => { noop!(mod ghost;); } }\n"
                "wrapper!();\n"
            )
            ghost.write_text('compile_error!("nested mod is inert");\n')

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, ghost]
            )
            self.assertIn(ghost.resolve(), excluded)

    def test_cyclic_wrapper_expansion_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "macro_rules! first { ($module:ident) => { second!($module); } }\n"
                "macro_rules! second { ($module:ident) => { first!($module); } }\n"
                "first!(behavior);\n"
            )
            behavior.write_text("fn should_not_be_guessed() {}\n")

            with self.assertRaisesRegex(
                ValueError, "cyclic module-generating macro expansion"
            ):
                behavior_inventory.external_test_module_paths(
                    repo_root, [lib, behavior]
                )

    def test_macro_uses_first_matching_arm(self) -> None:
        arms = behavior_inventory.rust_macro_arms(
            "($module:ident) => {}; ($other:ident) => { mod $other; };"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                arms, ["ghost"], {}, frozenset({"wrapper"})
            ),
            [],
        )

    def test_tt_fragment_preserves_first_matching_arm(self) -> None:
        no_op_first = behavior_inventory.rust_macro_arms(
            "($module:tt) => {}; ($module:ident) => { mod $module; };"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                no_op_first, ["ghost"], {}, frozenset({"wrapper"})
            ),
            [],
        )
        generating_first = behavior_inventory.rust_macro_arms(
            "($module:tt) => { mod $module; }; ($module:ident) => {};"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                generating_first, ["behavior"], {}, frozenset({"wrapper"})
            ),
            [("behavior", False)],
        )
        punctuation_first = behavior_inventory.rust_macro_arms(
            "($token:tt) => {}; (+) => { mod behavior; };"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                punctuation_first,
                [],
                {},
                frozenset({"wrapper"}),
                "+",
            ),
            [],
        )
        delimited_first = behavior_inventory.rust_macro_arms(
            "($token:tt) => {}; ({ value }) => { mod behavior; };"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                delimited_first,
                ["value"],
                {},
                frozenset({"wrapper"}),
                "{ value }",
            ),
            [],
        )
        decomposed_lifetime = "'be\N{COMBINING ACUTE ACCENT}"
        for literal in (
            '"x"',
            "42",
            "0xff_u8",
            "'x'",
            "'a",
            decomposed_lifetime,
            "1.",
            "&&",
            "..=",
        ):
            with self.subTest(literal=literal):
                literal_first = behavior_inventory.rust_macro_arms(
                    "($token:tt) => { mod behavior; }; () => {};"
                )
                self.assertEqual(
                    behavior_inventory.rust_macro_arm_modules(
                        literal_first,
                        [],
                        {},
                        frozenset({"wrapper"}),
                        literal,
                    ),
                    [("behavior", False)],
                )

        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "macro_rules! load {\n"
                "    ($token:tt) => { mod behavior; };\n"
                "}\n"
                'load!("x");\n'
            )
            behavior.write_text("fn shipped() {}\n")
            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_identifier_compatible_fragments_preserve_first_arm(self) -> None:
        for fragment in ("path", "ty", "expr", "pat", "stmt", "meta"):
            with self.subTest(fragment=fragment):
                arms = behavior_inventory.rust_macro_arms(
                    f"($module:{fragment}) => {{}}; "
                    "($module:ident) => { mod $module; };"
                )
                self.assertEqual(
                    behavior_inventory.rust_macro_arm_modules(
                        arms, ["ghost"], {}, frozenset({"wrapper"})
                    ),
                    [],
                )

    def test_unsupported_fragment_arm_selection_fails_closed(self) -> None:
        arms = behavior_inventory.rust_macro_arms(
            "($module:block) => {}; ($module:ident) => { mod $module; };"
        )
        with self.assertRaisesRegex(ValueError, "unsupported Rust macro fragment"):
            behavior_inventory.rust_macro_arm_modules(
                arms, ["ghost"], {}, frozenset({"wrapper"})
            )

    def test_non_identifier_unsupported_fragment_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "macro_rules! load {\n"
                "    ($value:block) => {};\n"
                "    () => { mod behavior; };\n"
                "}\n"
                "load!({});\n"
            )
            behavior.write_text('compile_error!("wrong arm");\n')

            with self.assertRaisesRegex(
                ValueError, "unsupported Rust macro fragment controls raw arm selection"
            ):
                behavior_inventory.external_test_module_paths(
                    repo_root, [lib, behavior]
                )

    def test_empty_visibility_fragment_fails_closed_before_fallthrough(self) -> None:
        arms = behavior_inventory.rust_macro_arms(
            "($visibility:vis behavior) => {}; (behavior) => { mod behavior; };"
        )
        with self.assertRaisesRegex(ValueError, "raw arm selection: vis"):
            behavior_inventory.rust_macro_arm_modules(
                arms,
                ["behavior"],
                {},
                frozenset({"load"}),
                "behavior",
            )

    def test_delegated_identifier_precedes_irrelevant_unsupported_arm(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "macro_rules! inner {\n"
                "    ($module:ident) => { mod $module; };\n"
                "    ($value:block) => {};\n"
                "}\n"
                "macro_rules! outer {\n"
                "    ($module:ident) => { inner!($module); }\n"
                "}\n"
                "outer!(behavior);\n"
            )
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_delegated_raw_substitution_preserves_structural_tokens(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "macro_rules! inner {\n"
                "    ($value:block) => {};\n"
                "    ($prefix:ident $module:ident) => { mod $module; };\n"
                "}\n"
                "macro_rules! outer {\n"
                "    ($module:ident) => { inner!({ prefix $module }); }\n"
                "}\n"
                "outer!(behavior);\n"
            )
            behavior.write_text('compile_error!("block arm is no-op");\n')

            with self.assertRaisesRegex(
                ValueError, "unsupported Rust macro fragment controls raw arm selection"
            ):
                behavior_inventory.external_test_module_paths(
                    repo_root, [lib, behavior]
                )

    def test_repetition_matcher_preserves_first_no_op_arm(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "macro_rules! load {\n"
                "    ($($module:ident),*) => {};\n"
                "    ($a:ident, $b:ident) => { mod behavior; };\n"
                "}\n"
                "load!(first, second);\n"
            )
            behavior.write_text('compile_error!("repetition arm wins");\n')

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertIn(behavior.resolve(), excluded)

            lib.write_text(
                "macro_rules! load {\n"
                "    ($($module:ident),*) => { mod behavior; };\n"
                "    ($a:ident, $b:ident) => {};\n"
                "}\n"
                "load!(first, second);\n"
            )
            with self.assertRaisesRegex(ValueError, "repetition"):
                behavior_inventory.external_test_module_paths(
                    repo_root, [lib, behavior]
                )

    def test_later_repetition_arm_precedes_fixed_arity_fallthrough(self) -> None:
        no_op_repetition = behavior_inventory.rust_macro_arms(
            "(single) => {}; "
            "($($module:ident),*) => {}; "
            "($a:ident, $b:ident, $c:ident) => { mod behavior; };"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                no_op_repetition,
                ["first", "second", "third"],
                {},
                frozenset({"load"}),
                "first, second, third",
            ),
            [],
        )
        effectful_repetition = behavior_inventory.rust_macro_arms(
            "(single) => {}; "
            "($($module:ident),*) => { mod behavior; }; "
            "($a:ident, $b:ident, $c:ident) => {};"
        )
        with self.assertRaisesRegex(ValueError, "repetition"):
            behavior_inventory.rust_macro_arm_modules(
                effectful_repetition,
                ["first", "second", "third"],
                {},
                frozenset({"load"}),
                "first, second, third",
            )

    def test_nonmatching_repetition_literals_fall_through(self) -> None:
        no_op_repetition = behavior_inventory.rust_macro_arms(
            "(only $($module:ident),*) => {}; "
            "($a:ident, $b:ident) => { mod behavior; };"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                no_op_repetition,
                ["first", "second"],
                {},
                frozenset({"load"}),
                "first, second",
            ),
            [("behavior", False)],
        )

    def test_repetition_cardinality_and_separator_must_match(self) -> None:
        plus_repetition = behavior_inventory.rust_macro_arms(
            "($($module:ident)+) => {}; () => { mod behavior; };"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                plus_repetition,
                [],
                {},
                frozenset({"load"}),
                "",
            ),
            [("behavior", False)],
        )
        unicode_repetition = behavior_inventory.rust_macro_arms(
            "($($module:ident),*) => {}; " "($module:ident) => { mod $module; };"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                unicode_repetition,
                ["béhavior"],
                {},
                frozenset({"load"}),
                "béhavior",
            ),
            [],
        )
        noncomposing = "a\N{COMBINING LONG SOLIDUS OVERLAY}"
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                unicode_repetition,
                [noncomposing],
                {},
                frozenset({"load"}),
                noncomposing,
            ),
            [],
        )
        decomposed = "be\N{COMBINING ACUTE ACCENT}havior"
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                unicode_repetition,
                [decomposed],
                {},
                frozenset({"load"}),
                decomposed,
            ),
            [],
        )
        arrow_repetition = behavior_inventory.rust_macro_arms(
            "($($module:ident)=>*) => {}; "
            "($a:ident => $b:ident) => { mod behavior; };"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                arrow_repetition,
                ["first", "second"],
                {},
                frozenset({"load"}),
                "first => second",
            ),
            [],
        )
        star_repetition = behavior_inventory.rust_macro_arms(
            "($($module:ident)*) => {}; () => { mod behavior; };"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                star_repetition,
                [],
                {},
                frozenset({"load"}),
                "",
            ),
            [],
        )
        comma_repetition = behavior_inventory.rust_macro_arms(
            "($($module:ident),*) => {}; " "($a:ident $b:ident) => { mod behavior; };"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                comma_repetition,
                ["first", "second"],
                {},
                frozenset({"load"}),
                "first second",
            ),
            [("behavior", False)],
        )

    def test_later_unsupported_arm_does_not_reject_supported_punctuation(self) -> None:
        arms = behavior_inventory.rust_macro_arms(
            "(special => $module:ident) => { mod $module; }; " "($value:block) => {};"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                arms,
                ["special", "behavior"],
                {},
                frozenset({"load"}),
                "special => behavior",
            ),
            [("behavior", False)],
        )

    def test_multitoken_fragment_does_not_fall_through(self) -> None:
        arms = behavior_inventory.rust_macro_arms(
            "($value:expr) => {}; " "($a:ident + $b:ident) => { mod behavior; };"
        )
        with self.assertRaisesRegex(ValueError, "raw arm selection"):
            behavior_inventory.rust_macro_arm_modules(
                arms,
                ["first", "second"],
                {},
                frozenset({"load"}),
                "first + second",
            )

        prefixed = behavior_inventory.rust_macro_arms(
            "(only $value:expr) => {}; " "($a:ident + $b:ident) => { mod behavior; };"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                prefixed,
                ["first", "second"],
                {},
                frozenset({"load"}),
                "first + second",
            ),
            [("behavior", False)],
        )
        suffixed = behavior_inventory.rust_macro_arms(
            "($value:expr only) => {}; " "($a:ident + $b:ident) => { mod behavior; };"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                suffixed,
                ["first", "second"],
                {},
                frozenset({"load"}),
                "first + second",
            ),
            [("behavior", False)],
        )

    def test_raw_identifier_does_not_match_ordinary_literal_arm(self) -> None:
        for name in ("foo", "type"):
            with self.subTest(name=name):
                arms = behavior_inventory.rust_macro_arms(
                    f"({name}) => {{}}; ($module:ident) => {{ mod behavior; }};"
                )
                self.assertEqual(
                    behavior_inventory.rust_macro_arm_modules(
                        arms,
                        [name],
                        {},
                        frozenset({"load"}),
                        f"r#{name}",
                    ),
                    [("behavior", False)],
                )

    def test_repetition_preserves_prefix_suffix_token_boundary(self) -> None:
        arms = behavior_inventory.rust_macro_arms(
            "(start$($module:ident),+end) => {}; "
            "(start $one:ident end) => { mod behavior; };"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                arms,
                ["start", "middle", "end"],
                {},
                frozenset({"load"}),
                "start middle end",
            ),
            [],
        )

        path_prefix = behavior_inventory.rust_macro_arms(
            "($path:path ; $($module:ident),*) => {}; "
            "($a:ident :: $b:ident ; $module:ident) => { mod behavior; };"
        )
        with self.assertRaisesRegex(ValueError, "repetition"):
            behavior_inventory.rust_macro_arm_modules(
                path_prefix,
                ["foo", "bar", "item"],
                {},
                frozenset({"load"}),
                "foo::bar ; item",
            )

    def test_delegated_raw_identifier_spelling_is_preserved(self) -> None:
        inner = behavior_inventory.rust_macro_arms(
            "(foo) => {}; ($module:ident) => { mod $module; };"
        )
        outer = behavior_inventory.rust_macro_arms(
            "($module:ident) => { inner!($module); };"
        )
        environment = {
            "inner": (False, inner),
            "outer": (False, outer),
        }
        self.assertEqual(
            behavior_inventory.rust_resolve_macro_modules(
                "outer", ["foo"], environment, raw_arguments="r#foo"
            ),
            [("foo", False)],
        )

    def test_macro_expanded_path_module_is_production_reachable(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            generated = source / "generated"
            generated.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            behavior = generated / "behavior.rs"
            lib.write_text(
                "macro_rules! load {\n"
                '    () => { #[path = "generated/behavior.rs"] mod behavior; };\n'
                "}\n"
                "load!();\n"
            )
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

            lib.write_text(
                "macro_rules! load {\n"
                "    () => {\n"
                '        #[cfg_attr(not(test), path = "generated/behavior.rs")]\n'
                "        mod behavior;\n"
                "    };\n"
                "}\n"
                "load!();\n"
            )
            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)
        effectful_repetition = behavior_inventory.rust_macro_arms(
            "($($module:ident),* only) => { mod ghost; }; "
            "($a:ident, $b:ident) => { mod behavior; };"
        )
        self.assertEqual(
            behavior_inventory.rust_macro_arm_modules(
                effectful_repetition,
                ["first", "second"],
                {},
                frozenset({"load"}),
                "first, second",
            ),
            [("behavior", False)],
        )

    def test_macro_expansion_target_carries_cfg_test_state(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            direct = source / "direct.rs"
            delegated = source / "delegated.rs"
            lib.write_text(
                "macro_rules! direct_load {\n"
                "    ($module:ident) => { #[cfg(test)] mod $module; }\n"
                "}\n"
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
                "macro_rules! delegated_load {\n"
                "    ($module:ident) => { #[cfg(test)] load!($module); }\n"
                "}\n"
                "direct_load!(direct);\n"
                "delegated_load!(delegated);\n"
            )
            direct.write_text('compile_error!("test-only direct");\n')
            delegated.write_text('compile_error!("test-only delegated");\n')

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, direct, delegated]
            )
            self.assertIn(direct.resolve(), excluded)
            self.assertIn(delegated.resolve(), excluded)

    def test_test_only_macro_in_production_module_is_not_imported(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            macros = source / "macros.rs"
            ghost = source / "ghost.rs"
            lib.write_text("#[macro_use]\nmod macros;\nload!(ghost);\n")
            macros.write_text(
                "#[cfg(test)]\n"
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
            )
            ghost.write_text('compile_error!("not shipped");\n')

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, macros, ghost]
            )
            self.assertIn(ghost.resolve(), excluded)

    def test_macro_use_import_is_visible_only_after_declaration(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            no_op = source / "no_op.rs"
            generator = source / "generator.rs"
            ghost = source / "ghost.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "#[macro_use]\nmod no_op;\n"
                "load!(ghost);\n"
                "#[macro_use]\nmod generator;\n"
                "load!(behavior);\n"
            )
            no_op.write_text("macro_rules! load { ($module:ident) => {}; }\n")
            generator.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
            )
            ghost.write_text('compile_error!("not shipped");\n')
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, no_op, generator, ghost, behavior]
            )
            self.assertIn(ghost.resolve(), excluded)
            self.assertNotIn(behavior.resolve(), excluded)

    def test_standard_use_import_reaches_exported_module_macro(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\nedition = "2024"\n'
            )
            lib = source / "lib.rs"
            macros = source / "macros.rs"
            behavior = source / "behavior.rs"
            renamed = source / "renamed.rs"
            grouped = source / "grouped.rs"
            grouped_renamed = source / "grouped_renamed.rs"
            lib.write_text(
                "mod macros;\n"
                "use macros::load;\n"
                "use crate::macros::load as renamed_load;\n"
                "use macros::{load as grouped_load, load as grouped_renamed_load};\n"
                "load!(behavior);\n"
                "renamed_load!(renamed);\n"
                "grouped_load!(grouped);\n"
                "grouped_renamed_load!(grouped_renamed);\n"
            )
            macros.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
                "pub(crate) use load;\n"
            )
            behavior.write_text("fn shipped() {}\n")
            renamed.write_text("fn renamed_shipped() {}\n")
            grouped.write_text("fn grouped_shipped() {}\n")
            grouped_renamed.write_text("fn grouped_renamed_shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root,
                [lib, macros, behavior, renamed, grouped, grouped_renamed],
            )
            self.assertNotIn(behavior.resolve(), excluded)
            self.assertNotIn(renamed.resolve(), excluded)
            self.assertNotIn(grouped.resolve(), excluded)
            self.assertNotIn(grouped_renamed.resolve(), excluded)

    def test_macro_export_reaches_crate_root_invocation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\nedition = "2024"\n'
            )
            lib = source / "lib.rs"
            macros = source / "macros.rs"
            behavior = source / "behavior.rs"
            lib.write_text("mod macros;\ncrate::make_mod!(behavior);\n")
            macros.write_text(
                "#[macro_export]\n"
                "macro_rules! make_mod { ($module:ident) => { mod $module; } }\n"
            )
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, macros, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_cfg_alternative_macro_definitions_union_production_modules(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            unix = source / "unix_impl.rs"
            windows = source / "windows_impl.rs"
            lib.write_text(
                "#[cfg(unix)]\n"
                "macro_rules! load { () => { mod unix_impl; } }\n"
                "#[cfg(windows)]\n"
                "macro_rules! load { () => { mod windows_impl; } }\n"
                "load!();\n"
            )
            unix.write_text("fn unix_behavior() {}\n")
            windows.write_text("fn windows_behavior() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, unix, windows]
            )
            self.assertNotIn(unix.resolve(), excluded)
            self.assertNotIn(windows.resolve(), excluded)

    def test_nested_use_path_reaches_exported_module_macro(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            nested = source / "outer"
            nested.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\nedition = "2024"\n'
            )
            lib = source / "lib.rs"
            outer = source / "outer.rs"
            inner = nested / "inner.rs"
            behavior = source / "behavior.rs"
            lib.write_text("mod outer;\nuse outer::inner::load;\nload!(behavior);\n")
            outer.write_text("pub mod inner;\n")
            inner.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
                "pub(crate) use load;\n"
            )
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, outer, inner, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_path_module_target_resolves_imported_macro(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            support = source / "support"
            support.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            macros = support / "macros.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                '#[path = "support/macros.rs"] mod macros;\n'
                "use macros::load;\n"
                "load!();\n"
            )
            macros.write_text(
                "macro_rules! load { () => { mod behavior; } }\n"
                "pub(crate) use load;\n"
            )
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, macros, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_nested_use_tree_and_unicode_macro_name_are_resolved(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            macros = source / "macros.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "mod macros;\n" "use crate::{macros::chargé};\n" "chargé!();\n"
            )
            macros.write_text(
                "macro_rules! chargé { () => { mod behavior; } }\n"
                "pub(crate) use chargé;\n"
            )
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, macros, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_inline_module_macro_import_is_resolved(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            behavior = source / "behavior.rs"
            lib.write_text(
                "mod macros {\n"
                "    macro_rules! load { () => { mod behavior; } }\n"
                "    pub(crate) use load;\n"
                "}\n"
                "use macros::load;\n"
                "load!();\n"
            )
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_unknown_macro_retains_explicit_module_tokens(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            platform = source / "platform.rs"
            lib.write_text("cfg_if! { if #[cfg(unix)] { mod platform; } }\n")
            platform.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, platform]
            )
            self.assertNotIn(platform.resolve(), excluded)

    def test_included_macro_is_visible_only_after_include(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            definitions = source / "defs.inc"
            before = source / "before.rs"
            after = source / "after.rs"
            lib.write_text(
                "macro_rules! load { ($module:ident) => {}; }\n"
                "load!(before);\n"
                'include!("defs.inc");\n'
                "load!(after);\n"
            )
            definitions.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
            )
            before.write_text('compile_error!("not shipped");\n')
            after.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, definitions, before, after]
            )
            self.assertIn(before.resolve(), excluded)
            self.assertNotIn(after.resolve(), excluded)

    def test_outer_macro_is_visible_inside_nested_includes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            outer = source / "outer.inc"
            inner = source / "inner.inc"
            behavior = source / "behavior.rs"
            lib.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
                'include!("outer.inc");\n'
            )
            outer.write_text('include!("inner.inc");\n')
            inner.write_text("load!(behavior);\n")
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, outer, inner, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_nested_include_macro_is_visible_after_outer_include(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            outer = source / "outer.inc"
            inner = source / "inner.inc"
            behavior = source / "behavior.rs"
            lib.write_text('include!("outer.inc");\nload!(behavior);\n')
            outer.write_text('include!("inner.inc");\n')
            inner.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
            )
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, outer, inner, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_macro_imported_inside_include_is_visible_after_include(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo_root = pathlib.Path(directory)
            crate = repo_root / "crates/nuxie"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "nuxie"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            included = source / "defs.inc"
            macros = source / "macros.rs"
            behavior = source / "behavior.rs"
            lib.write_text('include!("defs.inc");\nload!(behavior);\n')
            included.write_text("#[macro_use]\nmod macros;\n")
            macros.write_text(
                "macro_rules! load { ($module:ident) => { mod $module; } }\n"
            )
            behavior.write_text("fn shipped() {}\n")

            excluded = behavior_inventory.external_test_module_paths(
                repo_root, [lib, included, macros, behavior]
            )
            self.assertNotIn(behavior.resolve(), excluded)

    def test_upstream_worktree_must_be_clean(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True)
            source = root / "src" / "owner.cpp"
            source.parent.mkdir()
            source.write_text("int owner() { return 1; }\n")
            subprocess.run(["git", "add", "src/owner.cpp"], cwd=root, check=True)
            subprocess.run(
                [
                    "git",
                    "-c",
                    "user.name=Runtime Inventory",
                    "-c",
                    "user.email=inventory@example.invalid",
                    "commit",
                    "-qm",
                    "fixture",
                ],
                cwd=root,
                check=True,
            )
            self.assertTrue(behavior_inventory.git_worktree_clean(root))
            build = root / "build"
            build.mkdir()
            (build / "artifact.bin").write_bytes(b"ignored by inventory scope")
            self.assertTrue(behavior_inventory.git_worktree_clean(root))
            source.write_text("int owner() { return 2; }\n")
            self.assertFalse(behavior_inventory.git_worktree_clean(root))
            subprocess.run(
                ["git", "checkout", "--", "src/owner.cpp"], cwd=root, check=True
            )
            include = root / "include" / "rive"
            include.mkdir(parents=True)
            untracked = include / "untracked.hpp"
            untracked.write_text("int untracked;\n")
            self.assertFalse(behavior_inventory.git_worktree_clean(root))
            untracked.unlink()
            (root / ".gitignore").write_text("/src/ignored.cpp\n/build/\n")
            (root / "src" / "ignored.cpp").write_text("int ignored() {}\n")
            self.assertFalse(behavior_inventory.git_worktree_clean(root))

    def test_named_adaptation_rule_requires_live_approval(self) -> None:
        additions = {
            "addition": [
                {
                    "path": "crates/demo/src/adapter.rs",
                    "category": "product-host",
                }
            ]
        }
        manifest = {"file": []}
        self.assertIn(
            "stale=['crates/demo/src/adapter.rs']",
            "\n".join(
                behavior_inventory.adaptation_policy_coverage_errors(
                    additions,
                    manifest,
                    {"crates/demo/src/adapter.rs"},
                )
            ),
        )

    def test_every_shipped_runtime_support_crate_includes_apple_extension(self) -> None:
        self.assertIn("nux-apple-product-extension", behavior_inventory.RUST_CRATES)

    def test_production_modules_named_test_or_tests_remain_shipped(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            crate = root / "crates/demo"
            source = crate / "src"
            bin_dir = source / "bin"
            bin_dir.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "demo"\nversion = "0.1.0"\n'
            )
            lib = source / "lib.rs"
            module = source / "tests.rs"
            binary = bin_dir / "test.rs"
            lib.write_text("mod tests;\n")
            module.write_text("pub fn production_module() {}\n")
            binary.write_text("fn main() {}\n")
            self.assertEqual(
                set(),
                behavior_inventory.external_test_module_paths(
                    root, [lib, module, binary]
                ),
            )

    def test_external_cfg_test_module_files_are_discovered(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "crates/demo/src"
            source.mkdir(parents=True)
            (source / "lib.rs").write_text(
                "#[cfg(test)]\nmod arbitrary_oracle;\n"
                '#[cfg(feature = "test-support")]\nmod shipped_support;\n'
            )
            oracle = source / "arbitrary_oracle.rs"
            fixtures = source / "arbitrary_oracle/fixtures.rs"
            fixtures.parent.mkdir()
            support = source / "shipped_support.rs"
            oracle.write_text("mod fixtures;\nfn oracle() {}\n")
            fixtures.write_text("fn fixture() {}\n")
            support.write_text("fn support() {}\n")
            self.assertEqual(
                {oracle.resolve(), fixtures.resolve()},
                behavior_inventory.external_test_module_paths(
                    root, [source / "lib.rs", oracle, fixtures, support]
                ),
            )

    def test_include_files_follow_production_and_test_reachability(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "crates/demo/src"
            source.mkdir(parents=True)
            lib = source / "lib.rs"
            shipped = source / "shipped.rs"
            braced = source / "braced.rs"
            bracketed = source / "bracketed.rs"
            concatenated = source / "concatenated.rs"
            fixture = source / "fixture.rs"
            lib.write_text(
                'include!("shipped.rs");\n'
                'include! { "braced.rs" }\n'
                'include! [ "bracketed.rs" ]\n'
                'include!(concat!(/* nested /* path */ trivia */ "concat",\n'
                '    "enated" // stem\n'
                '    , /* suffix */ ".rs"));\n'
                '#[cfg(test)] include!(r#"fixture.rs"#);\n'
            )
            shipped.write_text("fn shipped() {}\n")
            braced.write_text("fn braced() {}\n")
            bracketed.write_text("fn bracketed() {}\n")
            concatenated.write_text("fn concatenated() {}\n")
            fixture.write_text("fn fixture() {}\n")
            self.assertEqual(
                {fixture.resolve()},
                behavior_inventory.external_test_module_paths(
                    root,
                    [lib, shipped, braced, bracketed, concatenated, fixture],
                ),
            )

    def test_included_file_resolves_child_modules_from_its_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "crates/demo/src"
            included_dir = source / "generated"
            included_dir.mkdir(parents=True)
            lib = source / "lib.rs"
            included = included_dir / "items.rs"
            child = included_dir / "child.rs"
            lib.write_text('include!("generated/items.rs");\n')
            included.write_text("mod child;\n")
            child.write_text("fn child() {}\n")
            self.assertEqual(
                set(),
                behavior_inventory.external_test_module_paths(
                    root, [lib, included, child]
                ),
            )

    def test_unresolved_repository_include_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            crate = root / "crates/demo"
            source = crate / "src"
            source.mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "demo"\nversion = "0.1.0"\n'
            )
            (crate / "build.rs").write_text("fn main() {}\n")
            lib = source / "lib.rs"
            lib.write_text(
                'include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/part.rs"));\n'
            )
            with self.assertRaisesRegex(ValueError, "unresolved repository include"):
                behavior_inventory.external_test_module_paths(root, [lib])

            lib.write_text(
                'include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"));\n'
            )
            with self.assertRaisesRegex(ValueError, "not declared"):
                behavior_inventory.external_test_module_paths(root, [lib])
            with mock.patch.dict(
                behavior_inventory.RUST_GENERATOR_OUTPUTS,
                {
                    "crates/demo/build.rs": [
                        "OUT_DIR/runtime_objects.rs",
                        "OUT_DIR/part.rs",
                    ]
                },
            ):
                self.assertEqual(
                    set(), behavior_inventory.external_test_module_paths(root, [lib])
                )
                for generated in (
                    'include!(concat![env!(/* build */ "OUT_DIR"), "/part.rs"]);\n',
                    'include!(concat!{env!("OUT_DIR"), /* suffix */ "/part.rs"});\n',
                    'include!(concat!(env!["OUT_DIR"], "/part.rs"));\n',
                    'include!(concat!(env!("OUT_DIR", "build output unavailable"), '
                    '"/part.rs"));\n',
                ):
                    with self.subTest(generated=generated):
                        lib.write_text(generated)
                        self.assertEqual(
                            set(),
                            behavior_inventory.external_test_module_paths(root, [lib]),
                        )

            part = source / "part.rs"
            part.write_text("fn part() {}\n")
            lib.write_text(
                'include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/part.rs")); '
                '// env!("OUT_DIR")\n'
            )
            with self.assertRaisesRegex(ValueError, "unresolved repository include"):
                behavior_inventory.external_test_module_paths(root, [lib, part])

            lib.write_text(
                '#[cfg(test)] include!(concat!(env!("CARGO_MANIFEST_DIR"), '
                '"/src/test_fixture.rs"));\n'
            )
            self.assertEqual(
                set(), behavior_inventory.external_test_module_paths(root, [lib])
            )

            for test_only in (
                '#[cfg(test)] include!("missing_fixture.rs");\n',
                '#[cfg(test)] include!(concat!("missing", "_fixture.rs"));\n',
            ):
                with self.subTest(test_only=test_only):
                    lib.write_text(test_only)
                    self.assertEqual(
                        set(),
                        behavior_inventory.external_test_module_paths(root, [lib]),
                    )

    def test_production_capable_external_cfg_module_is_not_excluded(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "crates/demo/src"
            source.mkdir(parents=True)
            (source / "lib.rs").write_text(
                '#[cfg(any(test, all(feature = "a", not(feature = "b"))))]\n'
                "mod oracle;\n"
            )
            oracle = source / "oracle.rs"
            oracle.write_text("fn shipped_oracle() {}\n")
            self.assertEqual(
                set(),
                behavior_inventory.external_test_module_paths(
                    root, [source / "lib.rs", oracle]
                ),
            )

    def test_external_module_with_a_production_declaration_is_not_excluded(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "crates/demo/src"
            source.mkdir(parents=True)
            (source / "lib.rs").write_text(
                "#[cfg(test)]\nmod shared;\n" "#[cfg(not(test))]\nmod shared;\n"
            )
            shared = source / "shared.rs"
            shared.write_text("fn shipped() {}\n")
            self.assertEqual(
                set(),
                behavior_inventory.external_test_module_paths(
                    root, [source / "lib.rs", shared]
                ),
            )

    def test_path_module_with_production_declaration_is_not_excluded(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "crates/demo/src"
            source.mkdir(parents=True)
            shared = source / "shared.rs"
            shared.write_text("fn shipped() {}\n")
            for literal in (
                '"shared.rs"',
                r'"shared\x2ers"',
                'r"shared.rs"',
                'r##"shared.rs"##',
            ):
                with self.subTest(literal=literal):
                    (source / "lib.rs").write_text(
                        "#[cfg(test)]\nmod shared;\n"
                        "#[cfg(not(test))]\n"
                        f"#[path = {literal}]\n"
                        "mod production_shared;\n"
                    )
                    self.assertEqual(
                        set(),
                        behavior_inventory.external_test_module_paths(
                            root, [source / "lib.rs", shared]
                        ),
                    )

            quoted = source / "foo'bar.rs"
            quoted.write_text("fn shipped_quote() {}\n")
            (source / "lib.rs").write_text(
                "#[cfg(test)]\n"
                '#[path = "foo\\\'bar.rs"]\n'
                "mod test_shared;\n"
                "#[cfg(not(test))]\n"
                '#[path = "foo\\\'bar.rs"]\n'
                "mod production_shared;\n"
            )
            self.assertNotIn(
                quoted.resolve(),
                behavior_inventory.external_test_module_paths(
                    root, [source / "lib.rs", shared, quoted]
                ),
            )

            (source / "lib.rs").write_text(
                "#[cfg(test)]\nmod shared;\n"
                "#[cfg(not(test))]\n"
                '#[path /* key */ = /* outer /* nested */ tail */ r"shared.rs"]\n'
                "mod production_shared;\n"
            )
            self.assertNotIn(
                shared.resolve(),
                behavior_inventory.external_test_module_paths(
                    root, [source / "lib.rs", shared]
                ),
            )

    def test_path_module_with_unicode_identifier_is_not_excluded(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "crates/demo/src"
            inline = source / "naïve"
            inline.mkdir(parents=True)
            owner = source / "lib.rs"
            owner.write_text(
                '#[path = "behavior.rs"]\nmod café;\n'
                'mod naïve { #[path = "nested.rs"] mod nested; }\n'
            )
            behavior = source / "behavior.rs"
            behavior.write_text("pub fn shipped() {}\n")
            nested = inline / "nested.rs"
            nested.write_text("pub fn nested_shipped() {}\n")

            self.assertTrue(
                {behavior.resolve(), nested.resolve()}.isdisjoint(
                    behavior_inventory.external_test_module_paths(
                        root, [owner, behavior, nested]
                    )
                )
            )

    def test_path_inside_inline_module_uses_inline_module_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "crates/demo/src"
            inline = source / "inline"
            inline.mkdir(parents=True)
            shared = inline / "shared.rs"
            shared.write_text("fn shipped() {}\n")
            owner = source / "lib.rs"
            owner.write_text(
                "#[cfg(test)]\n"
                '#[path = "inline/shared.rs"]\n'
                "mod test_shared;\n"
                "mod inline {\n"
                "    #[cfg(not(test))]\n"
                '    #[path = "shared.rs"]\n'
                "    mod production_shared;\n"
                "}\n"
            )
            self.assertNotIn(
                shared.resolve(),
                behavior_inventory.external_test_module_paths(root, [owner, shared]),
            )

            owner.write_text(
                "#[cfg(test)]\n"
                '#[path = "inline/shared.rs"]\n'
                "mod test_shared;\n"
                "mod r#inline {\n"
                "    #[cfg(not(test))]\n"
                '    #[path = "shared.rs"]\n'
                "    mod r#production_shared;\n"
                "}\n"
            )
            self.assertNotIn(
                shared.resolve(),
                behavior_inventory.external_test_module_paths(root, [owner, shared]),
            )

    def test_path_inside_nonroot_inline_module_uses_module_stem(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "crates/demo/src"
            shared = source / "outer/inline/shared.rs"
            shared.parent.mkdir(parents=True)
            shared.write_text("fn shipped() {}\n")
            lib = source / "lib.rs"
            outer = source / "outer.rs"
            lib.write_text(
                "#[cfg(test)]\n"
                '#[path = "outer/inline/shared.rs"]\n'
                "mod test_shared;\n"
                "mod outer;\n"
            )
            outer.write_text(
                "mod inline {\n"
                "    #[cfg(not(test))]\n"
                '    #[path = "shared.rs"]\n'
                "    mod production_shared;\n"
                "}\n"
            )
            self.assertNotIn(
                shared.resolve(),
                behavior_inventory.external_test_module_paths(
                    root, [lib, outer, shared]
                ),
            )

    def test_path_loaded_owner_does_not_contribute_its_file_stem(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "crates/demo/src"
            shared = source / "inline/shared.rs"
            shared.parent.mkdir(parents=True)
            shared.write_text("fn shipped() {}\n")
            lib = source / "lib.rs"
            custom = source / "custom.rs"
            lib.write_text(
                "#[cfg(test)]\n"
                '#[path = "inline/shared.rs"]\n'
                "mod test_shared;\n"
                '#[path = "custom.rs"]\n'
                "mod outer;\n"
            )
            custom.write_text(
                "mod inline {\n"
                "    #[cfg(not(test))]\n"
                '    #[path = "shared.rs"]\n'
                "    mod production_shared;\n"
                "}\n"
            )
            self.assertNotIn(
                shared.resolve(),
                behavior_inventory.external_test_module_paths(
                    root, [lib, custom, shared]
                ),
            )

    def test_file_mounted_normally_and_by_path_keeps_both_module_bases(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "crates/demo/src"
            ordinary_shared = source / "outer/inline/shared.rs"
            path_shared = source / "inline/shared.rs"
            ordinary_shared.parent.mkdir(parents=True)
            path_shared.parent.mkdir(parents=True)
            ordinary_shared.write_text("fn ordinary() {}\n")
            path_shared.write_text("fn path_mounted() {}\n")
            lib = source / "lib.rs"
            outer = source / "outer.rs"
            lib.write_text(
                "mod outer;\n"
                "#[cfg(test)]\n"
                '#[path = "outer.rs"]\n'
                "mod test_outer;\n"
                "#[cfg(test)]\n"
                '#[path = "outer/inline/shared.rs"]\n'
                "mod test_shared;\n"
            )
            outer.write_text(
                "mod inline {\n" '    #[path = "shared.rs"]\n' "    mod child;\n" "}\n"
            )
            excluded = behavior_inventory.external_test_module_paths(
                root, [lib, outer, ordinary_shared, path_shared]
            )
            self.assertNotIn(ordinary_shared.resolve(), excluded)
            self.assertIn(path_shared.resolve(), excluded)

    def test_cargo_binary_and_explicit_target_roots_retain_modules(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            crate = root / "crates/demo"
            source = crate / "src"
            binary_dir = source / "bin"
            binary_dir.mkdir(parents=True)
            shared = binary_dir / "shared.rs"
            shared.write_text("fn shipped() {}\n")
            lib = source / "lib.rs"
            binary = binary_dir / "tool.rs"
            custom = source / "custom_tool.rs"
            custom_shared = source / "custom_shared.rs"
            lib.write_text(
                "#[cfg(test)]\n"
                '#[path = "bin/shared.rs"]\n'
                "mod test_shared;\n"
                "#[cfg(test)]\n"
                '#[path = "custom_shared.rs"]\n'
                "mod test_custom_shared;\n"
            )
            binary.write_text("mod shared;\n")
            custom.write_text("mod custom_shared;\n")
            custom_shared.write_text("fn custom_shipped() {}\n")
            (crate / "Cargo.toml").write_text(
                '[[bin]]\nname = "custom"\npath = "src/custom_tool.rs"\n'
            )
            files = [lib, binary, shared, custom, custom_shared]
            roots = behavior_inventory.rust_crate_roots(root, files)
            self.assertIn(binary.resolve(), roots)
            self.assertIn(custom.resolve(), roots)
            excluded = behavior_inventory.external_test_module_paths(root, files)
            self.assertNotIn(shared.resolve(), excluded)
            self.assertNotIn(custom_shared.resolve(), excluded)

    def test_cfg_attr_path_keeps_production_and_excludes_test_alternative(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "crates/demo/src"
            source.mkdir(parents=True)
            lib = source / "lib.rs"
            production = source / "imp.rs"
            oracle = source / "oracle.rs"
            production.write_text("fn production() {}\n")
            oracle.write_text("fn test_only() {}\n")
            lib.write_text(
                '#[cfg_attr(test, path /* key */ = r"oracle.rs")]\nmod imp;\n'
            )
            excluded = behavior_inventory.external_test_module_paths(
                root, [lib, production, oracle]
            )
            self.assertNotIn(production.resolve(), excluded)
            self.assertIn(oracle.resolve(), excluded)
            lib.write_text(
                '#[cfg_attr(feature = "x", cfg_attr(test, path = "oracle.rs"))]\n'
                "mod imp;\n"
            )
            excluded = behavior_inventory.external_test_module_paths(
                root, [lib, production, oracle]
            )
            self.assertNotIn(production.resolve(), excluded)
            self.assertIn(oracle.resolve(), excluded)

    def test_nested_module_named_bin_is_not_a_cargo_target_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            crate = root / "crates/demo"
            source = crate / "src"
            nested = source / "internal/bin"
            nested.mkdir(parents=True)
            lib = source / "lib.rs"
            internal = source / "internal.rs"
            bin_module = source / "internal/bin.rs"
            oracle = nested / "oracle.rs"
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "demo"\nversion = "0.1.0"\n'
            )
            lib.write_text("mod internal;\n")
            internal.write_text("mod bin;\n")
            bin_module.write_text("#[cfg(test)]\nmod oracle;\n")
            oracle.write_text("fn test_only() {}\n")
            files = [lib, internal, bin_module, oracle]
            self.assertNotIn(
                oracle.resolve(), behavior_inventory.rust_crate_roots(root, files)
            )
            self.assertIn(
                oracle.resolve(),
                behavior_inventory.external_test_module_paths(root, files),
            )

    def test_nested_lib_or_main_file_is_not_a_cargo_target_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            crate = root / "crates/demo"
            source = crate / "src"
            nested = source / "internal"
            nested.mkdir(parents=True)
            lib = source / "lib.rs"
            internal = source / "internal.rs"
            fixture = nested / "lib.rs"
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "demo"\nversion = "0.1.0"\n'
            )
            lib.write_text("mod internal;\n")
            internal.write_text("#[cfg(test)]\nmod lib;\n")
            fixture.write_text("fn test_only() {}\n")
            files = [lib, internal, fixture]
            self.assertNotIn(
                fixture.resolve(), behavior_inventory.rust_crate_roots(root, files)
            )
            self.assertIn(
                fixture.resolve(),
                behavior_inventory.external_test_module_paths(root, files),
            )

    def test_cargo_target_overrides_replace_disabled_conventions(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            crate = root / "crates/demo"
            source = crate / "src"
            bin_dir = source / "bin"
            bin_dir.mkdir(parents=True)
            conventional_lib = source / "lib.rs"
            conventional_main = source / "main.rs"
            custom_lib = source / "custom.rs"
            named_bin = bin_dir / "tool.rs"
            for path in (conventional_lib, conventional_main, custom_lib, named_bin):
                path.write_text("fn target() {}\n")
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "demo"\nversion = "0.1.0"\n'
                "autolib = false\nautobins = false\n"
                '[lib]\npath = "src/custom.rs"\n'
                '[[bin]]\nname = "tool"\n'
            )
            roots = behavior_inventory.rust_crate_roots(
                root, [conventional_lib, conventional_main, custom_lib, named_bin]
            )
            self.assertEqual({custom_lib.resolve(), named_bin.resolve()}, roots)

    def test_objective_cpp_is_an_implementation_source(self) -> None:
        self.assertEqual(
            "implementation",
            behavior_inventory.cpp_file_classification("src/text/font_hb_apple.mm", []),
        )
        self.assertEqual(
            "src/text/font_hb.cpp",
            behavior_inventory.CPP_OWNER_ALIASES["src/text/font_hb_apple.mm"],
        )

    def test_workspace_crate_scope_is_derived_for_fail_closed_validation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "Cargo.toml").write_text(
                '[workspace]\nmembers = ["crates/runtime", "tools/helper"]\n'
            )
            self.assertEqual(
                {"runtime"}, behavior_inventory.workspace_shipped_crates(root)
            )

    def test_approved_adaptation_sources_all_have_named_item_policy_rules(self) -> None:
        additions = behavior_inventory.tomllib.loads(
            (pathlib.Path(__file__).parents[2] / "rust-additions.toml").read_text()
        )
        approved = {
            row["path"]
            for row in additions["addition"]
            if row["category"] in behavior_inventory.ADAPTATION_CATEGORIES
        }
        self.assertEqual(
            approved,
            approved & set(behavior_inventory.NAMED_ADAPTATION_PATH_RULES),
        )

    def test_itemless_adaptation_rules_still_publish_seam_policies(self) -> None:
        with mock.patch.dict(behavior_inventory.NAMED_EXTENSION_RULES, {}, clear=True):
            policies = behavior_inventory.compact_seam_policies([])
        by_adaptation = {policy["adaptation"]: policy for policy in policies}
        self.assertEqual(
            ["crates/nuxie-runtime/src/focus.rs"],
            by_adaptation["host-focus-bridge"]["rust_paths"],
        )
        self.assertEqual(
            ["crates/nuxie-audio/src/lib.rs"],
            by_adaptation["audio-host-backend"]["rust_paths"],
        )
        self.assertEqual(
            ["crates/nuxie-scripting/src/lib.rs"],
            by_adaptation["script-runtime-crate-boundary"]["rust_paths"],
        )

    def test_named_evidence_selectors_resolve(self) -> None:
        repo_root = pathlib.Path(__file__).parents[2]
        errors = behavior_inventory.validate_configuration(repo_root)
        self.assertFalse(
            [error for error in errors if "evidence selector" in error], errors
        )

    def test_inventory_diff_reports_owner_family_and_change_kind(self) -> None:
        expected = {
            "cpp_members": [
                {
                    "id": "cpp:src/shape.cpp:Shape::draw#1",
                    "owner_family": "shapes",
                    "content_sha256": "old",
                },
                {
                    "id": "cpp:src/shape.cpp:Shape::opacity#1",
                    "owner_family": "shapes",
                    "content_sha256": "same",
                },
            ],
            "rust_items": [],
        }
        actual = {
            "cpp_members": [
                {
                    "id": "cpp:src/shape.cpp:Shape::draw#1",
                    "owner_family": "shapes",
                    "content_sha256": "new",
                },
                {
                    "id": "cpp:src/shape.cpp:Shape::advance#1",
                    "owner_family": "shapes",
                    "content_sha256": "added",
                },
            ],
            "rust_items": [],
        }
        errors = behavior_inventory.inventory_differences(expected, actual)
        joined = "\n".join(errors)
        self.assertIn("[shapes] changed", joined)
        self.assertIn("[shapes] new", joined)
        self.assertIn("[shapes] removed", joined)

    def test_source_hash_backstop_detects_behavior_outside_scanned_items(self) -> None:
        expected = {
            "cpp_members": [],
            "rust_items": [],
            "rust_files": [{"path": "crates/demo/src/lib.rs", "sha256": "before"}],
        }
        actual = {
            "cpp_members": [],
            "rust_items": [],
            "rust_files": [{"path": "crates/demo/src/lib.rs", "sha256": "after"}],
        }
        self.assertIn(
            "[demo] changed Rust source: crates/demo/src/lib.rs",
            behavior_inventory.inventory_differences(expected, actual),
        )

    def test_source_hash_backstop_ignores_test_required_body_edits(self) -> None:
        before = "#[cfg(test)] mod tests { fn fixture() { assert!(true); } }\n"
        after = "#[cfg(test)] mod tests { fn fixture() { assert!(false); } }\n"
        self.assertEqual(
            behavior_inventory.sha256_text(
                behavior_inventory.rust_shipped_source(before)
            ),
            behavior_inventory.sha256_text(
                behavior_inventory.rust_shipped_source(after)
            ),
        )

    def test_scope_and_schema_changes_fail_even_without_source_records(self) -> None:
        expected = {
            "schema": behavior_inventory.SCHEMA,
            "scope": {"rust_crates": ["demo"]},
            "summary": {},
            "cpp_members": [],
            "rust_items": [],
        }
        actual = {
            **expected,
            "scope": {"rust_crates": ["demo", "new-runtime-crate"]},
        }
        self.assertIn(
            "[inventory] changed scope",
            "\n".join(behavior_inventory.inventory_differences(expected, actual)),
        )

    def test_duplicate_stable_ids_fail(self) -> None:
        inventory = {
            "cpp_members": [],
            "rust_items": [
                {"id": "rust:a::f#1", "owner_family": "a"},
                {"id": "rust:a::f#1", "owner_family": "a"},
            ],
        }
        self.assertIn(
            "duplicate",
            "\n".join(behavior_inventory.validate_inventory(inventory)),
        )

    def test_behavioral_macro_records_are_validated_fail_closed(self) -> None:
        macro = {
            "id": "macro:ACTION@0123456789abcdef",
            "name": "ACTION",
            "start_line": 1,
            "end_line": 1,
            "content_sha256": "0" * 64,
        }
        inventory = {
            "cpp_files": [
                {
                    "path": "include/rive/macros.hpp",
                    "classification": "declaration-only",
                    "behavioral_macro_count": 2,
                    "behavioral_macros": [macro, macro],
                }
            ]
        }
        errors = "\n".join(behavior_inventory.validate_inventory(inventory))
        self.assertIn("duplicate behavioral macro id", errors)
        self.assertIn("behavioral macros classified declaration-only", errors)

        inventory["cpp_files"][0]["behavioral_macros"] = [
            {**macro, "content_sha256": "not-a-hash"}
        ]
        errors = "\n".join(behavior_inventory.validate_inventory(inventory))
        self.assertIn("behavioral macro count mismatch", errors)
        self.assertIn("malformed behavioral macro record", errors)

    def test_checked_snapshot_duplicate_ids_fail_before_comparison(self) -> None:
        member = {
            "id": "cpp:src/demo.cpp:Demo::run@1",
            "owner_policy": "cpp-owner:demo",
            "correspondence": "mapped",
        }
        inventory = {
            "cpp_members": [member],
            "cpp_owner_policies": [{"id": "cpp-owner:demo", "disposition": "mapped"}],
            "rust_items": [],
        }
        with tempfile.TemporaryDirectory() as directory:
            snapshot = pathlib.Path(directory) / "inventory.json"
            snapshot.write_text(
                json.dumps({**inventory, "cpp_members": [member, member]})
            )
            errors = behavior_inventory.check_snapshot(snapshot, inventory)
        self.assertIn("duplicate cpp_members id", "\n".join(errors))

    def test_cpp_member_without_owner_policy_fails_unmapped(self) -> None:
        inventory = {
            "cpp_files": [{"path": "src/new_owner.cpp"}],
            "cpp_members": [
                {
                    "id": "cpp:src/new_owner.cpp:NewOwner::setValue#1",
                    "owner_family": "new-owner",
                }
            ],
            "cpp_owner_policies": [],
            "rust_items": [],
        }
        self.assertIn(
            "[new-owner] unmapped C++ member",
            "\n".join(behavior_inventory.validate_inventory(inventory)),
        )

    def test_header_gap_approval_is_exact_member_bound(self) -> None:
        inventory = {
            "cpp_members": [
                {
                    "id": "cpp:include/rive/a.hpp:A::old@1",
                    "correspondence": "unmapped",
                },
                {
                    "id": "cpp:include/rive/a.hpp:A::new@2",
                    "correspondence": "unmapped",
                },
            ]
        }
        expected = {
            "cpp_members": [
                {
                    "id": "cpp:include/rive/a.hpp:A::old@1",
                    "correspondence": "reviewed-gap",
                }
            ]
        }
        behavior_inventory.approve_header_gaps(inventory, expected, False)
        self.assertEqual("reviewed-gap", inventory["cpp_members"][0]["correspondence"])
        self.assertEqual("unmapped", inventory["cpp_members"][1]["correspondence"])

    def test_adaptation_policy_requires_exact_item_owner_and_constraints(self) -> None:
        inventory = {
            "cpp_members": [
                {
                    "id": "cpp:src/lua/lua_gpu.cpp:Context::shader#1",
                    "owner_family": "lua",
                    "owner_policy": "cpp-owner:lua-gpu",
                }
            ],
            "cpp_owner_policies": [
                {
                    "id": "cpp-owner:lua-gpu",
                    "cpp_owner": "src/lua/lua_gpu.cpp",
                    "disposition": "adapted",
                    "rust_modules": ["crates/demo/src/gpu.rs"],
                }
            ],
            "rust_items": [
                {
                    "id": "rust:crates/demo/src/gpu.rs::shader#1",
                    "owner_family": "lua",
                    "provenance": "adaptation",
                    "baseline_cpp_owners": ["src/lua/lua_gpu.cpp"],
                    "allowed_call_direction": "cpp-owner-to-rust-item",
                    "forbidden_baseline_effects": ["skip-baseline-draw"],
                    "evidence": ["tests/gpu.rs::fresh_shader"],
                }
            ],
        }
        self.assertEqual([], behavior_inventory.validate_inventory(inventory))
        inventory["rust_items"][0]["evidence"] = []
        self.assertIn(
            "required evidence",
            "\n".join(behavior_inventory.validate_inventory(inventory)),
        )

    def test_standalone_seam_policy_is_validated_fail_closed(self) -> None:
        inventory = {
            "cpp_files": [{"path": "src/real.cpp"}],
            "cpp_members": [],
            "cpp_owner_policies": [],
            "rust_files": [{"path": "crates/demo/src/lib.rs"}],
            "rust_items": [],
            "seam_policies": [
                {
                    "id": "seam:invalid",
                    "provenance": "adaptation",
                    "adaptation": "invalid",
                    "rust_paths": ["crates/demo/src/missing.rs"],
                    "item_selector": "all-items-or-module",
                    "baseline_cpp_owners": ["src/missing.cpp"],
                    "allowed_call_direction": "sideways",
                    "forbidden_baseline_effects": [],
                    "evidence": [],
                }
            ],
        }
        errors = "\n".join(behavior_inventory.validate_inventory(inventory))
        self.assertIn("exact Rust path binding", errors)
        self.assertIn("exact baseline C++ owner", errors)
        self.assertIn("allowed call direction", errors)
        self.assertIn("forbidden baseline effects", errors)
        self.assertIn("required evidence", errors)

    def test_named_extension_binds_the_exact_rust_item(self) -> None:
        item = {
            "path": "crates/nuxie-runtime/src/artboard.rs",
            "context": "impl ArtboardInstance",
            "name": "try_semantic_geometry_revision",
            "region": "handwritten",
        }
        behavior_inventory.enrich_rust_item(item, None)
        self.assertEqual("extension", item["provenance"])
        self.assertEqual("semantic-geometry-cache-authority", item["extension"])
        self.assertEqual(
            ["src/artboard.cpp", "src/shapes/clipping_shape.cpp"],
            item["baseline_cpp_owners"],
        )

    def test_stale_named_extension_selector_fails_snapshot_generation(self) -> None:
        stale_selector = (
            "crates/demo/src/lib.rs",
            "module",
            "removed_extension",
        )
        with mock.patch.dict(
            behavior_inventory.NAMED_EXTENSION_RULES,
            {
                stale_selector: (
                    "removed-extension",
                    ["src/demo.cpp"],
                    ["skip-baseline-effect"],
                    ["docs/PORTING.md:X1"],
                )
            },
            clear=True,
        ):
            with self.assertRaisesRegex(
                ValueError, "named extension selectors do not resolve"
            ):
                behavior_inventory.compact_seam_policies([])

    def test_only_exact_reviewed_host_items_escape_unmapped(self) -> None:
        inventory = {
            "cpp_members": [],
            "rust_items": [
                {
                    "id": "rust:crates/demo/src/lib.rs::module::reviewed#1",
                    "provenance": "unmapped",
                },
                {
                    "id": "rust:crates/demo/src/lib.rs::module::new_item#1",
                    "provenance": "unmapped",
                },
            ],
        }
        expected = {
            "rust_items": [
                {
                    "id": "rust:crates/demo/src/lib.rs::module::reviewed#1",
                    "provenance": "host-support",
                }
            ]
        }
        behavior_inventory.approve_host_support(inventory, expected, False)
        self.assertEqual(
            ["host-support", "unmapped"],
            [item["provenance"] for item in inventory["rust_items"]],
        )
        self.assertIn(
            "unmapped Rust item",
            "\n".join(behavior_inventory.validate_inventory(inventory)),
        )

    def test_check_mode_rejects_a_new_member_with_family_diagnostic(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            expected = root / "inventory.json"
            expected.write_text(
                json.dumps({"cpp_members": [], "rust_items": []}) + "\n"
            )
            actual = {
                "cpp_members": [
                    {
                        "id": "cpp:src/shapes/new.cpp:Shape::setX#1",
                        "owner_family": "shapes",
                        "content_sha256": "x",
                    }
                ],
                "rust_items": [],
            }
            errors = behavior_inventory.check_snapshot(expected, actual)
            self.assertIn("[shapes] new", "\n".join(errors))


if __name__ == "__main__":
    unittest.main()
