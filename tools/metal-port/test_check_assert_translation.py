import importlib.util
import sys
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_assert_translation.py")
SPEC = importlib.util.spec_from_file_location("check_assert_translation", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class AssertTranslationTests(unittest.TestCase):
    def test_lexical_counts_ignore_comments_and_literals(self):
        counts = MODULE.assertion_counts(
            ['assert(x); // assert(y)\nconst char* s = "assert(z)";'],
            ['debug_assert!(x); assert!(y); // debug_assert!(z)\n"assert!(q)"'],
        )
        self.assertEqual(counts, MODULE.Counts(1, 1, 1))

    def test_hostile_debug_to_release_mutation_changes_both_counts(self):
        before = MODULE.assertion_counts([], ["debug_assert!(condition);"])
        after = MODULE.assertion_counts([], ["assert!(condition);"])
        self.assertEqual(before, MODULE.Counts(0, 1, 0))
        self.assertEqual(after, MODULE.Counts(0, 0, 1))

    def test_worker_boundary_debug_abort_counts_as_ndebug_assertion(self):
        counts = MODULE.assertion_counts([], ["debug_assert_abort!();"])
        self.assertEqual(counts, MODULE.Counts(0, 1, 0))

    def test_manifest_has_one_mutation_probe_for_each_fixed_unit(self):
        fixed_units = {value[0] for value in MODULE.MUTATIONS.values()}
        self.assertEqual(
            fixed_units,
            {"generic-rive-types", "generic-gradient", "generic-rive-renderer"},
        )


if __name__ == "__main__":
    unittest.main()
