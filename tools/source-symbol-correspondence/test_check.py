import importlib.util
import json
import pathlib
import sys
import tempfile
import unittest


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


class DispositionCheckTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = pathlib.Path(self.temp.name) / "dispositions.json"
        self.denominator = {
            "upstream_ref": "abc",
            "owners": [
                {
                    "symbols": [
                        {"id": "src/a.cpp::one:1"},
                        {"id": "src/a.cpp::two:1"},
                    ]
                }
            ],
        }

    def write(self, rows: list[dict[str, str]]) -> None:
        self.path.write_text(
            json.dumps(
                {
                    "schema": CHECK.DISPOSITIONS_SCHEMA,
                    "upstream_ref": "abc",
                    "symbols": rows,
                }
            )
        )

    def test_complete_dispositions_pass(self) -> None:
        self.write(
            [
                {"id": "src/a.cpp::one:1", "disposition": "mechanically-equivalent"},
                {
                    "id": "src/a.cpp::two:1",
                    "disposition": "equivalent-under-adaptation",
                    "adaptation": "Taffy layout ceiling",
                },
            ]
        )
        self.assertEqual(CHECK.check_dispositions(self.denominator, self.path), [])

    def test_missing_unknown_duplicate_and_unjustified_rows_fail(self) -> None:
        self.write(
            [
                {
                    "id": "src/a.cpp::one:1",
                    "disposition": "equivalent-under-adaptation",
                },
                {"id": "src/a.cpp::one:1", "disposition": "made-up"},
                {"id": "unknown", "disposition": "mechanically-equivalent"},
            ]
        )
        errors = CHECK.check_dispositions(self.denominator, self.path)
        self.assertTrue(any("duplicate" in error for error in errors))
        self.assertTrue(any("named adaptation" in error for error in errors))
        self.assertTrue(any("lack dispositions" in error for error in errors))
        self.assertTrue(any("unknown symbol" in error for error in errors))


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
