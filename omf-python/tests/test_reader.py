import omf_python
from os import path
from unittest import TestCase


class TestReader(TestCase):
    def setUp(self) -> None:
        self.examples_dir = path.join(path.dirname(__file__), "../../examples")

    def test_should_return_expected_project_name(self) -> None:
        # Given
        omf_file = path.join(self.examples_dir, "pyramid/pyramid.omf")

        # When
        reader = omf_python.Reader(omf_file)

        # Then
        project, _ = reader.project()
        self.assertEqual(project.name, "Pyramid")

    def test_should_return_expected_problems(self) -> None:
        # Given
        onf_file_with_problem = path.join(path.dirname(__file__), "data/problem.omf")
        reader = omf_python.Reader(onf_file_with_problem)
        _, problems = reader.project()

        # Then
        self.assertEqual(len(problems), 1)

        problem = problems[0]

        self.assertEqual(str(problem), "Warning: \'Element::attributes[..]::name\' contains duplicate of \"Numbers\", inside 'Pyramid surface'")
        self.assertEqual(problem.name, "Pyramid surface")
        self.assertEqual(problem.field, "attributes[..]::name")
        self.assertEqual(problem.reason, "contains duplicate of \"Numbers\"")
        self.assertEqual(problem.type_name, "Element")
        self.assertEqual(problem.is_error(), False)

    def test_should_raise_expected_file_not_found_exception(self) -> None:
        # Given
        incorrect_location = path.join(self.examples_dir, "testfilenotfound.omf")

        # When
        with self.assertRaises(OSError) as context:
            omf_python.Reader(incorrect_location)

        # Then
        self.assertEqual(str(context.exception), "No such file or directory (os error 2)")
