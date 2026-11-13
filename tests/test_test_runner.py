"""The CI partitions must collectively execute the complete existing test set."""
import argparse
from collections import Counter
import importlib.util
from pathlib import Path
import sys
import unittest

spec = importlib.util.spec_from_file_location('rusk_test_runner', Path(__file__).resolve().parents[1] / 'tools/test.py')
runner = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = runner
spec.loader.exec_module(runner)


class TestPartitions(unittest.TestCase):
    def test_all_cargo_targets_are_executed_exactly_once(self):
        expected = runner.cargo_targets()
        self.assertIn(('bin', 'rusk'), expected)
        for name in ('atcoder', 'library_migration', 'standard_expansion'):
            self.assertIn(('test', name), expected)
        for count in (1, 3, len(expected)):
            actual = [target for index in range(1, count + 1)
                      for target in runner.selected_targets('tests', (index, count))]
            self.assertEqual(Counter(actual), Counter(expected))

    def test_reference_targets_are_executed_exactly_once(self):
        expected = [('test', name) for name in runner.REFERENCE_TESTS]
        actual = [target for index in range(1, 4)
                  for target in runner.selected_targets('reference', (index, 3))]
        self.assertEqual(Counter(actual), Counter(expected))

    def test_unpartitioned_commands_preserve_existing_selection(self):
        self.assertEqual(runner.cargo_test_command('tests'), ['cargo', 'test', '--locked'])
        command = runner.cargo_test_command('reference')
        self.assertEqual(command[3:], [arg for name in runner.REFERENCE_TESTS for arg in ('--test', name)])

    def test_partition_command_keeps_unit_tests_and_integration_targets(self):
        command = runner.cargo_test_command('tests', (1, 1))
        self.assertIn('--bin', command)
        self.assertIn('rusk', command)
        self.assertIn('standard_expansion', command)
        self.assertEqual(command.count('--test'), len(list((runner.ROOT / 'tests').glob('*.rs'))))

    def test_invalid_partitions_cannot_succeed_without_running_tests(self):
        for text in ('0/3', '4/3', '1/0', '1/-2', '1', '1/2/3', 'a/b'):
            with self.assertRaises(argparse.ArgumentTypeError):
                runner.shard_value(text)
        with self.assertRaises(ValueError):
            runner.selected_targets('tests', (1, 10000))


if __name__ == '__main__':
    unittest.main()
