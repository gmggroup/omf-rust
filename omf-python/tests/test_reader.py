from os import path
from unittest import TestCase

import omf2


class TestReader(TestCase):
    def setUp(self) -> None:
        self.examples_dir = path.join(path.dirname(__file__), "../../examples")

    def test_should_return_expected_project_name(self) -> None:
        # Given
        omf_file = path.join(self.examples_dir, "pyramid/pyramid.omf")

        # When
        reader = omf2.Reader(omf_file)

        # Then
        self.assertEqual(reader.version(), [2, 0])

        project, _ = reader.project()
        self.assertEqual(project.name, "Pyramid")

    def test_should_raise_exception_for_validation_error(self) -> None:
        # Given
        onf_file_with_error = path.join(
            path.dirname(__file__), "data/missing_parquet.omf"
        )
        reader = omf2.Reader(onf_file_with_error)

        with self.assertRaises(omf2.OmfValidationFailedException) as context:
            reader.project()

        self.assertEqual(
            str(context.exception),
            "OMF validation failed:\n"
            "  Error: 'PointSet::vertices' refers to non-existent archive member '1.parquet', inside 'Missing'",
        )

    def test_should_return_expected_problems(self) -> None:
        # Given
        onf_file_with_problem = path.join(
            path.dirname(__file__), "data/duplicate_element_name.omf"
        )
        reader = omf2.Reader(onf_file_with_problem)
        _, problems = reader.project()

        # Then
        self.assertEqual(len(problems), 1)

        problem = problems[0]

        self.assertEqual(
            str(problem),
            "Warning: 'Project::elements[..]::name' contains duplicate of \"Duplicate\", inside 'Duplicate Element Name Test'",
        )
        self.assertEqual(problem.name, "Duplicate Element Name Test")
        self.assertEqual(problem.field, "elements[..]::name")
        self.assertEqual(problem.reason, 'contains duplicate of "Duplicate"')
        self.assertEqual(problem.type_name, "Project")
        self.assertEqual(problem.is_error(), False)

    def test_should_raise_expected_invalid_data_exception(self) -> None:
        # Given
        onf_file_with_array_length_mismatch = path.join(
            path.dirname(__file__), "data/array_length_mismatch.omf"
        )

        reader = omf2.Reader(onf_file_with_array_length_mismatch)
        project, _ = reader.project()
        vertices_array = project.elements()[0].geometry().vertices

        # When
        with self.assertRaises(omf2.OmfInvalidDataException) as context:
            reader.array_vertices(vertices_array)

        # Then
        self.assertEqual(
            str(context.exception),
            "Data error: Error: array length 999 does not match the declared length 4",
        )

    def test_should_raise_expected_file_not_found_exception(self) -> None:
        # Given
        incorrect_location = path.join(self.examples_dir, "testfilenotfound.omf")

        # When
        with self.assertRaises(omf2.OmfFileIoException) as context:
            omf2.Reader(incorrect_location)

        # Then
        self.assertIn("(os error 2)", str(context.exception))

    def test_should_return_expected_default_limits(self) -> None:
        # Given
        omf_file = path.join(self.examples_dir, "pyramid/pyramid.omf")
        reader = omf2.Reader(omf_file)

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
        reader = omf2.Reader(omf_file)

        limits = omf2.Limits()
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
        reader = omf2.Reader(omf_file)

        limits = reader.limits()
        limits.json_bytes = 0

        # When
        reader.set_limits(limits)
        with self.assertRaises(omf2.OmfJsonException) as context:
            reader.project()

        # Then
        self.assertEqual(
            str(context.exception),
            "JSON deserialization error: Error: safety limit exceeded",
        )
