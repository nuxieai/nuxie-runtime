import importlib.util
import json
import pathlib
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


TOOL = pathlib.Path(__file__).with_name("check.py")
SPEC = importlib.util.spec_from_file_location("source_symbol_correspondence", TOOL)
assert SPEC is not None and SPEC.loader is not None
CHECK = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = CHECK
SPEC.loader.exec_module(CHECK)


class SourceSymbolExtractionTest(unittest.TestCase):
    def symbols(self, source: str) -> list[str]:
        return [definition.symbol for definition in CHECK.extract_definitions(source)]

    def test_extracts_namespaces_overloads_operators_and_constructor(self) -> None:
        source = r'''
        namespace rive {
        int free_function(int value) { return value; }
        int Thing::run() const { return 1; }
        int Thing::run(int value) { return value; }
        Thing::Thing(int value) : m_value{value}, m_other(make()) {}
        bool Thing::operator==(const Thing& other) const { return false; }
        Thing::operator bool() const noexcept { return true; }
        }
        extern "C" { int rive_open(void* value) { return value != nullptr; } }
        '''
        self.assertEqual(
            self.symbols(source),
            [
                "free_function",
                "Thing::run",
                "Thing::run",
                "Thing::Thing",
                "Thing::operator==",
                "Thing::operatorbool",
                "rive_open",
            ],
        )

    def test_extracts_direct_extern_c_function_definition(self) -> None:
        source = r'''
        extern "C" int rive_open(Context* context)
        {
            return context == nullptr ? 0 : 1;
        }
        '''
        self.assertEqual(self.symbols(source), ["rive_open"])

    def test_extracts_specialized_names_and_uncommon_operators(self) -> None:
        source = r'''
        template <> int Traits::value<std::vector<int>>() { return 1; }
        void* Thing::operator new(size_t size) { return allocate(size); }
        void Thing::operator delete(void* value) { release(value); }
        long double operator "" _deg(long double value) { return value; }
        auto Thing::operator<=>(const Thing&) const { return 0; }
        '''
        self.assertEqual(
            self.symbols(source),
            [
                "Traits::value<std::vector<int>>",
                "Thing::operatornew",
                "Thing::operatordelete",
                'operator""_deg',
                "Thing::operator<=>",
            ],
        )

    def test_ignores_inline_class_methods_lambdas_and_function_body_calls(self) -> None:
        source = r'''
        struct Local {
            int inline_method() { return 1; }
        };
        static int actual(int value)
        {
            auto lambda = [](int nested) { return nested; };
            if (value) { return helper(value); }
            return lambda(value);
        }
        constexpr Local global{1};
        '''
        self.assertEqual(self.symbols(source), ["actual"])

    def test_literals_comments_and_preprocessor_do_not_create_braces(self) -> None:
        source = r'''
        #define BODY(x) { x; }
        // fake() { }
        const char* text() {
            return R"tag(/* { } */)tag";
        }
        /* also_fake() { } */
        '''
        self.assertEqual(self.symbols(source), ["text"])

    def test_conditional_duplicate_definitions_are_both_denominator_entries(self) -> None:
        source = "#if A\nint mode() { return 1; }\n#else\nint mode() { return 2; }\n#endif\n"
        definitions = CHECK.extract_definitions(source)
        self.assertEqual([item.symbol for item in definitions], ["mode", "mode"])
        self.assertNotEqual(definitions[0].fingerprint, definitions[1].fingerprint)

    def test_extracts_inline_templates_constructors_destructors_and_operators(self) -> None:
        source = r'''
        namespace rive {
        template <class T>
        struct Box {
            Box(
                T value
            ) : value{value} {}
            ~Box() { release(); }
            template <typename U = T>
            U convert(
                const U& fallback
            ) const noexcept { return U(value); }
            T operator[](size_t index) const { return data[index]; }
            explicit operator bool() const { return value != T{}; }
            friend bool operator==(const Box& a, const Box& b) { return a.value == b.value; }
            T value;
            T data[1];
        };
        template <class T, class... Args>
        static T* make(Args&&... args) { return new T(args...); }
        }
        '''
        definitions = CHECK.extract_definitions(source, include_inline_class=True)
        self.assertEqual(
            [definition.symbol for definition in definitions],
            [
                "Box::Box",
                "Box::~Box",
                "Box::convert",
                "Box::operator[]",
                "Box::operatorbool",
                "operator==",
                "make",
            ],
        )

    def test_inline_constructor_parenthesized_initializers_are_not_functions(self) -> None:
        source = r'''
        template <typename T>
        struct alignas(32) Child : Parent {
            Child(T value) : Parent(value), m_value(value), m_callback(make(value)) {}
        private:
            T m_value;
            Callback m_callback;
        };
        '''
        self.assertEqual(
            [
                definition.symbol
                for definition in CHECK.extract_definitions(
                    source, include_inline_class=True
                )
            ],
            ["Child::Child"],
        )

    def test_array_subscripts_in_constructor_initializers_are_not_lambdas(self) -> None:
        source = r'''
        struct Eval {
            Eval(const Point points[2]) : Eval(points[0], points[1]) {}
            Eval(Point first, Point second) : m_first{first}, m_second{second} {}
            Point operator()(float t) const { return m_first * t + m_second; }
            Point m_first, m_second;
        };
        '''
        definitions = CHECK.extract_definitions(
            source,
            include_inline_class=True,
            include_lexical_fallbacks=True,
        )
        self.assertEqual(
            [definition.symbol for definition in definitions if definition.kind == "function"],
            ["Eval::Eval", "Eval::Eval", "Eval::operator()"],
        )
        self.assertEqual(
            len(
                [
                    definition
                    for definition in definitions
                    if definition.kind == "lexical-brace-authority"
                ]
            ),
            2,
        )

    def test_probable_member_initializer_function_names_fail_the_corpus_gate(self) -> None:
        owners = [
            {
                "upstream": "include/rive/example.hpp",
                "symbols": [
                    {"kind": "function", "symbol": "Example::m_value", "line": 12}
                ],
            }
        ]
        with self.assertRaisesRegex(ValueError, "probable constructor initializer"):
            CHECK._require_plausible_function_symbols(owners)

    def test_frozen_pinned_corpus_has_no_member_initializer_function_names(self) -> None:
        snapshot = json.loads(
            (TOOL.parents[2] / "docs/runtime-source-certification/symbol-denominator.json")
            .read_text()
        )
        invalid = [
            (owner["upstream"], symbol["symbol"])
            for owner in snapshot["owners"]
            for symbol in owner["symbols"]
            if symbol["kind"] == "function"
            and symbol["symbol"].split("::")[-1].startswith("m_")
        ]
        self.assertEqual(invalid, [])

    def test_inline_extraction_rejects_lambdas_and_braced_member_data(self) -> None:
        source = r'''
        struct Callbacks {
            std::function<void()> callback = []() { helper(); };
            std::function<int(int)> mapped{[](int value) { return value + 1; }};
            std::vector<int> values{1, 2, 3};
            int real() { return callback(), mapped(1); }
        };
        '''
        self.assertEqual(
            [
                definition.symbol
                for definition in CHECK.extract_definitions(
                    source, include_inline_class=True
                )
            ],
            ["Callbacks::real"],
        )

    def test_header_fallbacks_make_unclassified_braces_explicit(self) -> None:
        source = r'''
        struct Values {
            std::function<int()> callback{[]() { return 1; }};
            std::array<int, 2> values{1, 2};
            int real() { return callback(); }
        };
        '''
        definitions = CHECK.extract_definitions(
            source,
            include_inline_class=True,
            include_lexical_fallbacks=True,
        )
        self.assertEqual(
            [definition.kind for definition in definitions],
            ["lexical-brace-authority", "lexical-brace-authority", "function"],
        )
        self.assertEqual(definitions[-1].symbol, "Values::real")

    def test_nested_and_anonymous_class_bodies_are_never_silent(self) -> None:
        source = r'''
        struct Outer {
            struct Inner { int run() { return 1; } };
            struct { int call() { return 2; } } anonymous;
        };
        '''
        symbols = [
            definition.symbol
            for definition in CHECK.extract_definitions(source, include_inline_class=True)
        ]
        self.assertEqual(symbols[0], "Outer::Inner::run")
        self.assertRegex(symbols[1], r"^Outer::<anonymous-class@[0-9]+>::call$")

    def test_source_local_class_inline_methods_are_extractable(self) -> None:
        source = r'''
        class LocalWorker {
        public:
            LocalWorker(int value) : m_value(value) {}
            int run() { return m_value; }
        private:
            int m_value;
        };
        '''
        self.assertEqual(
            [
                definition.symbol
                for definition in CHECK.extract_definitions(
                    source, include_inline_class=True
                )
            ],
            ["LocalWorker::LocalWorker", "LocalWorker::run"],
        )

    def test_qualified_source_local_class_keeps_complete_owner_name(self) -> None:
        source = r'''
        class Server::Loader : public Base {
        public:
            Loader(Server* server, Source source) : m_server(server), m_source(source) {}
            bool load() { return m_source.ready(); }
        };
        '''
        self.assertEqual(
            [
                definition.symbol
                for definition in CHECK.extract_definitions(
                    source, include_inline_class=True
                )
            ],
            ["Server::Loader::Loader", "Server::Loader::load"],
        )

    def test_source_statements_include_data_and_defaulted_definitions(self) -> None:
        source = r'''
        using Alias = int;
        class Forward;
        int declared(int value);
        extern int external_value;
        int Counter::value = 0;
        std::mutex Counter::mutex;
        const Entry entries[] = {{"open", open_callback}};
        Worker::~Worker() = default;
        '''
        definitions = CHECK.extract_source_statements(source)
        self.assertEqual(
            [definition.signature for definition in definitions],
            [
                "source-statement int Counter :: value = 0",
                "source-statement std :: mutex Counter :: mutex",
                'source-statement const Entry entries [ ] = { { "open" , open_callback } }',
                "source-statement Worker :: ~ Worker ( ) = default",
            ],
        )


