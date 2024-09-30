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
        self.assertEqual(reader.version(), [2, 0])

        project, _ = reader.project()
        self.assertEqual(project.name, "Pyramid")

    def test_should_raise_exception_for_validation_error(self) -> None:
        # Given
        onf_file_with_problem = path.join(path.dirname(__file__), "data/missing_member.omf")
        reader = omf_python.Reader(onf_file_with_problem)

        with self.assertRaises(omf_python.OmfValidationFailedException) as context:
            reader.project()

        self.assertEqual(str(context.exception),
            "OMF validation failed:\n"
            "  Error: 'Surface::triangles' refers to non-existent archive member '2.parquet', inside 'Pyramid surface'"
        )

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
        with self.assertRaises(omf_python.OmfFileIoException) as context:
            omf_python.Reader(incorrect_location)

        # Then
        self.assertEqual(str(context.exception), "File IO error: No such file or directory (os error 2)")

    def test_should_return_expected_default_limits(self) -> None:
        # Given
        omf_file = path.join(self.examples_dir, "pyramid/pyramid.omf")
        reader = omf_python.Reader(omf_file)

        # When
        limits = reader.limits()

        # Then
        self.assertEqual(limits.json_bytes, 1024 * 1024)
        self.assertEqual(limits.image_bytes, 16 * 1024 * 1024 * 1024)
        self.assertEqual(limits.image_dim, None)
        self.assertEqual(limits.validation, 100)

    def test_should_set_limits(self) -> None:
        # Given
        omf_file = path.join(self.examples_dir, "pyramid/pyramid.omf")
        reader = omf_python.Reader(omf_file)

        limits = omf_python.Limits()
        limits.json_bytes = 1
        limits.image_bytes = 2
        limits.image_dim = 3
        limits.validation = 4

        # When
        reader.set_limits(limits)

        # Then
        updated_limits = reader.limits()
        self.assertEqual(updated_limits.json_bytes, limits.json_bytes)
        self.assertEqual(updated_limits.image_bytes, limits.image_bytes)
        self.assertEqual(updated_limits.image_dim, limits.image_dim)
        self.assertEqual(updated_limits.validation, limits.validation)

    def test_should_raise_exception_if_json_bytes_limit_reached(self) -> None:
        # Given
        omf_file = path.join(self.examples_dir, "pyramid/pyramid.omf")
        reader = omf_python.Reader(omf_file)

        limits = reader.limits()
        limits.json_bytes = 0

        # When
        reader.set_limits(limits)
        with self.assertRaises(omf_python.OmfJsonException) as context:
            reader.project()

        # Then
        self.assertEqual(str(context.exception), "JSON deserialization error: Error: safety limit exceeded")