class MacroAuthorityExtractionTest(unittest.TestCase):
    def test_all_defines_and_body_macro_invocations_are_explicit(self) -> None:
        source = r'''
        #define HEADER_GUARD
        #define VALUE 42
        #define DECL_OP(name, op)                 \
            int operator op(int value) const      \
            {                                      \
                return name + value;               \
            }
        struct Number {
            DECL_OP(base, +)
        };
        '''
        macros = CHECK.extract_macro_definitions(source)
        self.assertEqual(
            [definition.symbol for definition in macros],
            ["macro HEADER_GUARD", "macro VALUE", "macro DECL_OP"],
        )
        self.assertEqual(CHECK.executable_macro_names(source), {"DECL_OP"})
        invocations = CHECK.extract_macro_invocations(source, {"DECL_OP"})
        self.assertEqual(len(invocations), 1)
        self.assertEqual(invocations[0].symbol, "macro-invocation DECL_OP")
        self.assertEqual(invocations[0].signature, "DECL_OP ( base , + )")

    def test_macro_tokens_in_comments_literals_and_definitions_are_not_invocations(self) -> None:
        source = r'''
        #define BODY() do { work(); } while (false)
        // BODY()
        const char* text = "BODY()";
        void actual() { BODY(); }
        '''
        invocations = CHECK.extract_macro_invocations(source, {"BODY"})
        self.assertEqual(len(invocations), 1)
        self.assertEqual(invocations[0].line, 5)

    def test_body_macro_statement_does_not_contaminate_following_function(self) -> None:
        source = r'''
        #define DECL(name) int name() { return 1; }
        DECL(first)
        DECL(second)
        inline int ordinary() { return 2; }
        '''
        definitions = CHECK.extract_definitions(
            source,
            macro_statement_names={"DECL"},
        )
        self.assertEqual([definition.symbol for definition in definitions], ["ordinary"])
        self.assertEqual(definitions[0].line, 5)
        self.assertEqual(definitions[0].signature, "inline int ordinary ( )")

    def test_source_local_macros_and_unclassified_braces_are_explicit(self) -> None:
        source = r'''
        #define LOCAL_LIMIT 16
        static const Callback callbacks[] = {
            {"open", open_callback},
            {"close", close_callback},
        };
        int ordinary() { return LOCAL_LIMIT; }
        '''
        definitions = CHECK.extract_definitions(
            source, include_lexical_fallbacks=True
        )
        definitions.extend(CHECK.extract_macro_definitions(source))
        self.assertEqual(
            sorted(definition.kind for definition in definitions),
            ["function", "lexical-brace-authority", "macro-definition"],
        )


class GeneratedAuthorityTest(unittest.TestCase):
    def test_authority_set_freezes_paths_sizes_and_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "a").write_bytes(b"one")
            (root / "b").write_bytes(b"two")
            first = CHECK._authority_set(root, ["b", "a"])
            second = CHECK._authority_set(root, ["a", "b"])
            self.assertEqual(first, second)
            self.assertEqual(first["file_count"], 2)
            self.assertEqual(first["byte_count"], 6)
            self.assertEqual([row["path"] for row in first["files"]], ["a", "b"])

    def test_schema_replay_requires_byte_exact_codegen_output(self) -> None:
        with tempfile.TemporaryDirectory() as repo, tempfile.TemporaryDirectory() as upstream:
            repo_root = pathlib.Path(repo)
            upstream_root = pathlib.Path(upstream)
            schema = repo_root / "crates/nuxie-schema/src/generated/schema.rs"
            schema.parent.mkdir(parents=True)
            schema.write_text("exact\n")
            (upstream_root / "dev/defs").mkdir(parents=True)

            def replay(command, **kwargs):
                pathlib.Path(command[-1]).write_text("exact\n")
                return mock.Mock(returncode=0, stderr="")

            with mock.patch.object(CHECK.subprocess, "run", side_effect=replay):
                self.assertEqual(
                    CHECK.verify_generated_schema_replay(repo_root, upstream_root), []
                )

            def drift(command, **kwargs):
                pathlib.Path(command[-1]).write_text("drift\n")
                return mock.Mock(returncode=0, stderr="")

            with mock.patch.object(CHECK.subprocess, "run", side_effect=drift):
                errors = CHECK.verify_generated_schema_replay(repo_root, upstream_root)
            self.assertTrue(any("differs" in error for error in errors))

    def test_tracked_upstream_authority_changes_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "src").mkdir()
            source = root / "src/example.cpp"
            source.write_text("int example() { return 1; }\n")
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "add", "src/example.cpp"], check=True)
            subprocess.run(
                [
                    "git",
                    "-C",
                    str(root),
                    "-c",
                    "user.name=Source Census Test",
                    "-c",
                    "user.email=source-census@example.invalid",
                    "commit",
                    "-qm",
                    "fixture",
                ],
                check=True,
            )
            CHECK._require_clean_tracked_authority(root)
            source.write_text("int example() { return 2; }\n")
            with self.assertRaisesRegex(ValueError, "tracked changes"):
                CHECK._require_clean_tracked_authority(root)


class DispositionCheckTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.repo_root = pathlib.Path(self.temp.name)
        self.path = self.repo_root / "dispositions.json"
        receipts = self.repo_root / "docs/runtime-source-certification"
        receipts.mkdir(parents=True)
        for name in ("a.md", "one.md", "two.md", "wrapper.md"):
            (receipts / name).write_text(f"# {name}\n")
        self.denominator = {
            "upstream_ref": "abc",
            "owners": [
                {
                    "upstream": "src/a.cpp",
                    "symbol_count": 2,
                    "symbols": [
                        {"id": "src/a.cpp::one:1"},
                        {"id": "src/a.cpp::two:1"},
                    ]
                }
            ],
        }

    def write(
        self,
        rows: list[dict[str, object]],
        owners: list[dict[str, object]] | None = None,
    ) -> None:
        self.path.write_text(
            json.dumps(
                {
                    "schema": CHECK.DISPOSITIONS_SCHEMA,
                    "upstream_ref": "abc",
                    "owners": owners
                    or [
                        {
                            "upstream": "src/a.cpp",
                            "receipt": "docs/runtime-source-certification/a.md",
                            "independent_reviews": self.accepted_reviews(),
                        }
                    ],
                    "symbols": rows,
                }
            )
        )

    @staticmethod
    def accepted_reviews() -> list[dict[str, str]]:
        return [
            {"status": "accepted", "reviewer": "adversarial lane one"},
            {"status": "accepted", "reviewer": "adversarial lane two"},
        ]

    def test_complete_dispositions_pass(self) -> None:
        self.write(
            [
                {
                    "id": "src/a.cpp::one:1",
                    "disposition": "exact",
                    "rust_owners": ["crate::one"],
                    "receipt": "docs/runtime-source-certification/one.md",
                    "independent_reviews": self.accepted_reviews(),
                    "evidence": ["one_matches_cpp"],
                },
                {
                    "id": "src/a.cpp::two:1",
                    "disposition": "adapted",
                    "adaptation": "Taffy layout ceiling",
                    "rust_owners": ["crate::two"],
                    "receipt": "docs/runtime-source-certification/two.md",
                    "independent_reviews": self.accepted_reviews(),
                    "evidence_exemption": "compile-time forwarding wrapper",
                },
            ]
        )
        self.assertEqual(
            CHECK.check_dispositions(
                self.denominator, self.path, repo_root=self.repo_root
            ),
            [],
        )

    def test_receipts_must_be_existing_certification_documents(self) -> None:
        rows = [
            {
                "id": "src/a.cpp::one:1",
                "disposition": "exact",
                "rust_owners": ["crate::one"],
                "receipt": "docs/runtime-source-certification/missing.md",
                "independent_reviews": self.accepted_reviews(),
                "evidence": ["one_matches_cpp"],
            },
            {
                "id": "src/a.cpp::two:1",
                "disposition": "exact",
                "rust_owners": ["crate::two"],
                "receipt": "../outside.md",
                "independent_reviews": self.accepted_reviews(),
                "evidence": ["two_matches_cpp"],
            },
        ]
        self.write(rows)
        errors = CHECK.check_dispositions(
            self.denominator, self.path, repo_root=self.repo_root
        )
        self.assertTrue(any("missing receipt file" in error for error in errors))
        self.assertTrue(any("outside the repository" in error for error in errors))

    def test_missing_unknown_duplicate_and_unjustified_rows_fail(self) -> None:
        self.write(
            [
                {
                    "id": "src/a.cpp::one:1",
                    "disposition": "adapted",
                },
                {"id": "src/a.cpp::one:1", "disposition": "made-up"},
                {"id": "unknown", "disposition": "exact"},
            ]
        )
        errors = CHECK.check_dispositions(
            self.denominator, self.path, repo_root=self.repo_root
        )
        self.assertTrue(any("duplicate" in error for error in errors))
        self.assertTrue(any("named adaptation" in error for error in errors))
        self.assertTrue(any("lack dispositions" in error for error in errors))
        self.assertTrue(any("unknown symbol" in error for error in errors))

    def test_not_applicable_and_missing_require_governance(self) -> None:
        self.write(
            [
                {"id": "src/a.cpp::one:1", "disposition": "not-applicable"},
                {"id": "src/a.cpp::two:1", "disposition": "missing"},
            ]
        )
        errors = CHECK.check_dispositions(
            self.denominator, self.path, repo_root=self.repo_root
        )
        self.assertTrue(any("governing decision" in error for error in errors))
        self.assertTrue(any("lacks tracking" in error for error in errors))

    def test_owner_ledger_is_bijective_and_independently_reviewed(self) -> None:
        self.write(
            [],
            owners=[
                {"upstream": "src/a.cpp"},
                {"upstream": "src/a.cpp"},
                {"upstream": "unknown.hpp"},
            ],
        )
        errors = CHECK.check_dispositions(
            self.denominator, self.path, repo_root=self.repo_root
        )
        self.assertTrue(any("duplicate owner" in error for error in errors))
        self.assertTrue(any("owner lacks a receipt" in error for error in errors))
        self.assertTrue(any("two accepted independent reviews" in error for error in errors))
        self.assertTrue(any("unknown owner" in error for error in errors))

    def test_zero_unit_owner_requires_explicit_decision(self) -> None:
        denominator = {
            "upstream_ref": "abc",
            "owners": [
                {"upstream": "src/wrapper.mm", "symbol_count": 0, "symbols": []}
            ],
        }
        self.write(
            [],
            owners=[
                {
                    "upstream": "src/wrapper.mm",
                    "receipt": "docs/runtime-source-certification/wrapper.md",
                    "independent_reviews": self.accepted_reviews(),
                }
            ],
        )
        errors = CHECK.check_dispositions(
            denominator, self.path, repo_root=self.repo_root
        )
        self.assertTrue(any("no-executable-units decision" in error for error in errors))

    def test_reviews_must_be_accepted_and_have_distinct_reviewers(self) -> None:
        rows = [
            {
                "id": "src/a.cpp::one:1",
                "disposition": "exact",
                "rust_owners": ["crate::one"],
                "receipt": "docs/runtime-source-certification/one.md",
                "independent_reviews": [
                    {"status": "accepted", "reviewer": "same lane"},
                    {"status": "accepted", "reviewer": "same lane"},
                ],
                "evidence": ["one_matches_cpp"],
            },
            {
                "id": "src/a.cpp::two:1",
                "disposition": "exact",
                "rust_owners": ["crate::two"],
                "receipt": "docs/runtime-source-certification/two.md",
                "independent_reviews": [
                    {"status": "accepted", "reviewer": "lane one"},
                    {"status": "rejected", "reviewer": "lane two"},
                ],
                "evidence": ["two_matches_cpp"],
            },
        ]
        self.write(rows)
        errors = CHECK.check_dispositions(
            self.denominator, self.path, repo_root=self.repo_root
        )
        self.assertTrue(any("two distinct independent reviewers" in error for error in errors))
        self.assertTrue(any("invalid or unaccepted independent review" in error for error in errors))


class DenominatorCheckTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = pathlib.Path(self.temp.name) / "denominator.json"
        self.expected = {"schema": CHECK.DENOMINATOR_SCHEMA, "symbol_count": 1}

    def test_exact_snapshot_passes(self) -> None:
        self.path.write_text(json.dumps(self.expected))
        self.assertEqual(CHECK.check_denominator(self.expected, self.path), [])

    def test_snapshot_drift_fails(self) -> None:
        self.path.write_text(
            json.dumps({"schema": CHECK.DENOMINATOR_SCHEMA, "symbol_count": 2})
        )
        errors = CHECK.check_denominator(self.expected, self.path)
        self.assertTrue(any("drifted" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
